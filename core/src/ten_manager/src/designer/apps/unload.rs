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
    response::{ApiResponse, Status},
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
    let mut state = state.write().unwrap();
    state.pkgs_cache.remove(&request_payload.base_dir);

    let response = ApiResponse {
        status: Status::Ok,
        data: serde_json::json!({ "success": true }),
        meta: None,
    };

    Ok(HttpResponse::Ok().json(response))
}
