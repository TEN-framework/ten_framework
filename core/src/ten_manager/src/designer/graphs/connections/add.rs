//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::sync::{Arc, RwLock};

use actix_web::{web, HttpResponse, Responder};
use anyhow::Result;
use serde::{Deserialize, Serialize};

use ten_rust::{
    graph::{
        connection::{GraphConnection, GraphDestination, GraphMessageFlow},
        msg_conversion::MsgAndResultConversion,
    },
    pkg_info::{create_uri_to_pkg_info_map, message::MsgType},
};
use uuid::Uuid;

use crate::{
    designer::{
        response::{ApiResponse, ErrorResponse, Status},
        DesignerState,
    },
    graph::connections::add::graph_add_connection,
    pkg_info::belonging_pkg_info_find_by_graph_info_mut,
};

use crate::graph::{
    graphs_cache_find_by_id_mut, update_graph_connections_all_fields,
};

use super::msg_conversion::validate::{
    validate_msg_conversion_schema, MsgConversionValidateInfo,
};

#[derive(Serialize, Deserialize)]
pub struct AddGraphConnectionRequestPayload {
    pub graph_id: Uuid,

    pub src_app: Option<String>,
    pub src_extension: String,
    pub msg_type: MsgType,
    pub msg_name: String,
    pub dest_app: Option<String>,
    pub dest_extension: String,

    pub msg_conversion: Option<MsgAndResultConversion>,
}

#[derive(Serialize, Deserialize)]
pub struct AddGraphConnectionResponsePayload {
    pub success: bool,
}

/// Create a new GraphConnection from request params.
fn create_graph_connection(
    request_payload: &AddGraphConnectionRequestPayload,
) -> GraphConnection {
    // Create the destination.
    let destination = GraphDestination {
        app: request_payload.dest_app.clone(),
        extension: request_payload.dest_extension.clone(),
        msg_conversion: request_payload.msg_conversion.clone(),
    };

    // Create the message flow.
    let message_flow = GraphMessageFlow {
        name: request_payload.msg_name.clone(),
        dest: vec![destination],
    };

    // Create the connection object.
    let mut connection = GraphConnection {
        app: request_payload.src_app.clone(),
        extension: request_payload.src_extension.clone(),
        cmd: None,
        data: None,
        audio_frame: None,
        video_frame: None,
    };

    // Add the message flow to the appropriate field.
    match request_payload.msg_type {
        MsgType::Cmd => {
            connection.cmd = Some(vec![message_flow]);
        }
        MsgType::Data => {
            connection.data = Some(vec![message_flow]);
        }
        MsgType::AudioFrame => {
            connection.audio_frame = Some(vec![message_flow]);
        }
        MsgType::VideoFrame => {
            connection.video_frame = Some(vec![message_flow]);
        }
    }

    connection
}

/// Update the property.json file with the new connection.
fn update_property_file(
    base_dir: &str,
    property: &mut ten_rust::pkg_info::property::Property,
    graph_name: &str,
    connection: &GraphConnection,
) -> Result<()> {
    // Update only with the new connection.
    let connections_to_add = vec![connection.clone()];

    // Update the property_all_fields map and write to property.json.
    update_graph_connections_all_fields(
        base_dir,
        &mut property.all_fields,
        graph_name,
        Some(&connections_to_add),
        None,
        None,
    )
}

pub async fn add_graph_connection_endpoint(
    request_payload: web::Json<AddGraphConnectionRequestPayload>,
    state: web::Data<Arc<RwLock<DesignerState>>>,
) -> Result<impl Responder, actix_web::Error> {
    let mut state_write = state.write().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!(
            "Failed to acquire write lock: {}",
            e
        ))
    })?;

    let DesignerState {
        pkgs_cache,
        graphs_cache,
        ..
    } = &mut *state_write;

    // Create a hash map from app URIs to PkgsInfoInApp.
    let uri_to_pkg_info = match create_uri_to_pkg_info_map(pkgs_cache) {
        Ok(map) => map,
        Err(error_message) => {
            let error_response = ErrorResponse {
                status: Status::Fail,
                message: error_message,
                error: None,
            };
            return Ok(HttpResponse::BadRequest().json(error_response));
        }
    };

    // Get the specified graph from graphs_cache.
    let graph_info = match graphs_cache_find_by_id_mut(
        graphs_cache,
        &request_payload.graph_id,
    ) {
        Some(graph_info) => graph_info,
        None => {
            let error_response = ErrorResponse {
                status: Status::Fail,
                message: "Graph not found".to_string(),
                error: None,
            };
            return Ok(HttpResponse::NotFound().json(error_response));
        }
    };

    validate_msg_conversion_schema(
        graph_info,
        &MsgConversionValidateInfo {
            src_app: &request_payload.src_app,
            src_extension: &request_payload.src_extension,
            msg_type: &request_payload.msg_type,
            msg_name: &request_payload.msg_name,
            dest_app: &request_payload.dest_app,
            dest_extension: &request_payload.dest_extension,
            msg_conversion: &request_payload.msg_conversion,
        },
        &uri_to_pkg_info,
        pkgs_cache,
    )
    .map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!(
            "Failed to check message conversion schema: {}",
            e
        ))
    })?;

    // Add the connection using the converted PkgsInfoInApp map.
    match graph_add_connection(
        &mut graph_info.graph,
        request_payload.src_app.clone(),
        request_payload.src_extension.clone(),
        request_payload.msg_type.clone(),
        request_payload.msg_name.clone(),
        request_payload.dest_app.clone(),
        request_payload.dest_extension.clone(),
        &uri_to_pkg_info,
        request_payload.msg_conversion.clone(),
    ) {
        Ok(_) => {
            if let Ok(Some(pkg_info)) =
                belonging_pkg_info_find_by_graph_info_mut(
                    pkgs_cache, graph_info,
                )
            {
                // Update property.json file with the updated graph.
                if let Some(property) = &mut pkg_info.property {
                    // Create a new connection object.
                    let connection = create_graph_connection(&request_payload);

                    // Update the property.json file.
                    if let Err(e) = update_property_file(
                        &pkg_info.url,
                        property,
                        graph_info.name.as_ref().unwrap(),
                        &connection,
                    ) {
                        eprintln!(
                            "Warning: Failed to update property.json file: {}",
                            e
                        );
                    }
                }
            }

            let response = ApiResponse {
                status: Status::Ok,
                data: AddGraphConnectionResponsePayload { success: true },
                meta: None,
            };
            Ok(HttpResponse::Ok().json(response))
        }
        Err(err) => {
            let error_response = ErrorResponse {
                status: Status::Fail,
                message: format!("Failed to add connection: {}", err),
                error: None,
            };
            Ok(HttpResponse::BadRequest().json(error_response))
        }
    }
}
