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
        fs,
        path::Path,
        sync::{Arc, RwLock},
    };

    use actix_web::{test, web, App};
    use serde_json::Value;
    use ten_manager::{
        config::{internal::TmanInternalConfig, TmanConfig},
        constants::TEST_DIR,
        designer::{
            graphs::nodes::add::{
                add_graph_node_endpoint, AddGraphNodeRequestPayload,
                AddGraphNodeResponsePayload,
            },
            response::{ApiResponse, ErrorResponse, Status},
            DesignerState,
        },
        graph::graphs_cache_find_by_name,
        output::TmanOutputCli,
    };
    use ten_rust::pkg_info::{constants::PROPERTY_JSON_FILENAME, localhost};
    use uuid::Uuid;

    use crate::test_case::mock::inject_all_pkgs_for_mock;

    #[actix_web::test]
    async fn test_add_graph_node_invalid_graph() {
        let mut designer_state = DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            tman_internal_config: Arc::new(TmanInternalConfig::default()),
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
                "/api/designer/v1/graphs/nodes/add",
                web::post().to(add_graph_node_endpoint),
            ),
        )
        .await;

        // Try to add a node to a non-existent graph.
        let request_payload = AddGraphNodeRequestPayload {
            graph_id: Uuid::new_v4(),
            node_name: "test_node".to_string(),
            addon_name: "test_addon".to_string(),
            extension_group_name: None,
            app_uri: Some("http://test-app-uri.com".to_string()),
            property: None,
        };

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/graphs/nodes/add")
            .set_json(request_payload)
            .to_request();
        let resp = test::call_service(&app, req).await;

        // Should fail with a 404 Not Found.
        assert_eq!(resp.status(), 404);

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();
        println!("Response body: {}", body_str);

        let response: ErrorResponse = serde_json::from_str(body_str).unwrap();
        assert_eq!(response.status, Status::Fail);
        assert!(response.message.contains("not found"));
    }

    #[actix_web::test]
    async fn test_add_graph_node_invalid_app_uri() {
        let mut designer_state = DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            tman_internal_config: Arc::new(TmanInternalConfig::default()),
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

        let (graph_id, _) =
            graphs_cache_find_by_name(&designer_state.graphs_cache, "default")
                .unwrap();

        // Try to add a node with localhost app URI (which is not allowed).
        let request_payload = AddGraphNodeRequestPayload {
            graph_id: *graph_id,
            node_name: "test_node".to_string(),
            addon_name: "test_addon".to_string(),
            extension_group_name: None,
            app_uri: Some(localhost().to_string()),
            property: None,
        };

        let designer_state = Arc::new(RwLock::new(designer_state));

        let app = test::init_service(
            App::new().app_data(web::Data::new(designer_state)).route(
                "/api/designer/v1/graphs/nodes/add",
                web::post().to(add_graph_node_endpoint),
            ),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/graphs/nodes/add")
            .set_json(request_payload)
            .to_request();
        let resp = test::call_service(&app, req).await;

        println!("Response: {:?}", resp);

        // Should fail with a 400 Bad Request.
        assert_eq!(resp.status(), 400);

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();
        println!("Response body: {}", body_str);

        let response: ErrorResponse = serde_json::from_str(body_str).unwrap();
        assert_eq!(response.status, Status::Fail);
        assert!(response.message.contains("Failed to add node"));
    }

    #[actix_web::test]
    async fn test_add_graph_node_success() {
        let mut designer_state = DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            tman_internal_config: Arc::new(TmanInternalConfig::default()),
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

        let (graph_id, _) = graphs_cache_find_by_name(
            &designer_state.graphs_cache,
            "default_with_app_uri",
        )
        .unwrap();

        // Add a node to the default graph with the same app URI as other nodes
        let request_payload = AddGraphNodeRequestPayload {
            graph_id: *graph_id,
            node_name: "test_node".to_string(),
            addon_name: "test_addon".to_string(),
            extension_group_name: None,
            app_uri: Some("http://example.com:8000".to_string()),
            property: None,
        };

        let designer_state = Arc::new(RwLock::new(designer_state));

        let app = test::init_service(
            App::new().app_data(web::Data::new(designer_state)).route(
                "/api/designer/v1/graphs/nodes/add",
                web::post().to(add_graph_node_endpoint),
            ),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/graphs/nodes/add")
            .set_json(request_payload)
            .to_request();

        let resp = test::call_service(&app, req).await;

        let status = resp.status();
        println!("Response status: {}", status);

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();
        println!("Response body: {}", body_str);

        // Should succeed with a 200 OK.
        assert_eq!(status, 200);

        let response: ApiResponse<AddGraphNodeResponsePayload> =
            serde_json::from_str(body_str).unwrap();
        assert_eq!(response.status, Status::Ok);
        assert!(response.data.success);
    }

    #[actix_web::test]
    async fn test_add_graph_node_without_app_uri_success() {
        let mut designer_state = DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            tman_internal_config: Arc::new(TmanInternalConfig::default()),
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

        let (graph_id, _) =
            graphs_cache_find_by_name(&designer_state.graphs_cache, "default")
                .unwrap();

        // Add a node to the default graph with the same app URI as other nodes.
        let request_payload = AddGraphNodeRequestPayload {
            graph_id: *graph_id,
            node_name: "test_node".to_string(),
            addon_name: "test_addon".to_string(),
            extension_group_name: None,
            app_uri: None,
            property: None,
        };

        let designer_state = Arc::new(RwLock::new(designer_state));

        let app = test::init_service(
            App::new().app_data(web::Data::new(designer_state)).route(
                "/api/designer/v1/graphs/nodes/add",
                web::post().to(add_graph_node_endpoint),
            ),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/graphs/nodes/add")
            .set_json(request_payload)
            .to_request();

        let resp = test::call_service(&app, req).await;

        let status = resp.status();
        println!("Response status: {}", status);

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();
        println!("Response body: {}", body_str);

        // Should succeed with a 200 OK.
        assert_eq!(status, 200);

        let response: ApiResponse<AddGraphNodeResponsePayload> =
            serde_json::from_str(body_str).unwrap();
        assert_eq!(response.status, Status::Ok);
        assert!(response.data.success);
    }

    #[actix_web::test]
    async fn test_add_graph_node_preserves_field_order() {
        // Create a test property.json file with fields in a specific order
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_dir_path = temp_dir.path().to_str().unwrap().to_string();

        // Read test data from embedded JSON files
        let input_property_json_str = include_str!(
            "test_data_embed/input_property_with_ordered_fields.json"
        );
        let input_manifest_json_str =
            include_str!("test_data_embed/test_app_manifest.json");
        let expected_property = include_str!(
            "test_data_embed/expected_property_with_new_node.json"
        );

        // Write input files to temp directory.
        let property_path =
            Path::new(&temp_dir_path).join(PROPERTY_JSON_FILENAME);
        fs::write(&property_path, input_property_json_str).unwrap();

        let manifest_path = Path::new(&temp_dir_path).join("manifest.json");
        fs::write(&manifest_path, input_manifest_json_str).unwrap();

        // Initialize test state.
        let mut designer_state = DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            tman_internal_config: Arc::new(TmanInternalConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
            graphs_cache: HashMap::new(),
        };

        // Inject the test app into the mock.
        let all_pkgs_json = vec![(
            temp_dir_path.clone(),
            fs::read_to_string(&manifest_path).unwrap(),
            fs::read_to_string(&property_path).unwrap(),
        )];

        let inject_ret = inject_all_pkgs_for_mock(
            &mut designer_state.pkgs_cache,
            &mut designer_state.graphs_cache,
            all_pkgs_json,
        );
        assert!(inject_ret.is_ok());

        let (graph_id, _) = graphs_cache_find_by_name(
            &designer_state.graphs_cache,
            "test_graph",
        )
        .unwrap();

        // Add a node to the test-graph.
        let request_payload = AddGraphNodeRequestPayload {
            graph_id: *graph_id,
            node_name: "new_node".to_string(),
            addon_name: "new_addon".to_string(),
            extension_group_name: None,
            app_uri: None,
            property: None,
        };

        let designer_state = Arc::new(RwLock::new(designer_state));

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(designer_state.clone()))
                .route(
                    "/api/designer/v1/graphs/nodes/add",
                    web::post().to(add_graph_node_endpoint),
                ),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/graphs/nodes/add")
            .set_json(request_payload)
            .to_request();

        let resp = test::call_service(&app, req).await;

        // Should succeed with a 200 OK.
        assert_eq!(resp.status(), 200);

        // Read the updated property.json file.
        let updated_property_content =
            fs::read_to_string(&property_path).unwrap();

        // Parse the contents as JSON for proper comparison.
        let updated_property: Value =
            serde_json::from_str(&updated_property_content).unwrap();
        let expected_property: Value =
            serde_json::from_str(expected_property).unwrap();

        // Compare the updated property with the expected property.
        assert_eq!(
            updated_property, expected_property,
            "Updated property does not match expected property"
        );
    }

    #[actix_web::test]
    async fn test_add_graph_node_with_property_success() {
        // Create a test directory with property.json file.
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_dir_path = temp_dir.path().to_str().unwrap().to_string();

        // Read test data from embedded JSON files.
        let input_property_json_str =
            include_str!("../test_data_embed/app_property.json");
        let input_manifest_json_str =
            include_str!("../test_data_embed/app_manifest.json");

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
            tman_internal_config: Arc::new(TmanInternalConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
            graphs_cache: HashMap::new(),
        };

        // Inject the test app into the mock.
        let all_pkgs_json_str = vec![
            (
                temp_dir_path.clone(),
                input_manifest_json_str.to_string(),
                input_property_json_str.to_string(),
            ),
            (
                format!(
                    "{}{}",
                    temp_dir_path.clone(),
                    "/ten_packages/extension/extension_1"
                ),
                include_str!("test_data_embed/test_addon_manifest.json")
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

        let (graph_id, _) = graphs_cache_find_by_name(
            &designer_state.graphs_cache,
            "default_with_app_uri",
        )
        .unwrap();

        // Add a node to the default graph.
        let add_request_payload = AddGraphNodeRequestPayload {
            graph_id: *graph_id,
            node_name: "test_delete_node".to_string(),
            addon_name: "test_addon".to_string(),
            extension_group_name: None,
            app_uri: Some("http://example.com:8000".to_string()),
            property: Some(serde_json::json!({
                "test_property": "test_value_for_delete"
            })),
        };

        let designer_state = Arc::new(RwLock::new(designer_state));

        // First add a node, then delete it.
        // Setup the add endpoint.
        let app_add = test::init_service(
            App::new()
                .app_data(web::Data::new(designer_state.clone()))
                .route(
                    "/api/designer/v1/graphs/nodes/add",
                    web::post().to(add_graph_node_endpoint),
                ),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/graphs/nodes/add")
            .set_json(add_request_payload)
            .to_request();
        let resp = test::call_service(&app_add, req).await;

        // Ensure add was successful.
        assert_eq!(resp.status(), 200);

        let updated_property_json_str =
            std::fs::read_to_string(&property_path).unwrap();

        let expected_property_json_str =
            include_str!("test_data_embed/expected_property_after_adding_in_test_delete_graph_node_success.json");

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
    async fn test_add_graph_node_with_property_failure() {
        // Create a test directory with property.json file.
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_dir_path = temp_dir.path().to_str().unwrap().to_string();

        // Read test data from embedded JSON files.
        let input_property_json_str =
            include_str!("../test_data_embed/app_property.json");
        let input_manifest_json_str =
            include_str!("../test_data_embed/app_manifest.json");

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
            tman_internal_config: Arc::new(TmanInternalConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
            graphs_cache: HashMap::new(),
        };

        // Inject the test app into the mock.
        let all_pkgs_json_str = vec![
            (
                temp_dir_path.clone(),
                input_manifest_json_str.to_string(),
                input_property_json_str.to_string(),
            ),
            (
                format!(
                    "{}{}",
                    temp_dir_path.clone(),
                    "/ten_packages/extension/extension_1"
                ),
                include_str!("test_data_embed/test_addon_manifest.json")
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

        let (graph_id, _) = graphs_cache_find_by_name(
            &designer_state.graphs_cache,
            "default_with_app_uri",
        )
        .unwrap();

        // Add a node to the default graph.
        let add_request_payload = AddGraphNodeRequestPayload {
            graph_id: *graph_id,
            node_name: "test_delete_node".to_string(),
            addon_name: "test_addon".to_string(),
            extension_group_name: None,
            app_uri: Some("http://example.com:8000".to_string()),
            property: Some(serde_json::json!({
                "test_property": 13
            })),
        };

        let designer_state = Arc::new(RwLock::new(designer_state));

        // First add a node, then delete it.
        // Setup the add endpoint.
        let app_add = test::init_service(
            App::new()
                .app_data(web::Data::new(designer_state.clone()))
                .route(
                    "/api/designer/v1/graphs/nodes/add",
                    web::post().to(add_graph_node_endpoint),
                ),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/graphs/nodes/add")
            .set_json(add_request_payload)
            .to_request();
        let resp = test::call_service(&app_add, req).await;

        // Ensure add was failed.
        assert_eq!(resp.status(), 400);

        let updated_property_str =
            std::fs::read_to_string(&property_path).unwrap();

        // Parse the contents as JSON for proper comparison.
        let updated_property: serde_json::Value =
            serde_json::from_str(&updated_property_str).unwrap();
        let input_property: serde_json::Value =
            serde_json::from_str(input_property_json_str).unwrap();

        // Compare the updated property with the expected property.
        assert_eq!(
            updated_property, input_property,
            "Updated property does not match expected property"
        );
    }
}
