//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::sync::{Arc, RwLock};

use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use ten_rust::graph::{connection::GraphConnection, node::GraphNode};
use uuid::Uuid;

use crate::designer::{
    response::{ApiResponse, ErrorResponse, Status},
    DesignerState,
};

#[derive(Serialize, Deserialize)]
pub struct UpdateGraphRequestPayload {
    pub graph_id: Uuid,
    pub nodes: Vec<GraphNode>,
    pub connections: Vec<GraphConnection>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct UpdateGraphResponseData {
    pub success: bool,
}

pub async fn update_graph_endpoint(
    request_payload: web::Json<UpdateGraphRequestPayload>,
    state: web::Data<Arc<RwLock<DesignerState>>>,
) -> Result<impl Responder, actix_web::Error> {
    let mut state_write = state.write().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!(
            "Failed to acquire write lock: {}",
            e
        ))
    })?;

    // Get the graphs_cache from the state.
    let graphs_cache = &mut state_write.graphs_cache;

    // Call the update_graph_endpoint function from the graph module.
    let result = crate::graph::update_graph_endpoint(
        graphs_cache,
        &request_payload.graph_id,
        &request_payload.nodes,
        &request_payload.connections,
    );

    // Handle the result
    match result {
        Ok(_) => {
            let response = ApiResponse {
                status: Status::Ok,
                data: UpdateGraphResponseData { success: true },
                meta: None,
            };
            Ok(HttpResponse::Ok().json(response))
        }
        Err(err) => {
            let error_response = ErrorResponse {
                status: Status::Fail,
                message: err,
                error: None,
            };
            Ok(HttpResponse::BadRequest().json(error_response))
        }
    }
}
