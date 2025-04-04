//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::sync::{Arc, RwLock};

use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

use ten_rust::pkg_info::{
    message::{MsgDirection, MsgType},
    predefined_graphs::extension::{
        get_compatible_cmd_extension, get_compatible_data_like_msg_extension,
        get_extension, get_extension_nodes_in_graph,
        get_pkg_info_for_extension, CompatibleExtensionAndMsg,
    },
};

use crate::designer::{
    response::{ApiResponse, ErrorResponse, Status},
    DesignerState,
};

/// Represents the request payload for retrieving compatible messages.
///
/// This struct contains all necessary information to identify a specific
/// message within a graph and find other messages that are compatible with it.
#[derive(Debug, Deserialize, Serialize)]
pub struct GetCompatibleMsgsRequestPayload {
    /// Base directory path where the project files are located.
    pub base_dir: String,

    /// Name of the graph to search for compatible messages.
    pub graph: String,

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

    if let Some(base_dir_pkg_info) =
        state_read.pkgs_cache.get(&request_payload.base_dir)
    {
        if base_dir_pkg_info.app_pkg_info.is_none() {
            let error_response = ErrorResponse {
                status: Status::Fail,
                message: "Application package information is missing"
                    .to_string(),
                error: None,
            };
            return Ok(HttpResponse::NotFound().json(error_response));
        }

        // Get the app package directly for finding the graph.
        let app_pkg = base_dir_pkg_info.app_pkg_info.as_ref().unwrap();

        // Get extension package information directly, if available.
        let extensions_slice = base_dir_pkg_info.get_extensions();

        let extensions =
            match get_extension_nodes_in_graph(&request_payload.graph, app_pkg)
            {
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

        let pkg_info = get_pkg_info_for_extension(extension, extensions_slice);
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

                let results = match get_compatible_cmd_extension(
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
                };

                results
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

                let results = match get_compatible_data_like_msg_extension(
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
                };

                results
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
    } else {
        let error_response = ErrorResponse {
            status: Status::Fail,
            message: format!(
                "Package information not found for {}",
                request_payload.base_dir
            ),
            error: None,
        };
        Ok(HttpResponse::NotFound().json(error_response))
    }
}
