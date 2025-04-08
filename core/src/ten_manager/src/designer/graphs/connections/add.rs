//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use actix_web::{web, HttpResponse, Responder};
use anyhow::Result;
use serde::{Deserialize, Serialize};

use ten_rust::{
    base_dir_pkg_info::PkgsInfoInApp,
    graph::connection::{GraphConnection, GraphDestination, GraphMessageFlow},
    graph::msg_conversion::MsgAndResultConversion,
    pkg_info::message::MsgType,
};

use crate::designer::graphs::util::find_predefined_graph;
use crate::designer::{
    graphs::util::find_app_package_from_base_dir,
    response::{ApiResponse, ErrorResponse, Status},
    DesignerState,
};

use crate::graph::update_graph_connections_all_fields;

/// A struct that contains both PkgsInfoInApp and its base_dir.
#[derive(Clone)]
struct PkgInfoWithBaseDir {
    pkg_info: PkgsInfoInApp,
    base_dir: String,
}

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
    let mut uri_to_pkg_info: HashMap<Option<String>, PkgInfoWithBaseDir> =
        HashMap::new();

    // Process all available apps to map URIs to PkgsInfoInApp.
    for (base_dir, base_dir_pkg_info) in state_write.pkgs_cache.iter() {
        if let Some(app_pkg) = &base_dir_pkg_info.app_pkg_info {
            if let Some(property) = &app_pkg.property {
                if let Some(ten) = &property._ten {
                    // Map the URI to the PkgsInfoInApp, using None if URI is
                    // None.
                    let key = ten.uri.clone();

                    // Check if the key already exists.
                    if let Some(existing) = uri_to_pkg_info.get(&key) {
                        let error_message = if key.is_none() {
                            format!(
                                "Found two apps with unspecified URI in both '{}' and '{}'",
                                existing.base_dir,
                                base_dir
                            )
                        } else {
                            format!(
                                "Duplicate app uri '{}' found in both '{}' and '{}'",
                                key.as_ref().unwrap(),
                                existing.base_dir,
                                base_dir
                            )
                        };

                        let error_response = ErrorResponse {
                            status: Status::Fail,
                            message: error_message,
                            error: None,
                        };
                        return Ok(
                            HttpResponse::BadRequest().json(error_response)
                        );
                    }

                    uri_to_pkg_info.insert(
                        key,
                        PkgInfoWithBaseDir {
                            pkg_info: base_dir_pkg_info.clone(),
                            base_dir: base_dir.clone(),
                        },
                    );
                }
            }
        }
    }

    // Get the packages for this base_dir.
    if let Some(base_dir_pkg_info) =
        state_write.pkgs_cache.get_mut(&request_payload.base_dir)
    {
        // Find the app package.
        if let Some(app_pkg) = find_app_package_from_base_dir(base_dir_pkg_info)
        {
            // Get the specified graph from predefined_graphs.
            if let Some(predefined_graph) =
                find_predefined_graph(app_pkg, &request_payload.graph_name)
            {
                let mut graph = predefined_graph.graph.clone();

                // Add the connection using the converted PkgsInfoInApp map.
                match graph.add_connection(
                    request_payload.src_app.clone(),
                    request_payload.src_extension.clone(),
                    request_payload.msg_type.clone(),
                    request_payload.msg_name.clone(),
                    request_payload.dest_app.clone(),
                    request_payload.dest_extension.clone(),
                    &uri_to_pkg_info
                        .iter()
                        .map(|(k, v)| {
                            (k.clone().unwrap_or_default(), v.pkg_info.clone())
                        })
                        .collect(),
                    request_payload.msg_conversion.clone(),
                ) {
                    Ok(_) => {
                        // Update the predefined_graph in the app_pkg.
                        let mut new_graph = predefined_graph.clone();
                        new_graph.graph = graph;
                        app_pkg.update_predefined_graph(&new_graph);

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
