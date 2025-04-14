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
        constants::TEST_DIR,
        designer::{
            apps::property::schema::{
                get_app_property_schema_endpoint,
                GetAppPropertySchemaRequestPayload,
                GetAppPropertySchemaResponseData,
            },
            response::{ApiResponse, Status},
            DesignerState,
        },
        output::TmanOutputCli,
    };

    use crate::test_case::mock::inject_all_pkgs_for_mock;

    #[actix_web::test]
    async fn test_get_app_property_schema_success() {
        // Set up the designer state with initial data.
        let mut designer_state = DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            tman_internal_config: Arc::new(TmanInternalConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
            graphs_cache: HashMap::new(),
        };

        let all_pkgs_json_str = vec![(
            TEST_DIR.to_string(),
            include_str!("test_data_embed/app_manifest.json").to_string(),
            "{}".to_string(),
        )];

        let inject_ret = inject_all_pkgs_for_mock(
            &mut designer_state.pkgs_cache,
            &mut designer_state.graphs_cache,
            all_pkgs_json_str,
        );
        assert!(inject_ret.is_ok());

        let designer_state = Arc::new(RwLock::new(designer_state));

        // Set up the test service.
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(designer_state.clone()))
                .route(
                    "/test_get_app_property_schema",
                    web::post().to(get_app_property_schema_endpoint),
                ),
        )
        .await;

        // Create request payload for an existing app.
        let req = test::TestRequest::post()
            .uri("/test_get_app_property_schema")
            .set_json(&GetAppPropertySchemaRequestPayload {
                app_base_dir: TEST_DIR.to_string(),
            })
            .to_request();

        // Send the request and get the response.
        let resp = test::call_service(&app, req).await;

        // Verify the response is successful.
        assert_eq!(resp.status(), StatusCode::OK);

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();

        let api_response: ApiResponse<GetAppPropertySchemaResponseData> =
            serde_json::from_str(body_str).unwrap();

        assert_eq!(api_response.status, Status::Ok);
        // If the app has property schema, it will be included in the  response.
        assert!(api_response.data.property_schema.is_some());

        let expected_property_schema = serde_json::json!({
            "test_property": {
                "type": "int8"
            }
        });

        assert_eq!(
            serde_json::to_value(api_response.data.property_schema.unwrap())
                .unwrap(),
            expected_property_schema
        );
    }

    #[actix_web::test]
    async fn test_get_app_property_schema_app_not_found() {
        // Set up the designer state with initial data.
        let mut designer_state = DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            tman_internal_config: Arc::new(TmanInternalConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
            graphs_cache: HashMap::new(),
        };

        let all_pkgs_json_str = vec![(
            TEST_DIR.to_string(),
            include_str!("test_data_embed/app_manifest.json").to_string(),
            "{}".to_string(),
        )];

        let inject_ret = inject_all_pkgs_for_mock(
            &mut designer_state.pkgs_cache,
            &mut designer_state.graphs_cache,
            all_pkgs_json_str,
        );
        assert!(inject_ret.is_ok());

        let designer_state = Arc::new(RwLock::new(designer_state));

        // Set up the test service.
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(designer_state.clone()))
                .route(
                    "/test_get_app_property_schema_not_found",
                    web::post().to(get_app_property_schema_endpoint),
                ),
        )
        .await;

        // Create request payload for a non-existent extension.
        let req = test::TestRequest::post()
            .uri("/test_get_app_property_schema_not_found")
            .set_json(&GetAppPropertySchemaRequestPayload {
                app_base_dir: "non_existent_app".to_string(),
            })
            .to_request();

        // Send the request and get the response.
        let resp = test::call_service(&app, req).await;

        // Verify the response is not found.
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[actix_web::test]
    async fn test_get_extension_property_schema_app_not_found() {
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
                    "/test_get_app_property_schema_app_not_found",
                    web::post().to(get_app_property_schema_endpoint),
                ),
        )
        .await;

        // Create request payload for a non-existent app.
        let req = test::TestRequest::post()
            .uri("/test_get_app_property_schema_app_not_found")
            .set_json(&GetAppPropertySchemaRequestPayload {
                app_base_dir: "non_existent_app".to_string(),
            })
            .to_request();

        // Send the request and get the response.
        let resp = test::call_service(&app, req).await;

        // Verify the response is not found.
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }
}
