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
    graph::{node::GraphNode, Graph},
    pkg_info::{
        pkg_type::PkgType, pkg_type_and_name::PkgTypeAndName,
        property::predefined_graph::PredefinedGraph,
    },
};

use crate::designer::{
    graphs::util::{find_app_package_from_base_dir, find_predefined_graph},
    response::{ApiResponse, ErrorResponse, Status},
    DesignerState,
};

#[derive(Serialize, Deserialize)]
pub struct AddGraphNodeRequestPayload {
    pub base_dir: String,
    pub graph_name: String,
    pub node_name: String,
    pub addon_name: String,
    pub app_uri: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct AddGraphNodeResponsePayload {
    pub success: bool,
}

/// Adds a new extension node to a graph.
fn add_extension_node_to_graph(
    predefined_graph: &PredefinedGraph,
    node_name: &str,
    addon_name: &str,
    app_uri: &Option<String>,
) -> Result<Graph, String> {
    let mut graph = predefined_graph.graph.clone();

    // Add the extension node.
    match graph.add_extension_node(
        node_name.to_string(),
        addon_name.to_string(),
        app_uri.clone(),
        None,
        None,
    ) {
        Ok(_) => Ok(graph),
        Err(e) => Err(e.to_string()),
    }
}

/// Creates a new GraphNode for an extension.
fn create_extension_node(
    node_name: &str,
    addon_name: &str,
    app_uri: &Option<String>,
) -> GraphNode {
    GraphNode {
        type_and_name: PkgTypeAndName {
            pkg_type: PkgType::Extension,
            name: node_name.to_string(),
        },
        addon: addon_name.to_string(),
        extension_group: None,
        app: app_uri.clone(),
        property: None,
    }
}

/// Updates the property.json file with the new graph node.
fn update_node_property_file(
    base_dir: &str,
    property: &mut ten_rust::pkg_info::property::Property,
    graph_name: &str,
    node: &GraphNode,
) -> Result<(), Box<dyn std::error::Error>> {
    let nodes_to_add = vec![node.clone()];
    crate::graph::update_graph_node_all_fields(
        base_dir,
        &mut property.all_fields,
        graph_name,
        Some(&nodes_to_add),
        None,
    )?;
    Ok(())
}

pub async fn add_graph_node_endpoint(
    request_payload: web::Json<AddGraphNodeRequestPayload>,
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
            if let Some(predefined_graph) =
                find_predefined_graph(app_pkg, &request_payload.graph_name)
            {
                // Add the node to the graph
                match add_extension_node_to_graph(
                    predefined_graph,
                    &request_payload.node_name,
                    &request_payload.addon_name,
                    &request_payload.app_uri,
                ) {
                    Ok(graph) => {
                        // Update the predefined_graph in the app_pkg.
                        let mut new_graph = predefined_graph.clone();
                        new_graph.graph = graph;
                        app_pkg.update_predefined_graph(&new_graph);

                        // Create the graph node.
                        let new_node = create_extension_node(
                            &request_payload.node_name,
                            &request_payload.addon_name,
                            &request_payload.app_uri,
                        );

                        // Update property.json file with the new graph node.
                        if let Some(property) = &mut app_pkg.property {
                            // Write the updated property_all_fields map to
                            // property.json.
                            if let Err(e) = update_node_property_file(
                                &request_payload.base_dir,
                                property,
                                &request_payload.graph_name,
                                &new_node,
                            ) {
                                eprintln!("Warning: Failed to update property.json file: {}", e);
                            }
                        }

                        let response = ApiResponse {
                            status: Status::Ok,
                            data: AddGraphNodeResponsePayload { success: true },
                            meta: None,
                        };
                        Ok(HttpResponse::Ok().json(response))
                    }
                    Err(err) => {
                        let error_response = ErrorResponse {
                            status: Status::Fail,
                            message: format!("Failed to add node: {}", err),
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
