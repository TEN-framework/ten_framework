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
    use ten_manager::{
        config::{internal::TmanInternalConfig, TmanConfig},
        constants::TEST_DIR,
        designer::{
            graphs::nodes::replace::{
                replace_graph_node_endpoint, ReplaceGraphNodeRequestPayload,
                ReplaceGraphNodeResponsePayload,
            },
            response::{ApiResponse, ErrorResponse, Status},
            DesignerState,
        },
        graph::graphs_cache_find_by_name,
        output::TmanOutputCli,
    };
    use ten_rust::pkg_info::constants::{
        MANIFEST_JSON_FILENAME, PROPERTY_JSON_FILENAME,
    };
    use uuid::Uuid;

    use crate::test_case::common::mock::{
        inject_all_pkgs_for_mock, inject_all_standard_pkgs_for_mock,
    };

    #[actix_web::test]
    async fn test_replace_graph_node_invalid_graph() {
        // Setup a designer state.
        let mut designer_state = DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            tman_internal_config: Arc::new(TmanInternalConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
            graphs_cache: HashMap::new(),
        };

        inject_all_standard_pkgs_for_mock(
            &mut designer_state.pkgs_cache,
            &mut designer_state.graphs_cache,
            TEST_DIR,
        );

        let designer_state = Arc::new(RwLock::new(designer_state));

        // Setup test app.
        let app = test::init_service(
            App::new().app_data(web::Data::new(designer_state)).route(
                "/api/designer/v1/graphs/nodes/replace",
                web::post().to(replace_graph_node_endpoint),
            ),
        )
        .await;

        // Try to replace a node in a non-existent graph.
        let request_payload = ReplaceGraphNodeRequestPayload {
            graph_id: Uuid::new_v4(), // Non-existent graph ID.
            name: "test_node".to_string(),
            addon: "test_addon".to_string(),
            app: Some("http://test-app-uri.com".to_string()),
            property: None,
        };

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/graphs/nodes/replace")
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
        assert!(response.message.contains("Graph not found"));
    }

    #[actix_web::test]
    async fn test_replace_graph_node_not_found() {
        // Setup a designer state.
        let mut designer_state = DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            tman_internal_config: Arc::new(TmanInternalConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
            graphs_cache: HashMap::new(),
        };

        inject_all_standard_pkgs_for_mock(
            &mut designer_state.pkgs_cache,
            &mut designer_state.graphs_cache,
            TEST_DIR,
        );

        // Get an existing graph ID.
        let graphs_cache_clone = designer_state.graphs_cache.clone();
        let graph_id = {
            let (id, _) =
                graphs_cache_find_by_name(&graphs_cache_clone, "default")
                    .unwrap();
            *id
        };

        let designer_state = Arc::new(RwLock::new(designer_state));

        // Setup test app.
        let app = test::init_service(
            App::new().app_data(web::Data::new(designer_state)).route(
                "/api/designer/v1/graphs/nodes/replace",
                web::post().to(replace_graph_node_endpoint),
            ),
        )
        .await;

        // Try to replace a non-existent node.
        let request_payload = ReplaceGraphNodeRequestPayload {
            graph_id,
            name: "non_existent_node".to_string(),
            addon: "test_addon".to_string(),
            app: Some("http://example.com:8000".to_string()),
            property: None,
        };

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/graphs/nodes/replace")
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
        assert!(response.message.contains("not found in graph"));
    }

    #[actix_web::test]
    async fn test_replace_graph_node_with_property_success() {
        // Create a test directory with property.json file.
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_dir_path = temp_dir.path().to_str().unwrap().to_string();

        // Read test data from embedded JSON files.
        let input_property_json_str =
            include_str!("../../../../test_data/app_property.json");
        let input_manifest_json_str =
            include_str!("../../../../test_data/app_manifest.json");

        // Write input files to temp directory.
        let property_path =
            std::path::Path::new(&temp_dir_path).join(PROPERTY_JSON_FILENAME);
        std::fs::write(&property_path, input_property_json_str).unwrap();

        let manifest_path =
            std::path::Path::new(&temp_dir_path).join(MANIFEST_JSON_FILENAME);
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
                include_str!("../../../../test_data/test_addon_manifest.json")
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

        // Get an existing graph ID with a node we can replace.
        let graph_id = {
            let (id, _) = graphs_cache_find_by_name(
                &designer_state.graphs_cache,
                "default_with_app_uri",
            )
            .unwrap();
            *id
        };

        let designer_state_arc = Arc::new(RwLock::new(designer_state));

        // Setup test app.
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(designer_state_arc.clone()))
                .route(
                    "/api/designer/v1/graphs/nodes/replace",
                    web::post().to(replace_graph_node_endpoint),
                ),
        )
        .await;

        // Find an existing node name for our test.
        let existing_node_name = {
            let designer_state = designer_state_arc.read().unwrap();
            let graph_info =
                designer_state.graphs_cache.get(&graph_id).unwrap();
            // Assuming there's at least one node in the graph.
            graph_info
                .graph
                .nodes
                .first()
                .unwrap()
                .type_and_name
                .name
                .clone()
        };

        // Try to replace a node with an invalid property (integer instead of
        // string).
        let node_name = existing_node_name;
        let node_addon = "test_addon".to_string();
        let node_app = Some("http://example.com:8000".to_string());
        let node_property = Some(serde_json::json!({
            "test_property": "new_value"
        }));

        let request_payload = ReplaceGraphNodeRequestPayload {
            graph_id,
            name: node_name.clone(),
            addon: node_addon.clone(),
            app: node_app.clone(),
            property: node_property.clone(),
        };

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/graphs/nodes/replace")
            .set_json(request_payload)
            .to_request();
        let resp = test::call_service(&app, req).await;

        // The response should be 200 OK.
        let status = resp.status();
        assert_eq!(status, 200);

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();
        println!("Response status: {}", status);
        println!("Response body: {}", body_str);

        // Read the updated property.json file to verify it hasn't changed.
        let updated_property_content =
            std::fs::read_to_string(&property_path).unwrap();
        let expected_property_content =
            include_str!("../../../../test_data/expected_app_property_1.json");

        // Parse the contents as JSON for proper comparison.
        let updated_property: serde_json::Value =
            serde_json::from_str(&updated_property_content).unwrap();
        let expected_property: serde_json::Value =
            serde_json::from_str(expected_property_content).unwrap();

        println!(
            "Updated property: {}",
            serde_json::to_string_pretty(&updated_property).unwrap()
        );
        println!(
            "Expected property: {}",
            serde_json::to_string_pretty(&expected_property).unwrap()
        );

        // Compare the updated property with the expected property.
        assert_eq!(
            updated_property, expected_property,
            "Property file should not have changed"
        );
    }

    #[actix_web::test]
    async fn test_replace_graph_node_update_fails() {
        // Create a test directory with property.json file.
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_dir_path = temp_dir.path().to_str().unwrap().to_string();

        // Read test data from embedded JSON files.
        let input_property_json_str =
            include_str!("../../../../test_data/app_property.json");
        let input_manifest_json_str =
            include_str!("../../../../test_data/app_manifest.json");

        // Write input files to temp directory.
        let property_path =
            std::path::Path::new(&temp_dir_path).join(PROPERTY_JSON_FILENAME);
        std::fs::write(&property_path, input_property_json_str).unwrap();

        let manifest_path =
            std::path::Path::new(&temp_dir_path).join(MANIFEST_JSON_FILENAME);
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
                include_str!("../../../../test_data/test_addon_manifest.json")
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

        // Get an existing graph ID with a node we can replace.
        let graph_id = {
            let (id, _) = graphs_cache_find_by_name(
                &designer_state.graphs_cache,
                "default_with_app_uri",
            )
            .unwrap();
            *id
        };

        let designer_state_arc = Arc::new(RwLock::new(designer_state));

        // Setup test app.
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(designer_state_arc.clone()))
                .route(
                    "/api/designer/v1/graphs/nodes/replace",
                    web::post().to(replace_graph_node_endpoint),
                ),
        )
        .await;

        // Find an existing node name for our test.
        let existing_node_name = {
            let designer_state = designer_state_arc.read().unwrap();
            let graph_info =
                designer_state.graphs_cache.get(&graph_id).unwrap();
            // Assuming there's at least one node in the graph.
            graph_info
                .graph
                .nodes
                .first()
                .unwrap()
                .type_and_name
                .name
                .clone()
        };

        // Try to replace a node with an invalid property (integer instead of
        // string).
        let node_name = existing_node_name;
        let node_addon = "test_addon".to_string();
        let node_app = Some("http://example.com:8000".to_string());
        let node_property = Some(serde_json::json!({
            "test_property": 13
        }));

        let request_payload = ReplaceGraphNodeRequestPayload {
            graph_id,
            name: node_name.clone(),
            addon: node_addon.clone(),
            app: node_app.clone(),
            property: node_property.clone(),
        };

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/graphs/nodes/replace")
            .set_json(request_payload)
            .to_request();
        let resp = test::call_service(&app, req).await;

        // The response should be 400 Bad Request.
        let status = resp.status();
        assert_eq!(status, 400);

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();
        println!("Response status: {}", status);
        println!("Response body: {}", body_str);

        // The response should indicate failure.
        let response: ErrorResponse = serde_json::from_str(body_str).unwrap();
        assert_eq!(response.status, Status::Fail);
        assert!(
            response
                .message
                .contains("Failed to validate extension property")
                || response.message.contains("Property validation failed")
        );

        // Read the updated property.json file to verify it hasn't changed.
        let updated_property_content =
            std::fs::read_to_string(&property_path).unwrap();

        // Parse the contents as JSON for proper comparison.
        let updated_property: serde_json::Value =
            serde_json::from_str(&updated_property_content).unwrap();
        let input_property: serde_json::Value =
            serde_json::from_str(input_property_json_str).unwrap();

        // Compare the updated property with the input property (should be
        // unchanged).
        assert_eq!(
            updated_property, input_property,
            "Property file should not have changed"
        );
    }

    #[actix_web::test]
    async fn test_replace_graph_node_success() {
        // Setup a designer state.
        let mut designer_state = DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            tman_internal_config: Arc::new(TmanInternalConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
            graphs_cache: HashMap::new(),
        };

        inject_all_standard_pkgs_for_mock(
            &mut designer_state.pkgs_cache,
            &mut designer_state.graphs_cache,
            TEST_DIR,
        );

        // Get an existing graph ID with a node we can replace.
        let graphs_cache_clone = designer_state.graphs_cache.clone();
        let graph_id = {
            let (id, _) = graphs_cache_find_by_name(
                &graphs_cache_clone,
                "default_with_app_uri",
            )
            .unwrap();
            *id
        };

        let designer_state_arc = Arc::new(RwLock::new(designer_state));

        // Setup test app.
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(designer_state_arc.clone()))
                .route(
                    "/api/designer/v1/graphs/nodes/replace",
                    web::post().to(replace_graph_node_endpoint),
                ),
        )
        .await;

        // Find an existing node name and addon for our test.
        let (existing_node_name, existing_app_uri) = {
            let designer_state = designer_state_arc.read().unwrap();
            let graph_info =
                designer_state.graphs_cache.get(&graph_id).unwrap();
            // Assuming there's at least one node in the graph.
            let node = graph_info.graph.nodes.first().unwrap();
            (node.type_and_name.name.clone(), node.app.clone())
        };

        // Store the node details we'll use for replacement.
        let node_name = existing_node_name;
        let app_uri = existing_app_uri;
        let new_addon = "test_addon".to_string();

        // Replace a node with a valid addon and property.
        let request_payload = ReplaceGraphNodeRequestPayload {
            graph_id,
            name: node_name.clone(),
            addon: new_addon.clone(),
            app: app_uri.clone(),
            property: None,
        };

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/graphs/nodes/replace")
            .set_json(request_payload)
            .to_request();
        let resp = test::call_service(&app, req).await;

        // Should succeed with a 200 OK.
        assert_eq!(resp.status(), 200);

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();
        println!("Response body: {}", body_str);

        let response: ApiResponse<ReplaceGraphNodeResponsePayload> =
            serde_json::from_str(body_str).unwrap();
        assert_eq!(response.status, Status::Ok);
        assert!(response.data.success);

        // Verify the node was actually updated.
        let designer_state = designer_state_arc.read().unwrap();
        let graph_info = designer_state.graphs_cache.get(&graph_id).unwrap();
        let updated_node = graph_info
            .graph
            .nodes
            .iter()
            .find(|node| {
                node.type_and_name.name == node_name && node.app == app_uri
            })
            .unwrap();

        assert_eq!(updated_node.addon, new_addon);
    }
}
