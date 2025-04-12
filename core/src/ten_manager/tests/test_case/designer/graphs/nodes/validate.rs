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

    use actix_web::{test, web, App};
    use serde_json::json;
    use ten_manager::{
        config::TmanConfig,
        constants::TEST_DIR,
        designer::{
            graphs::nodes::validate::{
                validate_graph_node_endpoint, ValidateGraphNodeRequestPayload,
                ValidateGraphNodeResponsePayload,
            },
            DesignerState,
        },
        output::TmanOutputCli,
    };

    use crate::test_case::mock::inject_all_pkgs_for_mock;

    #[actix_web::test]
    async fn test_validate_graph_node_success() {
        let mut designer_state = DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
            graphs_cache: HashMap::new(),
        };

        let all_pkgs_json_str = vec![
            (
                TEST_DIR.to_string(),
                include_str!("../test_data_embed/app_manifest.json")
                    .to_string(),
                include_str!("../test_data_embed/app_property.json")
                    .to_string(),
            ),
            (
                format!(
                    "{}{}",
                    TEST_DIR.to_string(),
                    "/ten_packages/extension/extension_1"
                ),
                include_str!(
                    "../test_data_embed/extension_addon_1_manifest.json"
                )
                .to_string(),
                "{}".to_string(),
            ),
        ];

        let inject_ret = inject_all_pkgs_for_mock(
            &mut designer_state.pkgs_cache,
            &mut designer_state.graphs_cache,
            all_pkgs_json_str,
        );
        assert!(inject_ret.is_ok());

        let designer_state = Arc::new(RwLock::new(designer_state));

        let app = test::init_service(
            App::new().app_data(web::Data::new(designer_state)).route(
                "/api/designer/v1/graphs/nodes/validate",
                web::post().to(validate_graph_node_endpoint),
            ),
        )
        .await;

        // Valid request for an existing extension with matching app_uri.
        let request_payload = ValidateGraphNodeRequestPayload {
            addon_app_base_dir: Some(TEST_DIR.to_string()),
            node_name: "extension_1".to_string(),
            addon_name: "extension_addon_1".to_string(),
            extension_group_name: Some("extension_group_1".to_string()),
            app_uri: Some("http://example.com:8000".to_string()),
            property: None,
        };

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/graphs/nodes/validate")
            .set_json(request_payload)
            .to_request();
        let resp = test::call_service(&app, req).await;

        // Should succeed with a 200 OK
        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();

        let response: ValidateGraphNodeResponsePayload =
            serde_json::from_str(body_str).unwrap();
        assert!(response.success);
    }

    #[actix_web::test]
    async fn test_validate_graph_node_with_property_success() {
        let mut designer_state = DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
            graphs_cache: HashMap::new(),
        };

        // Include manifests with schema definitions
        let all_pkgs_json_str = vec![
            (
                TEST_DIR.to_string(),
                include_str!("../test_data_embed/app_manifest.json")
                    .to_string(),
                include_str!("../test_data_embed/app_property.json")
                    .to_string(),
            ),
            (
                format!(
                    "{}{}",
                    TEST_DIR.to_string(),
                    "/ten_packages/extension/extension_1"
                ),
                include_str!(
                    "../test_data_embed/extension_addon_1_manifest_with_property_schema.json"
                )
                .to_string(),
                "{}".to_string(),
            ),
        ];

        let inject_ret = inject_all_pkgs_for_mock(
            &mut designer_state.pkgs_cache,
            &mut designer_state.graphs_cache,
            all_pkgs_json_str,
        );
        assert!(inject_ret.is_ok());

        let designer_state = Arc::new(RwLock::new(designer_state));

        let app = test::init_service(
            App::new().app_data(web::Data::new(designer_state)).route(
                "/api/designer/v1/graphs/nodes/validate",
                web::post().to(validate_graph_node_endpoint),
            ),
        )
        .await;

        // Valid request with a valid property
        let request_payload = ValidateGraphNodeRequestPayload {
            addon_app_base_dir: Some(TEST_DIR.to_string()),
            node_name: "extension_1".to_string(),
            addon_name: "extension_addon_1".to_string(),
            extension_group_name: Some("extension_group_1".to_string()),
            app_uri: Some("http://example.com:8000".to_string()),
            property: Some(json!({
                "test_property": 42
            })),
        };

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/graphs/nodes/validate")
            .set_json(request_payload)
            .to_request();
        let resp = test::call_service(&app, req).await;

        // Should succeed with a 200 OK
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_validate_graph_node_with_property_failure() {
        let mut designer_state = DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
            graphs_cache: HashMap::new(),
        };

        // Include manifests with schema definitions
        let all_pkgs_json_str = vec![
            (
                TEST_DIR.to_string(),
                include_str!("../test_data_embed/app_manifest.json")
                    .to_string(),
                include_str!("../test_data_embed/app_property.json")
                    .to_string(),
            ),
            (
                format!(
                    "{}{}",
                    TEST_DIR.to_string(),
                    "/ten_packages/extension/extension_1"
                ),
                include_str!(
                    "../test_data_embed/extension_addon_1_manifest_with_property_schema.json"
                )
                .to_string(),
                "{}".to_string(),
            ),
        ];

        let inject_ret = inject_all_pkgs_for_mock(
            &mut designer_state.pkgs_cache,
            &mut designer_state.graphs_cache,
            all_pkgs_json_str,
        );
        assert!(inject_ret.is_ok());

        let designer_state = Arc::new(RwLock::new(designer_state));

        let app = test::init_service(
            App::new().app_data(web::Data::new(designer_state)).route(
                "/api/designer/v1/graphs/nodes/validate",
                web::post().to(validate_graph_node_endpoint),
            ),
        )
        .await;

        // Valid request with a valid property
        let request_payload = ValidateGraphNodeRequestPayload {
            addon_app_base_dir: Some(TEST_DIR.to_string()),
            node_name: "extension_1".to_string(),
            addon_name: "extension_addon_1".to_string(),
            extension_group_name: Some("extension_group_1".to_string()),
            app_uri: Some("http://example.com:8000".to_string()),
            property: Some(json!({
                "test_property": "wrong"
            })),
        };

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/graphs/nodes/validate")
            .set_json(request_payload)
            .to_request();
        let resp = test::call_service(&app, req).await;

        // Should fail with a 400 Bad Request
        assert_eq!(resp.status(), 400);
    }

    #[actix_web::test]
    async fn test_validate_graph_node_no_base_dir() {
        let designer_state = Arc::new(RwLock::new(DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
            graphs_cache: HashMap::new(),
        }));

        let app = test::init_service(
            App::new().app_data(web::Data::new(designer_state)).route(
                "/api/designer/v1/graphs/nodes/validate",
                web::post().to(validate_graph_node_endpoint),
            ),
        )
        .await;

        // No addon_app_base_dir specified (should pass as validation is
        // skipped).
        let request_payload = ValidateGraphNodeRequestPayload {
            addon_app_base_dir: None,
            node_name: "test_node".to_string(),
            addon_name: "test_addon".to_string(),
            extension_group_name: None,
            app_uri: None,
            property: None,
        };

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/graphs/nodes/validate")
            .set_json(request_payload)
            .to_request();
        let resp = test::call_service(&app, req).await;

        // Should succeed with a 200 OK as validation is skipped.
        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();

        let response: ValidateGraphNodeResponsePayload =
            serde_json::from_str(body_str).unwrap();
        assert!(response.success);
    }

    #[actix_web::test]
    async fn test_validate_graph_node_invalid_base_dir() {
        let designer_state = Arc::new(RwLock::new(DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
            graphs_cache: HashMap::new(),
        }));

        let app = test::init_service(
            App::new().app_data(web::Data::new(designer_state)).route(
                "/api/designer/v1/graphs/nodes/validate",
                web::post().to(validate_graph_node_endpoint),
            ),
        )
        .await;

        // Invalid base directory.
        let request_payload = ValidateGraphNodeRequestPayload {
            addon_app_base_dir: Some("non_existent_dir".to_string()),
            node_name: "test_node".to_string(),
            addon_name: "test_addon".to_string(),
            extension_group_name: None,
            app_uri: None,
            property: None,
        };

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/graphs/nodes/validate")
            .set_json(request_payload)
            .to_request();
        let resp = test::call_service(&app, req).await;

        // Should fail with a 400 Bad Request.
        assert_eq!(resp.status(), 400);

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();

        assert!(body_str.contains("not found in pkgs_cache"));
    }

    #[actix_web::test]
    async fn test_validate_graph_node_app_uri_mismatch() {
        let mut designer_state = DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
            graphs_cache: HashMap::new(),
        };

        let all_pkgs_json_str = vec![(
            TEST_DIR.to_string(),
            include_str!("../test_data_embed/app_manifest.json").to_string(),
            include_str!("../test_data_embed/app_property.json").to_string(),
        )];

        let inject_ret = inject_all_pkgs_for_mock(
            &mut designer_state.pkgs_cache,
            &mut designer_state.graphs_cache,
            all_pkgs_json_str,
        );
        assert!(inject_ret.is_ok());

        let designer_state = Arc::new(RwLock::new(designer_state));

        let app = test::init_service(
            App::new().app_data(web::Data::new(designer_state)).route(
                "/api/designer/v1/graphs/nodes/validate",
                web::post().to(validate_graph_node_endpoint),
            ),
        )
        .await;

        // App URI mismatch
        let request_payload = ValidateGraphNodeRequestPayload {
            addon_app_base_dir: Some(TEST_DIR.to_string()),
            node_name: "test_node".to_string(),
            addon_name: "test_addon".to_string(),
            extension_group_name: None,
            app_uri: Some("http://different-uri.com".to_string()),
            property: None,
        };

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/graphs/nodes/validate")
            .set_json(request_payload)
            .to_request();
        let resp = test::call_service(&app, req).await;

        // Should fail with a 400 Bad Request.
        assert_eq!(resp.status(), 400);

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();

        assert!(body_str.contains("app_uri"));
    }

    #[actix_web::test]
    async fn test_validate_graph_node_extension_not_found() {
        let mut designer_state = DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
            graphs_cache: HashMap::new(),
        };

        // Only include the app manifest and property, but exclude extension
        // manifests.
        let all_pkgs_json_str = vec![
            (
                TEST_DIR.to_string(),
                include_str!("../test_data_embed/app_manifest.json")
                    .to_string(),
                include_str!("../test_data_embed/app_property.json")
                    .to_string(),
            ),
            // Add extension_addon_1 manifest to ensure extension validation
            // kicks in.
            (
                format!(
                    "{}{}",
                    TEST_DIR.to_string(),
                    "/ten_packages/extension/extension_1"
                ),
                include_str!(
                    "../test_data_embed/extension_addon_1_manifest.json"
                )
                .to_string(),
                "{}".to_string(),
            ),
        ];

        let inject_ret = inject_all_pkgs_for_mock(
            &mut designer_state.pkgs_cache,
            &mut designer_state.graphs_cache,
            all_pkgs_json_str,
        );
        assert!(inject_ret.is_ok());

        let designer_state = Arc::new(RwLock::new(designer_state));

        let app = test::init_service(
            App::new().app_data(web::Data::new(designer_state)).route(
                "/api/designer/v1/graphs/nodes/validate",
                web::post().to(validate_graph_node_endpoint),
            ),
        )
        .await;

        // Try to validate a non-existent extension
        let request_payload = ValidateGraphNodeRequestPayload {
            addon_app_base_dir: Some(TEST_DIR.to_string()),
            node_name: "test_node".to_string(),
            addon_name: "non_existent_addon".to_string(),
            extension_group_name: None,
            app_uri: Some("http://example.com:8000".to_string()),
            property: None,
        };

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/graphs/nodes/validate")
            .set_json(request_payload)
            .to_request();
        let resp = test::call_service(&app, req).await;

        // Should fail with a 400 Bad Request.
        assert_eq!(resp.status(), 400);

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();

        assert!(body_str.contains("Extension 'non_existent_addon' not found"));
    }
}
