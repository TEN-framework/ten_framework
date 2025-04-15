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

use crate::designer::{
    response::{ApiResponse, Status},
    DesignerState,
};

#[derive(Serialize, Deserialize)]
pub struct ReplaceGraphNodeRequestPayload {
    pub graph_id: Uuid,

    pub old_addon: String,
    pub new_addon: String,

    pub app: Option<String>,
    pub name: String,

    pub property: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize)]
pub struct ReplaceGraphNodeResponsePayload {
    pub success: bool,
}

pub async fn replace_graph_node_endpoint(
    request_payload: web::Json<ReplaceGraphNodeRequestPayload>,
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

    // =-=-=

    let response = ApiResponse {
        status: Status::Ok,
        data: ReplaceGraphNodeResponsePayload { success: true },
        meta: None,
    };
    Ok(HttpResponse::Ok().json(response))
}
