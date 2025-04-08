//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        sync::{Arc, RwLock},
    };

    use actix_web::{http::StatusCode, test, web, App};
    use ten_manager::{
        config::TmanConfig,
        designer::{
            apps::reload::{reload_app_endpoint, ReloadPkgsRequestPayload},
            response::{ApiResponse, Status},
            DesignerState,
        },
        output::TmanOutputCli,
        pkg_info::get_all_pkgs::get_all_pkgs_in_app,
    };

    /// Test successful package reload with a specified base_dir.
    #[actix_web::test]
    async fn test_reload_app_success_with_base_dir() {
        // Set up the designer state with initial data.
        let mut designer_state = DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
        };

        let _ = get_all_pkgs_in_app(
            &mut designer_state.pkgs_cache,
            &"tests/test_data/app_with_uri".to_string(),
        );

        assert_eq!(
            designer_state
                .pkgs_cache
                .get("tests/test_data/app_with_uri")
                .unwrap()
                .len(),
            3
        );

        let designer_state = Arc::new(RwLock::new(designer_state));

        // Set up the test service.
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(designer_state.clone()))
                .route(
                    "/api/designer/v1/apps/reload",
                    web::post().to(reload_app_endpoint),
                ),
        )
        .await;

        // Create request with base_dir specified.
        let request_payload = ReloadPkgsRequestPayload {
            base_dir: Some("tests/test_data/app_with_uri".to_string()),
        };

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/apps/reload")
            .set_json(request_payload)
            .to_request();

        // Send the request and get the response.
        let resp = test::call_service(&app, req).await;

        // Verify the response.
        assert_eq!(resp.status(), StatusCode::OK);

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();

        let api_response: ApiResponse<&str> =
            serde_json::from_str(body_str).unwrap();
        assert_eq!(api_response.status, Status::Ok);
        assert_eq!(api_response.data, "Packages reloaded successfully");

        // Verify that the package cache still contains the base_dir entry
        // (it was reloaded, not removed).
        let state_read = designer_state.read().unwrap();
        assert!(state_read
            .pkgs_cache
            .contains_key("tests/test_data/app_with_uri"));
    }

    /// Test successful package reload without specifying base_dir when only one
    /// app is loaded.
    #[actix_web::test]
    async fn test_reload_app_success_without_base_dir() {
        // Set up the designer state with initial data.
        let mut designer_state = DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
        };

        let _ = get_all_pkgs_in_app(
            &mut designer_state.pkgs_cache,
            &"tests/test_data/app_with_uri".to_string(),
        );

        assert_eq!(
            designer_state
                .pkgs_cache
                .get("tests/test_data/app_with_uri")
                .unwrap()
                .len(),
            3
        );

        let designer_state = Arc::new(RwLock::new(designer_state));

        // Set up the test service.
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(designer_state.clone()))
                .route(
                    "/api/designer/v1/apps/reload",
                    web::post().to(reload_app_endpoint),
                ),
        )
        .await;

        // Create request without base_dir specified.
        let request_payload = ReloadPkgsRequestPayload { base_dir: None };

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/apps/reload")
            .set_json(request_payload)
            .to_request();

        // Send the request and get the response.
        let resp = test::call_service(&app, req).await;

        // Verify the response.
        assert_eq!(resp.status(), StatusCode::OK);

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();

        let api_response: ApiResponse<&str> =
            serde_json::from_str(body_str).unwrap();
        assert_eq!(api_response.status, Status::Ok);
        assert_eq!(api_response.data, "Packages reloaded successfully");

        // Verify that the package cache still contains the base_dir entry.
        let state_read = designer_state.read().unwrap();
        assert!(state_read
            .pkgs_cache
            .contains_key("tests/test_data/app_with_uri"));
    }
}
