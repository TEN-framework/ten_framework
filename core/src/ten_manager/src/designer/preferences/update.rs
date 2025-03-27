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
            .map_err(actix_web::error::ErrorBadRequest)?;

    // Save to config file
    save_config_to_file(tman_config)?;

    let response = ApiResponse {
        status: Status::Ok,
        data: (),
        meta: None,
    };

    Ok(HttpResponse::Ok().json(response))
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::{Arc, RwLock};

    use actix_web::{test, web, App};
    use serde_json::json;

    use crate::config::TmanConfig;
    use crate::designer::DesignerState;
    use crate::output::TmanOutputCli;

    use super::*;

    #[actix_web::test]
    async fn test_update_preferences_success() {
        // Create test state with mock config file path to avoid writing to real
        // file.
        let config = TmanConfig {
            config_file: None,
            ..TmanConfig::default()
        };

        let state = Arc::new(RwLock::new(DesignerState {
            tman_config: Arc::new(config),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
        }));

        // Create test app.
        let app = test::init_service(
            App::new().app_data(web::Data::new(state.clone())).service(
                web::scope("/api/designer/v1").route(
                    "/preferences",
                    web::put().to(update_preferences_endpoint),
                ),
            ),
        )
        .await;

        // Create valid preferences payload.
        let payload = UpdatePreferencesRequestPayload {
            preferences: json!({
                "logviewer_line_size": 2000  // Valid value according to schema.
            }),
        };

        // Create test request.
        let req = test::TestRequest::put()
            .uri("/api/designer/v1/preferences")
            .set_json(&payload)
            .to_request();
        let resp = test::call_service(&app, req).await;

        // Assert response status is 200 OK.
        assert!(resp.status().is_success());

        // Parse response body.
        let body = test::read_body(resp).await;
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        // Assert response structure.
        assert_eq!(json["status"], "ok");

        // Verify config was updated (though we're not actually writing to file
        // in test).
        let state_read = state.read().unwrap();
        assert_eq!(state_read.tman_config.designer.logviewer_line_size, 2000);
    }

    #[actix_web::test]
    async fn test_update_preferences_invalid_schema() {
        // Create test state.
        let config = TmanConfig {
            config_file: None,
            ..TmanConfig::default()
        };

        let state = Arc::new(RwLock::new(DesignerState {
            tman_config: Arc::new(config),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
        }));

        // Create test app.
        let app = test::init_service(
            App::new().app_data(web::Data::new(state)).service(
                web::scope("/api/designer/v1").route(
                    "/preferences",
                    web::put().to(update_preferences_endpoint),
                ),
            ),
        )
        .await;

        // Create invalid preferences payload (violates schema minimum
        // constraint).
        let payload = UpdatePreferencesRequestPayload {
            preferences: json!({
                "logviewer_line_size": 50  // Invalid value - minimum is 100.
            }),
        };

        // Create test request.
        let req = test::TestRequest::put()
            .uri("/api/designer/v1/preferences")
            .set_json(&payload)
            .to_request();
        let resp = test::call_service(&app, req).await;

        // Assert response status is 400 Bad Request due to schema violation.
        assert_eq!(resp.status(), 400);
    }
}
