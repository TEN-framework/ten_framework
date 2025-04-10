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
            graphs::nodes::property::update::{
                update_graph_node_property_endpoint,
                UpdateGraphNodePropertyRequestPayload,
                UpdateGraphNodePropertyResponsePayload,
            },
            response::{ApiResponse, ErrorResponse, Status},
            DesignerState,
        },
        output::TmanOutputCli,
    };

    use crate::test_case::mock::inject_all_pkgs_for_mock;

    #[actix_web::test]
    async fn test_update_graph_node_property_success() {
        // Create a test directory with property.json file.
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_dir_path = temp_dir.path().to_str().unwrap().to_string();

        // Read test data from embedded JSON files.
        let input_property_json_str =
            include_str!("../../test_data_embed/app_property.json");
        let input_manifest_json_str =
            include_str!("../../test_data_embed/app_manifest.json");

        // Write input files to temp directory.
        let property_path = std::path::Path::new(&temp_dir_path)
            .join(ten_rust::pkg_info::constants::PROPERTY_JSON_FILENAME);
        std::fs::write(&property_path, input_property_json_str).unwrap();

        let manifest_path =
            std::path::Path::new(&temp_dir_path).join("manifest.json");
        std::fs::write(&manifest_path, input_manifest_json_str).unwrap();

        // Initialize test state.
        let mut designer_state = DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
            graphs_cache: HashMap::new(),
        };

        // Inject the test app into the mock.
        let all_pkgs_json_str = vec![
            (
                input_manifest_json_str.to_string(),
                input_property_json_str.to_string(),
            ),
            (
                include_str!("../test_data_embed/test_addon_manifest.json")
                    .to_string(),
                "{}".to_string(),
            ),
        ];

        let inject_ret = inject_all_pkgs_for_mock(
            &temp_dir_path,
            &mut designer_state.pkgs_cache,
            &mut designer_state.graphs_cache,
            all_pkgs_json_str,
        );
        assert!(inject_ret.is_ok());

        let designer_state = Arc::new(RwLock::new(designer_state));

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(designer_state.clone()))
                .route(
                    "/api/designer/v1/graphs/nodes/property/update",
                    web::post().to(update_graph_node_property_endpoint),
                ),
        )
        .await;

        // Update property of an existing node.
        let request_payload = UpdateGraphNodePropertyRequestPayload {
            graph_app_base_dir: temp_dir_path.clone(),
            graph_name: "default_with_app_uri".to_string(),
            addon_app_base_dir: None,
            node_name: "extension_1".to_string(),
            addon_name: "extension_addon_1".to_string(),
            extension_group_name: Some("extension_group_1".to_string()),
            app_uri: Some("http://example.com:8000".to_string()),
            property: Some(json!({
                "key": "updated_value"
            })),
        };

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/graphs/nodes/property/update")
            .set_json(request_payload)
            .to_request();
        let resp = test::call_service(&app, req).await;

        // Should succeed with a 200 OK.
        assert_eq!(resp.status(), 200);

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();
        println!("Response body: {}", body_str);

        let response: ApiResponse<UpdateGraphNodePropertyResponsePayload> =
            serde_json::from_str(body_str).unwrap();
        assert_eq!(response.status, Status::Ok);
        assert!(response.data.success);

        let updated_property_json_str =
            std::fs::read_to_string(&property_path).unwrap();

        let expected_property_json_str = include_str!(
            "test_data_embed/expected_app_property_after_update_property.json"
        );

        // Parse the contents as JSON for proper comparison.
        let updated_property: serde_json::Value =
            serde_json::from_str(&updated_property_json_str).unwrap();
        let expected_property: serde_json::Value =
            serde_json::from_str(expected_property_json_str).unwrap();

        // Compare the updated property with the expected property.
        assert_eq!(
            updated_property, expected_property,
            "Updated property does not match expected property"
        );
    }

    #[actix_web::test]
    async fn test_update_graph_node_property_invalid_graph() {
        let mut designer_state = DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
            graphs_cache: HashMap::new(),
        };

        let all_pkgs_json_str = vec![(
            include_str!("../../test_data_embed/app_manifest.json").to_string(),
            include_str!("../../test_data_embed/app_property.json").to_string(),
        )];

        let inject_ret = inject_all_pkgs_for_mock(
            TEST_DIR,
            &mut designer_state.pkgs_cache,
            &mut designer_state.graphs_cache,
            all_pkgs_json_str,
        );
        assert!(inject_ret.is_ok());

        let designer_state = Arc::new(RwLock::new(designer_state));

        let app = test::init_service(
            App::new().app_data(web::Data::new(designer_state)).route(
                "/api/designer/v1/graphs/nodes/property/update",
                web::post().to(update_graph_node_property_endpoint),
            ),
        )
        .await;

        // Try to update a node in a non-existent graph.
        let request_payload = UpdateGraphNodePropertyRequestPayload {
            graph_app_base_dir: TEST_DIR.to_string(),
            graph_name: "non_existent_graph".to_string(),
            addon_app_base_dir: None,
            node_name: "node1".to_string(),
            addon_name: "addon1".to_string(),
            extension_group_name: None,
            app_uri: Some("http://example.com:8000".to_string()),
            property: Some(json!({
                "key": "updated_value"
            })),
        };

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/graphs/nodes/property/update")
            .set_json(request_payload)
            .to_request();
        let resp = test::call_service(&app, req).await;

        // Should fail with a 404 Not Found
        assert_eq!(resp.status(), 404);

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();
        println!("Response body: {}", body_str);

        let response: ErrorResponse = serde_json::from_str(body_str).unwrap();
        assert_eq!(response.status, Status::Fail);
        assert!(response.message.contains("not found"));
    }

    #[actix_web::test]
    async fn test_update_graph_node_property_node_not_found() {
        let mut designer_state = DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
            graphs_cache: HashMap::new(),
        };

        let all_pkgs_json_str = vec![(
            include_str!("../../test_data_embed/app_manifest.json").to_string(),
            include_str!("../../test_data_embed/app_property.json").to_string(),
        )];

        let inject_ret = inject_all_pkgs_for_mock(
            TEST_DIR,
            &mut designer_state.pkgs_cache,
            &mut designer_state.graphs_cache,
            all_pkgs_json_str,
        );
        assert!(inject_ret.is_ok());

        let designer_state = Arc::new(RwLock::new(designer_state));

        let app = test::init_service(
            App::new().app_data(web::Data::new(designer_state)).route(
                "/api/designer/v1/graphs/nodes/property/update",
                web::post().to(update_graph_node_property_endpoint),
            ),
        )
        .await;

        // Try to update a non-existent node
        let request_payload = UpdateGraphNodePropertyRequestPayload {
            graph_app_base_dir: TEST_DIR.to_string(),
            graph_name: "default_with_app_uri".to_string(),
            addon_app_base_dir: None,
            node_name: "non_existent_node".to_string(),
            addon_name: "addon1".to_string(),
            extension_group_name: None,
            app_uri: Some("http://example.com:8000".to_string()),
            property: Some(json!({
                "key": "updated_value"
            })),
        };

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/graphs/nodes/property/update")
            .set_json(request_payload)
            .to_request();
        let resp = test::call_service(&app, req).await;

        // Should fail with a 404 Not Found
        assert_eq!(resp.status(), 404);

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();
        println!("Response body: {}", body_str);

        let response: ErrorResponse = serde_json::from_str(body_str).unwrap();
        assert_eq!(response.status, Status::Fail);
        assert!(response.message.contains("not found"));
    }

    #[actix_web::test]
    async fn test_update_graph_node_property_validation_pass() {
        // Create a test directory with property.json file.
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_dir_path = temp_dir.path().to_str().unwrap().to_string();

        // Read test data from embedded JSON files.
        let input_property_json_str =
            include_str!("../../test_data_embed/app_property.json");
        let input_manifest_json_str =
            include_str!("../../test_data_embed/app_manifest.json");

        // Write input files to temp directory.
        let property_path = std::path::Path::new(&temp_dir_path)
            .join(ten_rust::pkg_info::constants::PROPERTY_JSON_FILENAME);
        std::fs::write(&property_path, input_property_json_str).unwrap();

        let manifest_path =
            std::path::Path::new(&temp_dir_path).join("manifest.json");
        std::fs::write(&manifest_path, input_manifest_json_str).unwrap();

        // Initialize test state.
        let mut designer_state = DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
            graphs_cache: HashMap::new(),
        };

        // Inject the test app into the mock.
        let all_pkgs_json_str = vec![
            (
                input_manifest_json_str.to_string(),
                input_property_json_str.to_string(),
            ),
            (
                include_str!("test_data_embed/extension_addon_1_manifest.json")
                    .to_string(),
                "{}".to_string(),
            ),
        ];

        let inject_ret = inject_all_pkgs_for_mock(
            &temp_dir_path,
            &mut designer_state.pkgs_cache,
            &mut designer_state.graphs_cache,
            all_pkgs_json_str,
        );
        assert!(inject_ret.is_ok());

        let designer_state = Arc::new(RwLock::new(designer_state));

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(designer_state.clone()))
                .route(
                    "/api/designer/v1/graphs/nodes/property/update",
                    web::post().to(update_graph_node_property_endpoint),
                ),
        )
        .await;

        // Update property of an existing node.
        let request_payload = UpdateGraphNodePropertyRequestPayload {
            graph_app_base_dir: temp_dir_path.clone(),
            graph_name: "default_with_app_uri".to_string(),
            addon_app_base_dir: Some(temp_dir_path.clone()),
            node_name: "extension_1".to_string(),
            addon_name: "extension_addon_1".to_string(),
            extension_group_name: Some("extension_group_1".to_string()),
            app_uri: Some("http://example.com:8000".to_string()),
            property: Some(json!({
                "key": "updated_value"
            })),
        };

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/graphs/nodes/property/update")
            .set_json(request_payload)
            .to_request();
        let resp = test::call_service(&app, req).await;

        // Should succeed with a 200 OK.
        assert_eq!(resp.status(), 200);

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();
        println!("Response body: {}", body_str);

        let response: ApiResponse<UpdateGraphNodePropertyResponsePayload> =
            serde_json::from_str(body_str).unwrap();
        assert_eq!(response.status, Status::Ok);
        assert!(response.data.success);

        let updated_property_json_str =
            std::fs::read_to_string(&property_path).unwrap();

        let expected_property_json_str = include_str!(
            "test_data_embed/expected_app_property_after_update_property.json"
        );

        // Parse the contents as JSON for proper comparison.
        let updated_property: serde_json::Value =
            serde_json::from_str(&updated_property_json_str).unwrap();
        let expected_property: serde_json::Value =
            serde_json::from_str(expected_property_json_str).unwrap();

        // Compare the updated property with the expected property.
        assert_eq!(
            updated_property, expected_property,
            "Updated property does not match expected property"
        );
    }

    #[actix_web::test]
    async fn test_update_graph_node_property_validation_failure() {
        // Create a test directory with property.json file.
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_dir_path = temp_dir.path().to_str().unwrap().to_string();

        // Read test data from embedded JSON files.
        let input_property_json_str =
            include_str!("../../test_data_embed/app_property.json");
        let input_manifest_json_str =
            include_str!("../../test_data_embed/app_manifest.json");

        // Write input files to temp directory.
        let property_path = std::path::Path::new(&temp_dir_path)
            .join(ten_rust::pkg_info::constants::PROPERTY_JSON_FILENAME);
        std::fs::write(&property_path, input_property_json_str).unwrap();

        let manifest_path =
            std::path::Path::new(&temp_dir_path).join("manifest.json");
        std::fs::write(&manifest_path, input_manifest_json_str).unwrap();

        // Initialize test state.
        let mut designer_state = DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
            graphs_cache: HashMap::new(),
        };

        // Inject the test app into the mock.
        let all_pkgs_json_str = vec![
            (
                input_manifest_json_str.to_string(),
                input_property_json_str.to_string(),
            ),
            (
                include_str!("test_data_embed/extension_addon_1_manifest.json")
                    .to_string(),
                "{}".to_string(),
            ),
        ];

        let inject_ret = inject_all_pkgs_for_mock(
            &temp_dir_path,
            &mut designer_state.pkgs_cache,
            &mut designer_state.graphs_cache,
            all_pkgs_json_str,
        );
        assert!(inject_ret.is_ok());

        let designer_state = Arc::new(RwLock::new(designer_state));

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(designer_state.clone()))
                .route(
                    "/api/designer/v1/graphs/nodes/property/update",
                    web::post().to(update_graph_node_property_endpoint),
                ),
        )
        .await;

        // Update property of an existing node.
        let request_payload = UpdateGraphNodePropertyRequestPayload {
            graph_app_base_dir: temp_dir_path.clone(),
            graph_name: "default_with_app_uri".to_string(),
            addon_app_base_dir: Some(temp_dir_path.clone()),
            node_name: "extension_1".to_string(),
            addon_name: "extension_addon_1".to_string(),
            extension_group_name: Some("extension_group_1".to_string()),
            app_uri: Some("http://example.com:8000".to_string()),
            property: Some(json!({
                "key": 32
            })),
        };

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/graphs/nodes/property/update")
            .set_json(request_payload)
            .to_request();
        let resp = test::call_service(&app, req).await;

        // Should fail with a 400 Bad Request.
        assert_eq!(resp.status(), 400);

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();
        println!("Response body: {}", body_str);

        let response: ErrorResponse = serde_json::from_str(body_str).unwrap();
        assert_eq!(response.status, Status::Fail);
        assert!(response.message.contains("validation failed"));

        let updated_property_json_str =
            std::fs::read_to_string(&property_path).unwrap();

        // Parse the contents as JSON for proper comparison.
        let updated_property: serde_json::Value =
            serde_json::from_str(&updated_property_json_str).unwrap();
        let expected_property: serde_json::Value =
            serde_json::from_str(input_property_json_str).unwrap();

        // Compare the updated property with the expected property.
        assert_eq!(
            updated_property, expected_property,
            "Updated property does not match expected property"
        );
    }

    #[actix_web::test]
    async fn test_update_graph_node_property_validation_complex_pass() {
        // Create a test directory with property.json file.
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_dir_path = temp_dir.path().to_str().unwrap().to_string();

        // Read test data from embedded JSON files.
        let input_property_json_str =
            include_str!("../../test_data_embed/app_property.json");
        let input_manifest_json_str =
            include_str!("../../test_data_embed/app_manifest.json");

        // Write input files to temp directory.
        let property_path = std::path::Path::new(&temp_dir_path)
            .join(ten_rust::pkg_info::constants::PROPERTY_JSON_FILENAME);
        std::fs::write(&property_path, input_property_json_str).unwrap();

        let manifest_path =
            std::path::Path::new(&temp_dir_path).join("manifest.json");
        std::fs::write(&manifest_path, input_manifest_json_str).unwrap();

        // Initialize test state.
        let mut designer_state = DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
            graphs_cache: HashMap::new(),
        };

        // Inject the test app into the mock.
        let all_pkgs_json_str = vec![
            (
                input_manifest_json_str.to_string(),
                input_property_json_str.to_string(),
            ),
            (
                include_str!("test_data_embed/extension_addon_1_with_complex_property_manifest.json")
                .to_string(),
                "{}".to_string(),
            ),
        ];

        let inject_ret = inject_all_pkgs_for_mock(
            &temp_dir_path,
            &mut designer_state.pkgs_cache,
            &mut designer_state.graphs_cache,
            all_pkgs_json_str,
        );
        assert!(inject_ret.is_ok());

        let designer_state = Arc::new(RwLock::new(designer_state));

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(designer_state.clone()))
                .route(
                    "/api/designer/v1/graphs/nodes/property/update",
                    web::post().to(update_graph_node_property_endpoint),
                ),
        )
        .await;

        // Update property of an existing node.
        let request_payload = UpdateGraphNodePropertyRequestPayload {
            graph_app_base_dir: temp_dir_path.clone(),
            graph_name: "default_with_app_uri".to_string(),
            addon_app_base_dir: Some(temp_dir_path.clone()),
            node_name: "extension_1".to_string(),
            addon_name: "extension_addon_1".to_string(),
            extension_group_name: Some("extension_group_1".to_string()),
            app_uri: Some("http://example.com:8000".to_string()),
            property: Some(json!({
                "key": {
                    "a": 1,
                    "b": "2"
                }
            })),
        };

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/graphs/nodes/property/update")
            .set_json(request_payload)
            .to_request();
        let resp = test::call_service(&app, req).await;

        // Should succeed with a 200 OK.
        assert_eq!(resp.status(), 200);

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();
        println!("Response body: {}", body_str);

        let response: ApiResponse<UpdateGraphNodePropertyResponsePayload> =
            serde_json::from_str(body_str).unwrap();
        assert_eq!(response.status, Status::Ok);
        assert!(response.data.success);

        let updated_property_json_str =
            std::fs::read_to_string(&property_path).unwrap();

        let expected_property_json_str = include_str!("test_data_embed/expected_app_property_after_update_complex_property.json"        );

        // Parse the contents as JSON for proper comparison.
        let updated_property: serde_json::Value =
            serde_json::from_str(&updated_property_json_str).unwrap();
        let expected_property: serde_json::Value =
            serde_json::from_str(expected_property_json_str).unwrap();

        // Compare the updated property with the expected property.
        assert_eq!(
            updated_property, expected_property,
            "Updated property does not match expected property"
        );
    }

    #[actix_web::test]
    async fn test_update_graph_node_property_validation_complex_failure() {
        // Create a test directory with property.json file.
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_dir_path = temp_dir.path().to_str().unwrap().to_string();

        // Read test data from embedded JSON files.
        let input_property_json_str =
            include_str!("../../test_data_embed/app_property.json");
        let input_manifest_json_str =
            include_str!("../../test_data_embed/app_manifest.json");

        // Write input files to temp directory.
        let property_path = std::path::Path::new(&temp_dir_path)
            .join(ten_rust::pkg_info::constants::PROPERTY_JSON_FILENAME);
        std::fs::write(&property_path, input_property_json_str).unwrap();

        let manifest_path =
            std::path::Path::new(&temp_dir_path).join("manifest.json");
        std::fs::write(&manifest_path, input_manifest_json_str).unwrap();

        // Initialize test state.
        let mut designer_state = DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
            graphs_cache: HashMap::new(),
        };

        // Inject the test app into the mock.
        let all_pkgs_json_str = vec![
            (
                input_manifest_json_str.to_string(),
                input_property_json_str.to_string(),
            ),
            (
                include_str!("test_data_embed/extension_addon_1_with_complex_property_manifest.json")
                .to_string(),
                "{}".to_string(),
            ),
        ];

        let inject_ret = inject_all_pkgs_for_mock(
            &temp_dir_path,
            &mut designer_state.pkgs_cache,
            &mut designer_state.graphs_cache,
            all_pkgs_json_str,
        );
        assert!(inject_ret.is_ok());

        let designer_state = Arc::new(RwLock::new(designer_state));

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(designer_state.clone()))
                .route(
                    "/api/designer/v1/graphs/nodes/property/update",
                    web::post().to(update_graph_node_property_endpoint),
                ),
        )
        .await;

        // Update property of an existing node with invalid data.
        let request_payload = UpdateGraphNodePropertyRequestPayload {
            graph_app_base_dir: temp_dir_path.clone(),
            graph_name: "default_with_app_uri".to_string(),
            addon_app_base_dir: Some(temp_dir_path.clone()),
            node_name: "extension_1".to_string(),
            addon_name: "extension_addon_1".to_string(),
            extension_group_name: Some("extension_group_1".to_string()),
            app_uri: Some("http://example.com:8000".to_string()),
            property: Some(json!({
                "key": {
                    "a": "wrong",
                    "b": "2"
                }
            })),
        };

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/graphs/nodes/property/update")
            .set_json(request_payload)
            .to_request();
        let resp = test::call_service(&app, req).await;

        // Should fail with a 400 Bad Request.
        assert_eq!(resp.status(), 400);

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();
        println!("Response body: {}", body_str);

        let response: ErrorResponse = serde_json::from_str(body_str).unwrap();
        assert_eq!(response.status, Status::Fail);
        assert!(response.message.contains("validation failed"));

        let updated_property_json_str =
            std::fs::read_to_string(&property_path).unwrap();

        // Parse the contents as JSON for proper comparison.
        let updated_property: serde_json::Value =
            serde_json::from_str(&updated_property_json_str).unwrap();
        let expected_property: serde_json::Value =
            serde_json::from_str(input_property_json_str).unwrap();

        // Compare the updated property with the expected property.
        assert_eq!(
            updated_property, expected_property,
            "Updated property does not match expected property"
        );
    }
}
