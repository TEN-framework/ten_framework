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
        config::{internal::TmanInternalConfig, TmanConfig},
        designer::{
            apps::scripts::{
                get_app_scripts_endpoint, GetPackagesScriptsRequestPayload,
                GetPackagesScriptsResponseData,
            },
            response::{ApiResponse, Status},
            DesignerState,
        },
        output::TmanOutputCli,
        pkg_info::get_all_pkgs::get_all_pkgs_in_app,
    };

    #[actix_web::test]
    async fn test_get_apps_scripts_success() {
        // Set up the designer state with initial data.
        let mut designer_state = DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            tman_internal_config: Arc::new(TmanInternalConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
            graphs_cache: HashMap::new(),
        };

        let _ = get_all_pkgs_in_app(
            &mut designer_state.pkgs_cache,
            &mut designer_state.graphs_cache,
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
                    "/api/designer/v1/apps/scripts",
                    web::post().to(get_app_scripts_endpoint),
                ),
        )
        .await;

        // Create request with base_dir.
        let request_payload = GetPackagesScriptsRequestPayload {
            base_dir: "tests/test_data/app_with_uri".to_string(),
        };

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/apps/scripts")
            .set_json(request_payload)
            .to_request();

        // Send the request and get the response.
        let resp = test::call_service(&app, req).await;

        // Verify the response.
        assert_eq!(resp.status(), StatusCode::OK);

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();

        let api_response: ApiResponse<GetPackagesScriptsResponseData> =
            serde_json::from_str(body_str).unwrap();
        assert_eq!(api_response.status, Status::Ok);

        // Verify the scripts content
        let scripts = api_response.data.scripts.unwrap();
        assert_eq!(scripts.len(), 2);
        assert!(scripts.contains(&"start".to_string()));
        assert!(scripts.contains(&"build".to_string()));
    }

    #[actix_web::test]
    async fn test_get_apps_scripts_base_dir_not_found() {
        // Set up the designer state with initial data.
        let designer_state = DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            tman_internal_config: Arc::new(TmanInternalConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
            graphs_cache: HashMap::new(),
        };

        let designer_state = Arc::new(RwLock::new(designer_state));

        // Set up the test service.
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(designer_state.clone()))
                .route(
                    "/api/designer/v1/apps/scripts",
                    web::post().to(get_app_scripts_endpoint),
                ),
        )
        .await;

        // Create request with non-existent base_dir.
        let request_payload = GetPackagesScriptsRequestPayload {
            base_dir: "non_existent_dir".to_string(),
        };

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/apps/scripts")
            .set_json(request_payload)
            .to_request();

        // Send the request and get the response.
        let resp = test::call_service(&app, req).await;

        // Verify the response.
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();
        println!("Response body: {}", body_str);
    }
}
