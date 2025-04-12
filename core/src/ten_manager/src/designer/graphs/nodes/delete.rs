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
    pkg_info::{pkg_type::PkgType, pkg_type_and_name::PkgTypeAndName},
};
use uuid::Uuid;

use crate::{
    designer::{
        response::{ApiResponse, ErrorResponse, Status},
        DesignerState,
    },
    graph::{graphs_cache_find_by_id_mut, update_graph_node_all_fields},
    pkg_info::pkg_info_find_by_graph_info_mut,
};

#[derive(Serialize, Deserialize)]
pub struct DeleteGraphNodeRequestPayload {
    pub graph_id: Uuid,

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

    // Delete the extension node.
    match graph_info.graph.delete_extension_node(
        request_payload.node_name.clone(),
        request_payload.addon_name.clone(),
        request_payload.app_uri.clone(),
        request_payload.extension_group_name.clone(),
    ) {
        Ok(_) => {
            if let Ok(Some(pkg_info)) =
                pkg_info_find_by_graph_info_mut(pkgs_cache, graph_info)
            {
                // Update property.json file to remove the graph node.
                if let Some(property) = &mut pkg_info.property {
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
                        &pkg_info.url,
                        &mut property.all_fields,
                        graph_info.name.as_ref().unwrap(),
                        None,
                        Some(&nodes_to_remove),
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
                data: DeleteGraphNodeResponsePayload { success: true },
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
}
