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
use uuid::Uuid;

use ten_rust::{
    graph::{graph_info::GraphInfo, node::GraphNode, Graph},
    pkg_info::{
        create_uri_to_pkg_info_map, get_pkg_info_for_extension_addon,
        pkg_type::PkgType, pkg_type_and_name::PkgTypeAndName, PkgInfo,
    },
};

use crate::{
    designer::{
        graphs::nodes::validate::{
            validate_node_request, GraphNodeValidatable,
        },
        response::{ApiResponse, ErrorResponse, Status},
        DesignerState,
    },
    graph::{graphs_cache_find_by_id_mut, update_graph_node_all_fields},
    pkg_info::belonging_pkg_info_find_by_graph_info_mut,
};

#[derive(Serialize, Deserialize)]
pub struct AddGraphNodeRequestPayload {
    pub graph_id: Uuid,

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

pub fn graph_add_extension_node(
    graph: &mut Graph,
    pkg_name: String,
    addon: String,
    app: Option<String>,
    extension_group: Option<String>,
    property: Option<serde_json::Value>,
) -> Result<()> {
    // Store the original state in case validation fails.
    let original_graph = graph.clone();

    // Create new GraphNode.
    let node = GraphNode {
        type_and_name: PkgTypeAndName {
            pkg_type: PkgType::Extension,
            name: pkg_name,
        },
        addon,
        extension_group,
        app,
        property,
    };

    // Add the node to the graph.
    graph.nodes.push(node);

    // Validate the graph.
    match graph.validate_and_complete() {
        Ok(_) => Ok(()),
        Err(e) => {
            // Restore the original graph if validation fails.
            *graph = original_graph;
            Err(e)
        }
    }
}

/// Adds a new extension node to a graph.
fn add_extension_node_to_graph(
    graph_info: &mut GraphInfo,
    node_name: &str,
    addon_name: &str,
    app_uri: &Option<String>,
) -> Result<(), String> {
    // Add the extension node.
    match graph_add_extension_node(
        &mut graph_info.graph,
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

impl GraphNodeValidatable for AddGraphNodeRequestPayload {
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
    extension_pkg_info: &PkgInfo,
) -> Result<(), String> {
    validate_node_request(request_payload, extension_pkg_info)
}

pub async fn add_graph_node_endpoint(
    request_payload: web::Json<AddGraphNodeRequestPayload>,
    state: web::Data<Arc<RwLock<DesignerState>>>,
) -> Result<impl Responder, actix_web::Error> {
    // Get a write lock on the state since we need to modify the graph.
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

    if let Some(extension_pkg_info) = get_pkg_info_for_extension_addon(
        &request_payload.app_uri,
        &request_payload.addon_name,
        &uri_to_pkg_info,
        graph_info.app_base_dir.as_ref(),
        pkgs_cache,
    ) {
        // Validate the request payload before proceeding.
        if let Err(validation_error) = validate_add_graph_node_request(
            &request_payload,
            extension_pkg_info,
        ) {
            let error_response = ErrorResponse {
                status: Status::Fail,
                message: format!("Validation failed: {}", validation_error),
                error: None,
            };
            return Ok(HttpResponse::BadRequest().json(error_response));
        }
    };

    // Add the node to the graph.
    match add_extension_node_to_graph(
        graph_info,
        &request_payload.node_name,
        &request_payload.addon_name,
        &request_payload.app_uri,
    ) {
        Ok(_) => {
            if let Ok(Some(pkg_info)) =
                belonging_pkg_info_find_by_graph_info_mut(
                    pkgs_cache, graph_info,
                )
            {
                // Create the graph node.
                let new_node = GraphNode {
                    type_and_name: PkgTypeAndName {
                        pkg_type: PkgType::Extension,
                        name: request_payload.node_name.to_string(),
                    },
                    addon: request_payload.addon_name.to_string(),
                    extension_group: request_payload
                        .extension_group_name
                        .clone(),
                    app: request_payload.app_uri.clone(),
                    property: request_payload.property.clone(),
                };

                // Update property.json file with the new graph node.
                if let Some(property) = &mut pkg_info.property {
                    // Write the updated property_all_fields map to
                    // property.json.
                    let nodes_to_add = vec![new_node.clone()];

                    if let Err(e) = update_graph_node_all_fields(
                        &pkg_info.url,
                        &mut property.all_fields,
                        graph_info.name.as_ref().unwrap(),
                        Some(&nodes_to_add),
                        None,
                        None,
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
}
