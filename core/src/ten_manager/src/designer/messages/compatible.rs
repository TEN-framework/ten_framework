//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::sync::{Arc, RwLock};

use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use ten_rust::{
    graph::extension::{
        get_compatible_cmd_extension, get_compatible_cmd_extension_with_slice,
        get_compatible_data_like_msg_extension,
        get_compatible_data_like_msg_extension_with_slice, get_extension,
        get_extension_nodes_in_graph, get_pkg_info_for_extension,
        CompatibleExtensionAndMsg,
    },
    pkg_info::{
        message::{MsgDirection, MsgType},
        pkg_type::PkgType,
    },
};

use crate::designer::{
    response::{ApiResponse, ErrorResponse, Status},
    DesignerState,
};

use crate::designer::graphs::connections::add::create_uri_to_pkg_info_map;

/// Represents the request payload for retrieving compatible messages.
///
/// This struct contains all necessary information to identify a specific
/// message within a graph and find other messages that are compatible with it.
#[derive(Debug, Deserialize, Serialize)]
pub struct GetCompatibleMsgsRequestPayload {
    /// Base directory path where the project files are located.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_dir: Option<String>,

    /// ID of the graph to search for compatible messages.
    pub graph: Uuid,

    /// Optional application name that contains the extension.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app: Option<String>,

    /// Optional extension group that the extension belongs to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extension_group: Option<String>,

    /// Name of the extension to find compatible messages for.
    pub extension: String,

    /// Type of the message (e.g., "command", "data").
    pub msg_type: MsgType,

    /// Direction of the message (e.g., "input", "output").
    pub msg_direction: MsgDirection,

    /// Name of the specific message to find compatibility with.
    pub msg_name: String,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct GetCompatibleMsgsSingleResponseData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub extension_group: Option<String>,

    pub extension: String,
    pub msg_type: MsgType,
    pub msg_direction: MsgDirection,
    pub msg_name: String,
}

impl From<CompatibleExtensionAndMsg<'_>>
    for GetCompatibleMsgsSingleResponseData
{
    fn from(compatible: CompatibleExtensionAndMsg) -> Self {
        GetCompatibleMsgsSingleResponseData {
            app: compatible.extension.app.clone(),
            extension_group: compatible.extension.extension_group.clone(),
            extension: compatible.extension.type_and_name.name.clone(),
            msg_type: compatible.msg_type,
            msg_direction: compatible.msg_direction,
            msg_name: compatible.msg_name,
        }
    }
}

