//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::sync::{Arc, RwLock};

use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

use ten_rust::graph::connection::GraphConnection;
use ten_rust::graph::node::GraphNode;
use ten_rust::pkg_info::predefined_graphs::get_pkg_predefined_graph_from_nodes_and_connections;

use super::connections::get::GraphConnectionsSingleResponseData;
use super::nodes::get::GraphNodesSingleResponseData;
use super::util::find_app_package_from_base_dir;
use crate::designer::response::{ApiResponse, ErrorResponse, Status};
use crate::designer::DesignerState;

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
pub struct GraphUpdateRequestPayload {
    pub base_dir: String,

    pub graph_name: String,

    pub auto_start: bool,
    pub nodes: Vec<GraphNodesSingleResponseData>,
    pub connections: Vec<GraphConnectionsSingleResponseData>,
}

#[derive(Serialize, Debug, PartialEq)]
pub struct GraphUpdateResponseData {
    pub success: bool,
}

pub async fn update_graph_endpoint(
    request_payload: web::Json<GraphUpdateRequestPayload>,
    state: web::Data<Arc<RwLock<DesignerState>>>,
) -> Result<impl Responder, actix_web::Error> {
    let mut state_write = state.write().unwrap();

    if let Some(base_dir_pkg_info) =
        state_write.pkgs_cache.get_mut(&request_payload.base_dir)
    {
        if let Some(app_pkg) = find_app_package_from_base_dir(base_dir_pkg_info)
        {
            let graph_name = &request_payload.graph_name;

            // Collect nodes into a Vec<GraphNode>.
            let nodes: Vec<GraphNode> = request_payload
                .nodes
                .iter()
                .cloned()
                .map(|v| v.into())
                .collect();

            // Collect connections into a Vec<GraphConnection>.
            let connections: Vec<GraphConnection> = request_payload
                .connections
                .iter()
                .cloned()
                .map(|v| v.into())
                .collect();

            let new_graph =
                match get_pkg_predefined_graph_from_nodes_and_connections(
                    graph_name,
                    request_payload.auto_start,
                    &nodes,
                    &connections,
                ) {
                    Ok(graph) => graph,
                    Err(err) => {
                        let error_response = ErrorResponse::from_error(
                            &err,
                            "Invalid input data:",
                        );
                        return Ok(
                            HttpResponse::NotFound().json(error_response)
                        );
                    }
                };

            // TODO(Liu): Add graph check.
            app_pkg.update_predefined_graph(&new_graph);

            let response = ApiResponse {
                status: Status::Ok,
                data: GraphUpdateResponseData { success: true },
                meta: None,
            };

            Ok(HttpResponse::Ok().json(response))
        } else {
            let error_response = ErrorResponse {
                status: Status::Fail,
                message: "Failed to find app package.".to_string(),
                error: None,
            };
            Ok(HttpResponse::NotFound().json(error_response))
        }
    } else {
        let error_response = ErrorResponse {
            status: Status::Fail,
            message: "Package information is missing".to_string(),
            error: None,
        };
        Ok(HttpResponse::NotFound().json(error_response))
    }
}
