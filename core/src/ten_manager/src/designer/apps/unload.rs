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

use crate::designer::{
    response::{ApiResponse, ErrorResponse, Status},
    DesignerState,
};

#[derive(Deserialize, Serialize, Debug)]
pub struct UnloadAppRequestPayload {
    pub base_dir: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UnloadAppResponseData {
    pub success: bool,
}

pub async fn unload_app_endpoint(
    request_payload: web::Json<UnloadAppRequestPayload>,
    state: web::Data<Arc<RwLock<DesignerState>>>,
) -> Result<impl Responder, actix_web::Error> {
    let mut state_write = state.write().unwrap();

    // Check if the base_dir exists in pkgs_cache.
    if !state_write
        .pkgs_cache
        .contains_key(&request_payload.base_dir)
    {
        let error_response = ErrorResponse {
            status: Status::Fail,
            message: format!(
                "App with base_dir '{}' is not loaded",
                request_payload.base_dir
            ),
            error: None,
        };
        return Ok(HttpResponse::BadRequest().json(error_response));
    }

    state_write.pkgs_cache.remove(&request_payload.base_dir);

    let response = ApiResponse {
        status: Status::Ok,
        data: serde_json::json!({ "success": true }),
        meta: None,
    };

    Ok(HttpResponse::Ok().json(response))
}