pub async fn get_compatible_messages_endpoint(
    request_payload: web::Json<GetCompatibleMsgsRequestPayload>,
    state: web::Data<Arc<RwLock<DesignerState>>>,
) -> Result<impl Responder, actix_web::Error> {
    let state_read = state.read().unwrap();

    // Create a hash map from app URIs to PkgsInfoInApp.
    let uri_to_pkg_info =
        match create_uri_to_pkg_info_map(&state_read.pkgs_cache) {
            Ok(map) => map,
            Err(error_response) => {
                return Ok(HttpResponse::BadRequest().json(error_response))
            }
        };

    // Get the first entry from pkgs_cache
    let base_dir_pkg_info =
        if let Some((_, pkg_info)) = state_read.pkgs_cache.iter().next() {
            pkg_info
        } else {
            let error_response = ErrorResponse {
                status: Status::Fail,
                message: "No packages available in the cache".to_string(),
                error: None,
            };
            return Ok(HttpResponse::NotFound().json(error_response));
        };

    if base_dir_pkg_info.app_pkg_info.is_none() {
        let error_response = ErrorResponse {
            status: Status::Fail,
            message: "Application package information is missing".to_string(),
            error: None,
        };
        return Ok(HttpResponse::NotFound().json(error_response));
    }

    // Get extension package information directly, if available.
    let extensions_slice = base_dir_pkg_info.get_extensions();

    let extensions = match get_extension_nodes_in_graph(
        &request_payload.graph,
        &state_read.graphs_cache,
    ) {
        Ok(exts) => exts,
        Err(err) => {
            let error_response = ErrorResponse::from_error(
                &err,
                format!(
                    "Error fetching runtime extensions for graph '{}'",
                    request_payload.graph
                )
                .as_str(),
            );
            return Ok(HttpResponse::NotFound().json(error_response));
        }
    };

    let extension = match get_extension(
        &extensions,
        &request_payload.app,
        &request_payload.extension_group,
        &request_payload.extension,
    ) {
        Ok(ext) => ext,
        Err(err) => {
            let error_response = ErrorResponse::from_error(
                &err,
                format!(
                    "Failed to find the extension: {}",
                    request_payload.extension
                )
                .as_str(),
            );

            return Ok(HttpResponse::NotFound().json(error_response));
        }
    };

    let msg_ty = request_payload.msg_type.clone();
    let msg_dir = request_payload.msg_direction.clone();

    let mut desired_msg_dir = msg_dir.clone();
    desired_msg_dir.toggle();

    // First try to find pkg_info using the uri_to_pkg_info map.
    let mut pkg_info = get_pkg_info_for_extension(extension, &uri_to_pkg_info);

    // For backward compatibility with tests, if no pkg_info is found in
    // uri_to_pkg_info, try to find it in extensions_slice
    if pkg_info.is_none() {
        pkg_info = extensions_slice.iter().find(|pkg| {
            if let Some(manifest) = &pkg.manifest {
                manifest.type_and_name.pkg_type == PkgType::Extension
                    && manifest.type_and_name.name == extension.addon
            } else {
                false
            }
        });
    }

    if pkg_info.is_none() {
        let error_response = ErrorResponse::from_error(
            &anyhow::anyhow!("Extension not found"),
            format!(
                "Error fetching runtime extensions for graph '{}'",
                request_payload.graph
            )
            .as_str(),
        );
        return Ok(HttpResponse::NotFound().json(error_response));
    }

    let pkg_info = pkg_info.unwrap();

    let compatible_list = match msg_ty {
        MsgType::Cmd => {
            let src_cmd_schema =
                pkg_info.schema_store.as_ref().and_then(|schema_store| {
                    match msg_dir {
                        MsgDirection::In => schema_store
                            .cmd_in
                            .get(request_payload.msg_name.as_str()),
                        MsgDirection::Out => schema_store
                            .cmd_out
                            .get(request_payload.msg_name.as_str()),
                    }
                });

            // First try with uri_to_pkg_info
            if let Ok(results) = get_compatible_cmd_extension(
                &extensions,
                &uri_to_pkg_info,
                &desired_msg_dir,
                src_cmd_schema,
                request_payload.msg_name.as_str(),
            ) {
                results
            } else {
                // If that fails, for backward compatibility with tests, try
                // with extensions_slice
                match get_compatible_cmd_extension_with_slice(
                    &extensions,
                    extensions_slice,
                    &desired_msg_dir,
                    src_cmd_schema,
                    request_payload.msg_name.as_str(),
                ) {
                    Ok(results) => results,
                    Err(err) => {
                        let error_response = ErrorResponse::from_error(
                            &err,
                            format!(
                                "Failed to find compatible cmd/{}:",
                                request_payload.msg_name
                            )
                            .as_str(),
                        );
                        return Ok(
                            HttpResponse::NotFound().json(error_response)
                        );
                    }
                }
            }
        }
        _ => {
            let src_msg_schema =
                pkg_info.schema_store.as_ref().and_then(|schema_store| {
                    match msg_ty {
                        MsgType::Data => match msg_dir {
                            MsgDirection::In => schema_store
                                .data_in
                                .get(request_payload.msg_name.as_str()),
                            MsgDirection::Out => schema_store
                                .data_out
                                .get(request_payload.msg_name.as_str()),
                        },
                        MsgType::AudioFrame => match msg_dir {
                            MsgDirection::In => schema_store
                                .audio_frame_in
                                .get(request_payload.msg_name.as_str()),
                            MsgDirection::Out => schema_store
                                .audio_frame_out
                                .get(request_payload.msg_name.as_str()),
                        },
                        MsgType::VideoFrame => match msg_dir {
                            MsgDirection::In => schema_store
                                .video_frame_in
                                .get(request_payload.msg_name.as_str()),
                            MsgDirection::Out => schema_store
                                .video_frame_out
                                .get(request_payload.msg_name.as_str()),
                        },
                        _ => panic!("should not happen."),
                    }
                });

            // First try with uri_to_pkg_info
            if let Ok(results) = get_compatible_data_like_msg_extension(
                &extensions,
                &uri_to_pkg_info,
                &desired_msg_dir,
                src_msg_schema,
                &msg_ty,
                request_payload.msg_name.clone(),
            ) {
                results
            } else {
                // If that fails, for backward compatibility with tests, try
                // with extensions_slice
                match get_compatible_data_like_msg_extension_with_slice(
                    &extensions,
                    extensions_slice,
                    &desired_msg_dir,
                    src_msg_schema,
                    &msg_ty,
                    request_payload.msg_name.clone(),
                ) {
                    Ok(results) => results,
                    Err(err) => {
                        let error_response = ErrorResponse::from_error(
                            &err,
                            format!(
                                "Failed to find compatible {}:",
                                request_payload.msg_name
                            )
                            .as_str(),
                        );
                        return Ok(
                            HttpResponse::NotFound().json(error_response)
                        );
                    }
                }
            }
        }
    };

    // Convert CompatibleExtensionAndMsg to
    // GetCompatibleMsgsSingleResponseData
    let response_data: Vec<GetCompatibleMsgsSingleResponseData> =
        compatible_list.into_iter().map(Into::into).collect();

    let response = ApiResponse {
        status: Status::Ok,
        data: response_data,
        meta: None,
    };

    Ok(HttpResponse::Ok().json(response))
}
