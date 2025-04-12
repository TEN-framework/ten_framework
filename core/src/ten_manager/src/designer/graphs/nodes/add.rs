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
    graph::{graph_info::GraphInfo, node::GraphNode},
    pkg_info::{pkg_type::PkgType, pkg_type_and_name::PkgTypeAndName},
};
use uuid::Uuid;

use crate::{
    designer::{
        graphs::{
            nodes::validate::{validate_node_request, GraphNodeValidatable},
            util::find_app_package_from_base_dir,
        },
        response::{ApiResponse, ErrorResponse, Status},
        DesignerState,
    },
    graph::graphs_cache_find_by_id_mut,
};

#[derive(Serialize, Deserialize)]
pub struct AddGraphNodeRequestPayload {
    pub graph_app_base_dir: String,
    pub graph_id: Uuid,

    pub addon_app_base_dir: Option<String>,
    pub node_name: String,
    pub addon_name: String,
    pub extension_group_name: Option<String>,
    pub app_uri: Option<String>,
    pub property: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize)]
pub struct AddGraphNodeResponsePayload {
    pub success: bool,
}

/// Adds a new extension node to a graph.
fn add_extension_node_to_graph(
    graph_info: &mut GraphInfo,
    node_name: &str,
    addon_name: &str,
    app_uri: &Option<String>,
) -> Result<(), String> {
    // Add the extension node.
    match graph_info.graph.add_extension_node(
        node_name.to_string(),
        addon_name.to_string(),
        app_uri.clone(),
        None,
        None,
    ) {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

/// Creates a new GraphNode for an extension.
fn create_extension_node(
    node_name: &str,
    addon_name: &str,
    extension_group_name: &Option<String>,
    app_uri: &Option<String>,
    property: &Option<serde_json::Value>,
) -> GraphNode {
    GraphNode {
        type_and_name: PkgTypeAndName {
            pkg_type: PkgType::Extension,
            name: node_name.to_string(),
        },
        addon: addon_name.to_string(),
        extension_group: extension_group_name.clone(),
        app: app_uri.clone(),
        property: property.clone(),
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
        None,
    )?;
    Ok(())
}

impl GraphNodeValidatable for AddGraphNodeRequestPayload {
    fn get_addon_app_base_dir(&self) -> &Option<String> {
        &self.addon_app_base_dir
    }

    fn get_addon_name(&self) -> &str {
        &self.addon_name
    }

    fn get_app_uri(&self) -> &Option<String> {
        &self.app_uri
    }

    fn get_property(&self) -> &Option<serde_json::Value> {
        &self.property
    }
}

/// Validates the AddGraphNodeRequestPayload based on the addon extension
/// schema.
fn validate_add_graph_node_request(
    request_payload: &AddGraphNodeRequestPayload,
    state: &mut DesignerState,
) -> Result<(), String> {
    validate_node_request(request_payload, state)
}

pub async fn add_graph_node_endpoint(
    request_payload: web::Json<AddGraphNodeRequestPayload>,
    state: web::Data<Arc<RwLock<DesignerState>>>,
) -> Result<impl Responder, actix_web::Error> {
    // Get a write lock on the state since we need to modify the graph.
    let mut state_write = state.write().unwrap();

    // Validate the request payload before proceeding.
    if let Err(validation_error) =
        validate_add_graph_node_request(&request_payload, &mut state_write)
    {
        let error_response = ErrorResponse {
            status: Status::Fail,
            message: format!("Validation failed: {}", validation_error),
            error: None,
        };
        return Ok(HttpResponse::BadRequest().json(error_response));
    }

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

    // Get the packages for this base_dir.
    if let Some(base_dir_pkg_info) =
        pkgs_cache.get_mut(&request_payload.graph_app_base_dir)
    {
        // Find the app package.
        if let Some(app_pkg) = find_app_package_from_base_dir(base_dir_pkg_info)
        {
            // Add the node to the graph.
            match add_extension_node_to_graph(
                graph_info,
                &request_payload.node_name,
                &request_payload.addon_name,
                &request_payload.app_uri,
            ) {
                Ok(_) => {
                    // Create the graph node.
                    let new_node = create_extension_node(
                        &request_payload.node_name,
                        &request_payload.addon_name,
                        &request_payload.extension_group_name,
                        &request_payload.app_uri,
                        &request_payload.property,
                    );

                    // Update property.json file with the new graph node.
                    if let Some(property) = &mut app_pkg.property {
                        // Write the updated property_all_fields map to
                        // property.json.
                        if let Err(e) = update_node_property_file(
                            &request_payload.graph_app_base_dir,
                            property,
                            graph_info.name.as_ref().unwrap(),
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
