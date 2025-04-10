//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::sync::{Arc, RwLock};

use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

use ten_rust::{
    graph::connection::GraphConnection,
    pkg_info::{
        message::MsgType, pkg_type::PkgType,
        predefined_graphs::pkg_predefined_graphs_find_mut,
    },
};

use crate::{
    designer::{
        graphs::util::find_app_package_from_base_dir,
        response::{ApiResponse, ErrorResponse, Status},
        DesignerState,
    },
    graph::update_graph_connections_all_fields,
};

#[derive(Serialize, Deserialize)]
pub struct DeleteGraphConnectionRequestPayload {
    pub base_dir: String,
    pub graph_name: String,
    pub src_app: Option<String>,
    pub src_extension: String,
    pub msg_type: MsgType,
    pub msg_name: String,
    pub dest_app: Option<String>,
    pub dest_extension: String,
}

#[derive(Serialize, Deserialize)]
pub struct DeleteGraphConnectionResponsePayload {
    pub success: bool,
}

pub async fn delete_graph_connection_endpoint(
    request_payload: web::Json<DeleteGraphConnectionRequestPayload>,
    state: web::Data<Arc<RwLock<DesignerState>>>,
) -> Result<impl Responder, actix_web::Error> {
    // Get a write lock on the state since we need to modify the graph.
    let mut state_write = state.write().unwrap();

    let DesignerState {
        pkgs_cache,
        graphs_cache,
        ..
    } = &mut *state_write;

    // Get the packages for this base_dir.
    if let Some(base_dir_pkg_info) =
        pkgs_cache.get_mut(&request_payload.base_dir)
    {
        // Find the app package.
        if let Some(app_pkg) = find_app_package_from_base_dir(base_dir_pkg_info)
        {
            // Get the specified graph from graphs_cache.
            if let Some(graph_info) =
                pkg_predefined_graphs_find_mut(graphs_cache, |g| {
                    g.name == request_payload.graph_name
                        && (g.app_base_dir.is_some()
                            && g.app_base_dir.as_ref().unwrap()
                                == &request_payload.base_dir)
                        && (g.belonging_pkg_type.is_some()
                            && g.belonging_pkg_type.unwrap() == PkgType::App)
                })
            {
                // Delete the connection.
                match graph_info.graph.delete_connection(
                    request_payload.src_app.clone(),
                    request_payload.src_extension.clone(),
                    request_payload.msg_type.clone(),
                    request_payload.msg_name.clone(),
                    request_payload.dest_app.clone(),
                    request_payload.dest_extension.clone(),
                ) {
                    Ok(_) => {
                        // Update property.json file to remove the connection.
                        if let Some(property) = &mut app_pkg.property {
                            // Create a GraphConnection representing what we
                            // want to remove.
                            let mut connection = GraphConnection {
                                app: request_payload.src_app.clone(),
                                extension: request_payload
                                    .src_extension
                                    .clone(),
                                cmd: None,
                                data: None,
                                audio_frame: None,
                                video_frame: None,
                            };

                            // Set the appropriate message type field.
                            match request_payload.msg_type {
                                MsgType::Cmd => {
                                    connection.cmd = Some(vec![]);
                                }
                                MsgType::Data => {
                                    connection.data = Some(vec![]);
                                }
                                MsgType::AudioFrame => {
                                    connection.audio_frame = Some(vec![]);
                                }
                                MsgType::VideoFrame => {
                                    connection.video_frame = Some(vec![]);
                                }
                            }

                            let connections_to_remove = vec![connection];

                            // Write the updated property_all_fields map to
                            // property.json.
                            if let Err(e) = update_graph_connections_all_fields(
                                &request_payload.base_dir,
                                &mut property.all_fields,
                                &request_payload.graph_name,
                                None,
                                Some(&connections_to_remove),
                                None,
                            ) {
                                eprintln!("Warning: Failed to update property.json file: {}", e);
                            }
                        }

                        let response = ApiResponse {
                            status: Status::Ok,
                            data: DeleteGraphConnectionResponsePayload {
                                success: true,
                            },
                            meta: None,
                        };
                        Ok(HttpResponse::Ok().json(response))
                    }
                    Err(err) => {
                        let error_response = ErrorResponse {
                            status: Status::Fail,
                            message: format!(
                                "Failed to delete connection: {}",
                                err
                            ),
                            error: None,
                        };
                        Ok(HttpResponse::BadRequest().json(error_response))
                    }
                }
            } else {
                let error_response = ErrorResponse {
                    status: Status::Fail,
                    message: format!(
                        "Graph '{}' not found",
                        request_payload.graph_name
                    ),
                    error: None,
                };
                Ok(HttpResponse::NotFound().json(error_response))
            }
        } else {
            let error_response = ErrorResponse {
                status: Status::Fail,
                message: "App package not found".to_string(),
                error: None,
            };
            Ok(HttpResponse::NotFound().json(error_response))
        }
    } else {
        let error_response = ErrorResponse {
            status: Status::Fail,
            message: "Base directory not found".to_string(),
            error: None,
        };
        Ok(HttpResponse::NotFound().json(error_response))
    }
}
