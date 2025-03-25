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
            apps::load::{
                load_app_endpoint, LoadAppRequestPayload, LoadAppResponseData,
            },
            response::{ApiResponse, Status},
            DesignerState,
        },
        output::TmanOutputCli,
    };

    #[actix_web::test]
    async fn test_load_app_success_with_app_uri() {
        // Set up the designer state with initial data.
        let tman_config = TmanConfig {
            verbose: true,
            ..TmanConfig::default()
        };

        let designer_state = DesignerState {
            tman_config: Arc::new(tman_config),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
        };

        let designer_state = Arc::new(RwLock::new(designer_state));

        // Set up the test service.
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(designer_state.clone()))
                .route(
                    "/api/designer/v1/apps",
                    web::post().to(load_app_endpoint),
                ),
        )
        .await;

        // Create request with base_dir specified.
        let request_payload = LoadAppRequestPayload {
            base_dir: "tests/test_data/cmd_builtin_function_install"
                .to_string(),
        };

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/apps")
            .set_json(request_payload)
            .to_request();

        // Send the request and get the response.
        let resp = test::call_service(&app, req).await;

        let resp_status = &resp.status();
        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();

        // Verify the response.
        if resp_status != &StatusCode::OK {
            println!("{}", body_str);

            panic!("Failed to load app");
        }

        let api_response: ApiResponse<LoadAppResponseData> =
            serde_json::from_str(body_str).unwrap();
        assert_eq!(api_response.status, Status::Ok);
        assert_eq!(
            api_response.data.app_uri,
            Some("msgpack://localhost:8000".to_string())
        );
    }

    #[actix_web::test]
    async fn test_load_app_success_without_app_uri() {
        // Set up the designer state with initial data.
        let designer_state = DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
        };

        let designer_state = Arc::new(RwLock::new(designer_state));

        // Set up the test service.
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(designer_state.clone()))
                .route(
                    "/api/designer/v1/apps",
                    web::post().to(load_app_endpoint),
                ),
        )
        .await;

        // Create request with base_dir specified.
        let request_payload = LoadAppRequestPayload {
            base_dir: "tests/test_data/cmd_builtin_function_install_all"
                .to_string(),
        };

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/apps")
            .set_json(request_payload)
            .to_request();

        // Send the request and get the response.
        let resp = test::call_service(&app, req).await;

        // Verify the response.
        assert_eq!(resp.status(), StatusCode::OK);

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();

        let api_response: ApiResponse<LoadAppResponseData> =
            serde_json::from_str(body_str).unwrap();
        assert_eq!(api_response.status, Status::Ok);
        assert_eq!(api_response.data.app_uri, None);
    }
}
