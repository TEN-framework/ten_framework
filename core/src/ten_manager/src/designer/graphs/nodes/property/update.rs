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
    graph::graph_info::GraphInfo,
    pkg_info::{
        create_uri_to_pkg_info_map, get_pkg_info_for_extension_addon, PkgInfo,
    },
};

use crate::{
    designer::{
        graphs::nodes::{
            update_graph_node_in_property_all_fields,
            validate::{validate_node_request, GraphNodeValidatable},
            GraphNodeUpdateAction,
        },
        response::{ApiResponse, ErrorResponse, Status},
        DesignerState,
    },
    graph::graphs_cache_find_by_id_mut,
};

#[derive(Serialize, Deserialize)]
pub struct UpdateGraphNodePropertyRequestPayload {
    pub graph_id: Uuid,

    pub name: String,
    pub addon: String,
    pub extension_group: Option<String>,
    pub app: Option<String>,

    pub property: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize)]
pub struct UpdateGraphNodePropertyResponsePayload {
    pub success: bool,
}

impl GraphNodeValidatable for UpdateGraphNodePropertyRequestPayload {
    fn get_addon_name(&self) -> &str {
        &self.addon
    }

    fn get_app_uri(&self) -> &Option<String> {
        &self.app
    }

    fn get_property(&self) -> &Option<serde_json::Value> {
        &self.property
    }
}

/// Validates the UpdateGraphNodePropertyRequestPayload based on the addon
/// extension schema.
fn validate_update_graph_node_property_request(
    request_payload: &UpdateGraphNodePropertyRequestPayload,
    extension_pkg_info: &PkgInfo,
) -> Result<(), String> {
    validate_node_request(request_payload, extension_pkg_info)
}

fn update_node_property_in_graph(
    graph_info: &mut GraphInfo,
    request_payload: &UpdateGraphNodePropertyRequestPayload,
) -> Result<()> {
    // Find the node in the graph.
    let graph_node = graph_info.graph.nodes.iter_mut().find(|node| {
        node.type_and_name.name == request_payload.name
            && node.addon == request_payload.addon
            && node.extension_group == request_payload.extension_group
            && node.app == request_payload.app
    });

    if graph_node.is_none() {
        return Err(anyhow::anyhow!(
            "Node '{}' with addon '{}' not found in graph '{}'",
            request_payload.name,
            request_payload.addon,
            request_payload.graph_id
        ));
    }

    // Update the node's property.
    graph_node.unwrap().property = request_payload.property.clone();

    Ok(())
}

pub async fn update_graph_node_property_endpoint(
    request_payload: web::Json<UpdateGraphNodePropertyRequestPayload>,
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

    match get_pkg_info_for_extension_addon(
        &request_payload.app,
        &request_payload.addon,
        &uri_to_pkg_info,
        graph_info.app_base_dir.as_ref(),
        pkgs_cache,
    ) {
        Some(pkg_info) => {
            // Validate the request payload before proceeding.
            if let Err(validation_error) =
                validate_update_graph_node_property_request(
                    &request_payload,
                    pkg_info,
                )
            {
                let error_response = ErrorResponse {
                    status: Status::Fail,
                    message: format!("Validation failed: {}", validation_error),
                    error: None,
                };
                return Ok(HttpResponse::BadRequest().json(error_response));
            }
        }
        None => {
            let error_response = ErrorResponse {
                status: Status::Fail,
                message: "Extension not found".to_string(),
                error: None,
            };
            return Ok(HttpResponse::NotFound().json(error_response));
        }
    };

    update_node_property_in_graph(graph_info, &request_payload).map_err(
        |e| {
            actix_web::error::ErrorInternalServerError(format!(
                "Failed to update node property in graph: {}",
                e
            ))
        },
    )?;

    update_graph_node_in_property_all_fields(
        pkgs_cache,
        graph_info,
        &request_payload.name,
        &request_payload.addon,
        &request_payload.extension_group,
        &request_payload.app,
        &request_payload.property,
        GraphNodeUpdateAction::Update,
    )
    .map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!(
            "Failed to update property.json file: {}",
            e
        ))
    })?;

    let response = ApiResponse {
        status: Status::Ok,
        data: UpdateGraphNodePropertyResponsePayload { success: true },
        meta: None,
    };
    Ok(HttpResponse::Ok().json(response))
}
