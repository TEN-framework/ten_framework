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
    graph::node::GraphNode,
    pkg_info::{
        pkg_type::PkgType, pkg_type_and_name::PkgTypeAndName,
        predefined_graphs::pkg_predefined_graphs_find,
    },
};

use crate::{
    designer::{
        graphs::util::find_app_package_from_base_dir,
        response::{ApiResponse, ErrorResponse, Status},
        DesignerState,
    },
    graph::update_graph_node_all_fields,
};

#[derive(Serialize, Deserialize)]
pub struct DeleteGraphNodeRequestPayload {
    pub base_dir: String,
    pub graph_name: String,
    pub node_name: String,
    pub addon_name: String,
    pub extension_group_name: Option<String>,
    pub app_uri: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct DeleteGraphNodeResponsePayload {
    pub success: bool,
}

pub async fn delete_graph_node_endpoint(
    request_payload: web::Json<DeleteGraphNodeRequestPayload>,
    state: web::Data<Arc<RwLock<DesignerState>>>,
) -> Result<impl Responder, actix_web::Error> {
    // Get a write lock on the state since we need to modify the graph.
    let mut state_write = state.write().unwrap();

    // Get the packages for this base_dir.
    if let Some(base_dir_pkg_info) =
        state_write.pkgs_cache.get_mut(&request_payload.base_dir)
    {
        // Find the app package.
        if let Some(app_pkg) = find_app_package_from_base_dir(base_dir_pkg_info)
        {
            // Get the specified graph from predefined_graphs.
            if let Some(predefined_graph) = pkg_predefined_graphs_find(
                app_pkg.get_predefined_graphs(),
                |g| g.name == request_payload.graph_name,
            ) {
                let mut graph = predefined_graph.graph.clone();

                // Delete the extension node.
                match graph.delete_extension_node(
                    request_payload.node_name.clone(),
                    request_payload.addon_name.clone(),
                    request_payload.app_uri.clone(),
                    request_payload.extension_group_name.clone(),
                ) {
                    Ok(_) => {
                        // Update the predefined_graph in the app_pkg.
                        let mut new_graph = predefined_graph.clone();
                        new_graph.graph = graph;
                        app_pkg.update_predefined_graph(&new_graph);

                        // Update property.json file to remove the graph node.
                        if let Some(property) = &mut app_pkg.property {
                            // Create the GraphNode we want to remove.
                            let node_to_remove = GraphNode {
                                type_and_name: PkgTypeAndName {
                                    pkg_type: PkgType::Extension,
                                    name: request_payload.node_name.clone(),
                                },
                                addon: request_payload.addon_name.clone(),
                                extension_group: request_payload
                                    .extension_group_name
                                    .clone(),
                                app: request_payload.app_uri.clone(),
                                property: None,
                            };
                            let nodes_to_remove = vec![node_to_remove];

                            // Write the updated property_all_fields map to
                            // property.json.
                            if let Err(e) = update_graph_node_all_fields(
                                &request_payload.base_dir,
                                &mut property.all_fields,
                                &request_payload.graph_name,
                                None,
                                Some(&nodes_to_remove),
                                None,
                            ) {
                                eprintln!("Warning: Failed to update property.json file: {}", e);
                            }
                        }

                        let response = ApiResponse {
                            status: Status::Ok,
                            data: DeleteGraphNodeResponsePayload {
                                success: true,
                            },
                            meta: None,
                        };
                        Ok(HttpResponse::Ok().json(response))
                    }
                    Err(err) => {
                        let error_response = ErrorResponse {
                            status: Status::Fail,
                            message: format!("Failed to delete node: {}", err),
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
