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

use ten_rust::pkg_info::pkg_type::PkgType;
use ten_rust::pkg_info::predefined_graphs::graphs_cache_find_mut;
use ten_rust::{
    graph::connection::{GraphConnection, GraphDestination, GraphMessageFlow},
    graph::msg_conversion::MsgAndResultConversion,
    pkg_info::message::MsgType,
};

use crate::designer::{
    graphs::util::find_app_package_from_base_dir,
    response::{ApiResponse, ErrorResponse, Status},
    DesignerState,
};

use crate::graph::update_graph_connections_all_fields;
use crate::pkg_info::create_uri_to_pkg_info_map;

#[derive(Serialize, Deserialize)]
pub struct AddGraphConnectionRequestPayload {
    pub base_dir: String,
    pub graph_name: String,

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
    let mut state_write = state.write().unwrap();

    // Create a hash map from app URIs to PkgsInfoInApp for use with
    // add_connection.
    let uri_to_pkg_info =
        match create_uri_to_pkg_info_map(&state_write.pkgs_cache) {
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
            if let Some(graph_info) = graphs_cache_find_mut(graphs_cache, |g| {
                g.name
                    .as_ref()
                    .map(|name| name == &request_payload.graph_name)
                    .unwrap_or(false)
                    && (g.app_base_dir.is_some()
                        && g.app_base_dir.as_ref().unwrap()
                            == &request_payload.base_dir)
                    && (g.belonging_pkg_type.is_some()
                        && g.belonging_pkg_type.unwrap() == PkgType::App)
            }) {
                // Add the connection using the converted PkgsInfoInApp map.
                match graph_info.graph.add_connection(
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
                        // Update property.json file with the updated graph.
                        if let Some(property) = &mut app_pkg.property {
                            // Create a new connection object.
                            let connection =
                                create_graph_connection(&request_payload);

                            // Update the property.json file.
                            if let Err(e) = update_property_file(
                                &request_payload.base_dir,
                                property,
                                &request_payload.graph_name,
                                &connection,
                            ) {
                                eprintln!("Warning: Failed to update property.json file: {}", e);
                            }
                        }

                        let response = ApiResponse {
                            status: Status::Ok,
                            data: AddGraphConnectionResponsePayload {
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
                                "Failed to add connection: {}",
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
