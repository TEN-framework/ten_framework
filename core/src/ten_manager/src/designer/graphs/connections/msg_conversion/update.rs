//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use uuid::Uuid;

use actix_web::{web, HttpResponse, Responder};
use ten_rust::{
    graph::{
        connection::{GraphConnection, GraphDestination, GraphMessageFlow},
        msg_conversion::MsgAndResultConversion,
    },
    pkg_info::message::MsgType,
};

use crate::{
    designer::{
        response::{ApiResponse, ErrorResponse, Status},
        DesignerState,
    },
    graph::{
        graphs_cache_find_by_id_mut,
        msg_conversion::msg_conversion_get_final_target_schema,
        nodes::extension_graph_node_find_by_loc,
        update_graph_connections_all_fields,
    },
    pkg_info::{create_uri_to_pkg_info_map, pkg_info_find_by_graph_info_mut},
};

#[derive(Serialize, Deserialize)]
pub struct UpdateGraphConnectionMsgConversionRequestPayload {
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
pub struct UpdateGraphConnectionMsgConversionResponsePayload {
    pub success: bool,
}

pub async fn update_graph_connection_msg_conversion_endpoint(
    request_payload: web::Json<
        UpdateGraphConnectionMsgConversionRequestPayload,
    >,
    state: web::Data<Arc<RwLock<DesignerState>>>,
) -> Result<impl Responder, actix_web::Error> {
    // Get a write lock on the state since we may need to modify the graph.
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

    // Update the GraphInfo structure.
    // First check if connections exist in the graph.
    if let Some(connections) = &mut graph_info.graph.connections {
        // Try to find the matching connection based on app and extension.
        for connection in connections.iter_mut() {
            if connection.app == request_payload.src_app
                && connection.extension == request_payload.src_extension
            {
                // Find the correct message flow vector based on msg_type.
                let msg_flow_vec = match request_payload.msg_type {
                    MsgType::Cmd => &mut connection.cmd,
                    MsgType::Data => &mut connection.data,
                    MsgType::AudioFrame => &mut connection.audio_frame,
                    MsgType::VideoFrame => &mut connection.video_frame,
                };

                // If we found the message flow vector, find the specific
                // message flow by name.
                if let Some(msg_flows) = msg_flow_vec {
                    for msg_flow in msg_flows.iter_mut() {
                        if msg_flow.name == request_payload.msg_name {
                            // Find the matching destination
                            for dest in msg_flow.dest.iter_mut() {
                                if dest.app == request_payload.dest_app
                                    && dest.extension
                                        == request_payload.dest_extension
                                {
                                    // Update the msg_conversion field.
                                    dest.msg_conversion =
                                        request_payload.msg_conversion.clone();
                                    break;
                                }
                            }
                            break;
                        }
                    }
                }
                break;
            }
        }
    }

    if let Some(msg_conversion) = &request_payload.msg_conversion {
        let src_graph_node = extension_graph_node_find_by_loc(
            graph_info,
            request_payload.src_app.as_ref(),
            &request_payload.src_extension,
        )
        .unwrap();

        let dest_graph_node = extension_graph_node_find_by_loc(
            graph_info,
            request_payload.dest_app.as_ref(),
            &request_payload.dest_extension,
        )
        .unwrap();

        let _ = msg_conversion_get_final_target_schema(
            &uri_to_pkg_info,
            graph_info.app_base_dir.as_ref(),
            pkgs_cache,
            &request_payload.msg_type,
            &src_graph_node.app,
            &src_graph_node.addon,
            &request_payload.msg_name,
            &dest_graph_node.app,
            &dest_graph_node.addon,
            &request_payload.msg_name,
            msg_conversion,
        );
    }

    if let Ok(Some(pkg_info)) =
        pkg_info_find_by_graph_info_mut(pkgs_cache, graph_info)
    {
        // Check if the property exists.
        if let Some(property) = &mut pkg_info.property {
            // Create a GraphConnection with the message conversion to
            // update.
            let mut connection = GraphConnection {
                app: request_payload.src_app.clone(),
                extension: request_payload.src_extension.clone(),
                cmd: None,
                data: None,
                audio_frame: None,
                video_frame: None,
            };

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

            // Set the appropriate message type field.
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

            // Update the connection with the new message conversion.
            let connections_to_modify = vec![connection];

            // Update the property.json file.
            if let Err(e) = update_graph_connections_all_fields(
                &pkg_info.url,
                &mut property.all_fields,
                graph_info.name.as_ref().unwrap(),
                None,
                None,
                Some(&connections_to_modify),
            ) {
                // Return error if failed to update.
                let error_response = ErrorResponse {
                    status: Status::Fail,
                    message: format!(
                        "Failed to update message conversion: {}",
                        e
                    ),
                    error: None,
                };
                return Ok(
                    HttpResponse::InternalServerError().json(error_response)
                );
            }
        }
    }

    let response = ApiResponse {
        status: Status::Ok,
        data: UpdateGraphConnectionMsgConversionResponsePayload {
            success: true,
        },
        meta: None,
    };
    Ok(HttpResponse::Ok().json(response))
}
