//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::sync::{Arc, RwLock};

use crate::config::Designer;
use crate::designer::response::{ApiResponse, Status};
use crate::designer::DesignerState;
use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GetPreferencesResponseData {
    pub preferences: Designer,
}

/// Get the full content of designer frontend preferences.
pub async fn get_preferences_endpoint(
    state: web::Data<Arc<RwLock<DesignerState>>>,
) -> Result<impl Responder, actix_web::Error> {
    let state_read = state.read().unwrap();
    let preferences = state_read.tman_config.designer.clone();

    let response_data = GetPreferencesResponseData { preferences };
    let response = ApiResponse {
        status: Status::Ok,
        data: response_data,
        meta: None,
    };

    Ok(HttpResponse::Ok().json(response))
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::{Arc, RwLock};

    use actix_web::{test, web, App};

    use crate::config::TmanConfig;
    use crate::designer::DesignerState;
    use crate::output::TmanOutputCli;

    use super::*;

    #[actix_web::test]
    async fn test_get_preferences_success() {
        // Create test state.
        let state = Arc::new(RwLock::new(DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
        }));

        // Create test app.
        let app = test::init_service(
            App::new().app_data(web::Data::new(state)).service(
                web::scope("/api/designer/v1").route(
                    "/preferences",
                    web::get().to(get_preferences_endpoint),
                ),
            ),
        )
        .await;

        // Create test request.
        let req = test::TestRequest::get()
            .uri("/api/designer/v1/preferences")
            .to_request();
        let resp = test::call_service(&app, req).await;

        // Assert response status is 200 OK.
        assert!(resp.status().is_success());

        // Parse response body.
        let body = test::read_body(resp).await;
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        // Assert response structure.
        assert_eq!(json["status"], "ok");
        assert!(json["data"].is_object());
        assert!(json["data"]["preferences"].is_object());

        // Verify preferences contains the expected properties from the default
        // TmanConfig.
        let preferences = &json["data"]["preferences"];
        assert!(preferences["logviewer_line_size"].is_number());
    }

    #[actix_web::test]
    async fn test_get_preferences_invalid_path() {
        // Create test state.
        let state = Arc::new(RwLock::new(DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
        }));

        // Create test app.
        let app = test::init_service(
            App::new().app_data(web::Data::new(state)).service(
                web::scope("/api/designer/v1").route(
                    "/preferences",
                    web::get().to(get_preferences_endpoint),
                ),
            ),
        )
        .await;

        // Create test request with invalid path.
        let req = test::TestRequest::get()
            .uri("/api/designer/v1/invalid")
            .to_request();
        let resp = test::call_service(&app, req).await;

        // Assert response status is 404 Not Found.
        assert_eq!(resp.status(), 404);
    }
}
