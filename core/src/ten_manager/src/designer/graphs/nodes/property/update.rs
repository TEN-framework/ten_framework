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
    graph::node::GraphNode,
    pkg_info::{pkg_type::PkgType, pkg_type_and_name::PkgTypeAndName},
};

use crate::{
    designer::{
        graphs::nodes::validate::{
            validate_node_request, GraphNodeValidatable,
        },
        response::{ApiResponse, ErrorResponse, Status},
        DesignerState,
    },
    graph::graphs_cache_find_by_id_mut,
    pkg_info::pkg_info_find_by_graph_info_mut,
};

#[derive(Serialize, Deserialize)]
pub struct UpdateGraphNodePropertyRequestPayload {
    pub graph_id: Uuid,

    pub addon_app_base_dir: Option<String>,
    pub node_name: String,
    pub addon_name: String,
    pub extension_group_name: Option<String>,
    pub app_uri: Option<String>,
    pub property: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize)]
pub struct UpdateGraphNodePropertyResponsePayload {
    pub success: bool,
}

impl GraphNodeValidatable for UpdateGraphNodePropertyRequestPayload {
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

/// Validates the UpdateGraphNodePropertyRequestPayload based on the addon
/// extension schema.
fn validate_update_graph_node_property_request(
    request_payload: &UpdateGraphNodePropertyRequestPayload,
    state: &mut DesignerState,
) -> Result<(), String> {
    validate_node_request(request_payload, state)
}

/// Updates the property.json file with the updated graph node property.
fn update_node_property(
    base_dir: &str,
    property: &mut ten_rust::pkg_info::property::Property,
    graph_name: &str,
    node: &GraphNode,
) -> Result<(), Box<dyn std::error::Error>> {
    let nodes_to_modify = vec![node.clone()];
    crate::graph::update_graph_node_all_fields(
        base_dir,
        &mut property.all_fields,
        graph_name,
        None,
        None,
        Some(&nodes_to_modify),
    )?;
    Ok(())
}

pub async fn update_graph_node_property_endpoint(
    request_payload: web::Json<UpdateGraphNodePropertyRequestPayload>,
    state: web::Data<Arc<RwLock<DesignerState>>>,
) -> Result<impl Responder, actix_web::Error> {
    // Get a write lock on the state since we need to modify the graph.
    let mut state_write = state.write().unwrap();

    // Validate the request payload before proceeding.
    if let Err(validation_error) = validate_update_graph_node_property_request(
        &request_payload,
        &mut state_write,
    ) {
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

    // Find the node in the graph
    let node_found = graph_info.graph.nodes.iter().any(|node| {
        node.type_and_name.name == request_payload.node_name
            && node.addon == request_payload.addon_name
            && node.extension_group == request_payload.extension_group_name
            && node.app == request_payload.app_uri
    });

    if !node_found {
        let error_response = ErrorResponse {
            status: Status::Fail,
            message: format!(
                "Node '{}' with addon '{}' not found in graph '{}'",
                request_payload.node_name,
                request_payload.addon_name,
                request_payload.graph_id
            ),
            error: None,
        };
        return Ok(HttpResponse::NotFound().json(error_response));
    }

    if let Ok(Some(pkg_info)) =
        pkg_info_find_by_graph_info_mut(pkgs_cache, graph_info)
    {
        // Create the graph node with updated property.
        let node_to_update = GraphNode {
            type_and_name: PkgTypeAndName {
                pkg_type: PkgType::Extension,
                name: request_payload.node_name.clone(),
            },
            addon: request_payload.addon_name.clone(),
            extension_group: request_payload.extension_group_name.clone(),
            app: request_payload.app_uri.clone(),
            property: request_payload.property.clone(),
        };

        // Update property.json file with the updated graph node
        // property.
        if let Some(property) = &mut pkg_info.property {
            // Write the updated property_all_fields map to
            // property.json.
            if let Err(e) = update_node_property(
                &pkg_info.url,
                property,
                graph_info.name.as_ref().unwrap(),
                &node_to_update,
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
        data: UpdateGraphNodePropertyResponsePayload { success: true },
        meta: None,
    };
    Ok(HttpResponse::Ok().json(response))
}
