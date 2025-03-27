//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::sync::{Arc, RwLock};

use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::designer::response::{ApiResponse, Status};
use crate::designer::DesignerState;

#[derive(Debug, Serialize, Deserialize)]
pub struct GetPreferencesResponseData {
    pub preferences: Value,
}

/// Get the full content of designer frontend preferences.
pub async fn get_preferences_endpoint(
    state: web::Data<Arc<RwLock<DesignerState>>>,
) -> Result<impl Responder, actix_web::Error> {
    let state_read = state.read().unwrap();
    let preferences = serde_json::to_value(&state_read.tman_config.designer)
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    let response_data = GetPreferencesResponseData { preferences };
    let response = ApiResponse {
        status: Status::Ok,
        data: response_data,
        meta: None,
    };

    Ok(HttpResponse::Ok().json(response))
}
