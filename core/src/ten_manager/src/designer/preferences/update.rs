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
use crate::schema::validate_designer_config;

use super::save_config_to_file;

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdatePreferencesRequestPayload {
    pub preferences: Value,
}

/// Update the full content of designer frontend preferences.
pub async fn update_preferences_endpoint(
    request_payload: web::Json<UpdatePreferencesRequestPayload>,
    state: web::Data<Arc<RwLock<DesignerState>>>,
) -> Result<impl Responder, actix_web::Error> {
    // Validate against schema.
    if let Err(e) = validate_designer_config(&request_payload.preferences) {
        return Err(actix_web::error::ErrorBadRequest(e.to_string()));
    }

    let mut state_write = state.write().unwrap();
    let tman_config =
        Arc::get_mut(&mut state_write.tman_config).ok_or_else(|| {
            actix_web::error::ErrorInternalServerError(
                "Failed to get mutable TmanConfig",
            )
        })?;

    // Update designer field
    tman_config.designer =
        serde_json::from_value(request_payload.preferences.clone())
            .map_err(|e| actix_web::error::ErrorBadRequest(e))?;

    // Save to config file
    save_config_to_file(tman_config)?;

    let response = ApiResponse {
        status: Status::Ok,
        data: (),
        meta: None,
    };

    Ok(HttpResponse::Ok().json(response))
}
