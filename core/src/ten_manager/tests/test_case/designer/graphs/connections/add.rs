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
        config::TmanConfig,
        designer::{
            graphs::connections::add::{
                add_graph_connection_endpoint,
                AddGraphConnectionRequestPayload,
                AddGraphConnectionResponsePayload,
            },
            response::ApiResponse,
            DesignerState,
        },
        graph::graphs_cache_find_by_name,
        output::TmanOutputCli,
    };
    use ten_rust::pkg_info::{
        constants::PROPERTY_JSON_FILENAME, message::MsgType,
    };
    use uuid::Uuid;

    use crate::test_case::mock::inject_all_pkgs_for_mock;

    #[actix_web::test]
    async fn test_add_graph_connection_success_1() {
        let mut designer_state = DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
            graphs_cache: HashMap::new(),
        };

        // Create a temporary directory for our test to store the generated
        // property.json.
        let temp_dir = tempfile::tempdir().unwrap();
        let test_dir = temp_dir.path().to_str().unwrap().to_string();

        // Load both the app package JSON and extension addon package JSONs.
        let app_manifest_json_str =
            include_str!("../test_data_embed/app_manifest.json").to_string();
        let app_property_json_str =
            include_str!("../test_data_embed/app_property.json").to_string();

        // Create the property.json file in the temporary directory.
        let property_path =
            std::path::Path::new(&test_dir).join(PROPERTY_JSON_FILENAME);
        std::fs::write(&property_path, &app_property_json_str).unwrap();

        // Create extension addon manifest strings.
        let ext1_manifest = r#"{
            "type": "extension",
            "name": "extension_1",
            "version": "0.1.0"
        }"#
        .to_string();

        let ext2_manifest = r#"{
            "type": "extension",
            "name": "extension_2",
            "version": "0.1.0"
        }"#
        .to_string();

        let ext3_manifest = r#"{
            "type": "extension",
            "name": "extension_3",
            "version": "0.1.0"
        }"#
        .to_string();

        // The empty property for addons
        let empty_property = r#"{"_ten":{}}"#.to_string();

        let all_pkgs_json = vec![
            (
                test_dir.clone(),
                app_manifest_json_str,
                app_property_json_str,
            ),
            (
                format!(
                    "{}{}",
                    test_dir.clone(),
                    "/ten_packages/extension/extension_1"
                ),
                ext1_manifest,
                empty_property.clone(),
            ),
            (
                format!(
                    "{}{}",
                    test_dir.clone(),
                    "/ten_packages/extension/extension_2"
                ),
                ext2_manifest,
                empty_property.clone(),
            ),
            (
                format!(
                    "{}{}",
                    test_dir.clone(),
                    "/ten_packages/extension/extension_3"
                ),
                ext3_manifest,
                empty_property.clone(),
            ),
        ];

        let inject_ret = inject_all_pkgs_for_mock(
            &mut designer_state.pkgs_cache,
            &mut designer_state.graphs_cache,
            all_pkgs_json,
        );
        assert!(inject_ret.is_ok());

        let (graph_id, _) = graphs_cache_find_by_name(
            &designer_state.graphs_cache,
            "default_with_app_uri",
        )
        .unwrap();

        // Add a connection between existing nodes in the default graph.
        // Use "http://example.com:8000" for both src_app and dest_app to match the test data.
        let request_payload = AddGraphConnectionRequestPayload {
            graph_id: *graph_id,
            src_app: Some("http://example.com:8000".to_string()),
            src_extension: "extension_1".to_string(),
            msg_type: MsgType::Cmd,
            msg_name: "test_cmd".to_string(),
            dest_app: Some("http://example.com:8000".to_string()),
            dest_extension: "extension_2".to_string(),
            msg_conversion: None,
        };

        let designer_state = Arc::new(RwLock::new(designer_state));

        let app = test::init_service(
            App::new().app_data(web::Data::new(designer_state)).route(
                "/api/designer/v1/graphs/connections/add",
                web::post().to(add_graph_connection_endpoint),
            ),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/graphs/connections/add")
            .set_json(request_payload)
            .to_request();
        let resp = test::call_service(&app, req).await;

        // Print the status and body for debugging.
        let status = resp.status();
        println!("Response status: {:?}", status);
        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();
        println!("Response body: {}", body_str);

        assert!(status.is_success());

        let response: ApiResponse<AddGraphConnectionResponsePayload> =
            serde_json::from_str(body_str).unwrap();

        assert!(response.data.success);

        // Define expected property.json content after adding the connection.
        let expected_property_json_str = include_str!("test_data_embed/expected_json__test_add_graph_connection_success.json");

        // Read the actual property.json file generated during the test.
        let property_path =
            std::path::Path::new(&test_dir).join(PROPERTY_JSON_FILENAME);
        let actual_property = std::fs::read_to_string(property_path).unwrap();

        // Normalize both JSON strings to handle formatting differences.
        let expected_value: serde_json::Value =
            serde_json::from_str(expected_property_json_str).unwrap();
        let actual_value: serde_json::Value =
            serde_json::from_str(&actual_property).unwrap();

        // Compare the normalized JSON values.
        assert_eq!(
            expected_value, actual_value,
            "Property file doesn't match expected content.\nExpected:\n{}\nActual:\n{}",
            serde_json::to_string_pretty(&expected_value).unwrap(),
            serde_json::to_string_pretty(&actual_value).unwrap()
        );
    }

    #[actix_web::test]
    async fn test_add_graph_connection_success_2() {
        let mut designer_state = DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
            graphs_cache: HashMap::new(),
        };

        // Create a temporary directory for our test to store the generated
        // property.json.
        let temp_dir = tempfile::tempdir().unwrap();
        let test_dir = temp_dir.path().to_str().unwrap().to_string();

        // Load both the app package JSON and extension addon package JSONs.
        let app_manifest_json_str =
            include_str!("../test_data_embed/app_manifest.json").to_string();
        let app_property_json_str = include_str!(
            "test_data_embed/initial_property_add_connection_2.json"
        )
        .to_string();

        // Create the property.json file in the temporary directory.
        let property_path =
            std::path::Path::new(&test_dir).join(PROPERTY_JSON_FILENAME);
        std::fs::write(&property_path, &app_property_json_str).unwrap();

        // Create extension addon manifest strings.
        let ext1_manifest = r#"{
            "type": "extension",
            "name": "aio_http_server_python",
            "version": "0.1.0"
        }"#
        .to_string();

        let ext2_manifest = r#"{
            "type": "extension",
            "name": "simple_echo_cpp",
            "version": "0.1.0"
        }"#
        .to_string();

        let ext3_manifest = r#"{
            "type": "extension",
            "name": "mock_extension_1",
            "version": "0.1.0"
        }"#
        .to_string();

        let ext4_manifest = r#"{
          "type": "extension",
          "name": "mock_extension_2",
          "version": "0.1.0"
        }"#
        .to_string();

        // The empty property for addons
        let empty_property = r#"{"_ten":{}}"#.to_string();

        let all_pkgs_json = vec![
            (
                test_dir.clone(),
                app_manifest_json_str,
                app_property_json_str,
            ),
            (
                format!(
                    "{}{}",
                    test_dir.clone(),
                    "/ten_packages/extension/extension_1"
                ),
                ext1_manifest,
                empty_property.clone(),
            ),
            (
                format!(
                    "{}{}",
                    test_dir.clone(),
                    "/ten_packages/extension/extension_2"
                ),
                ext2_manifest,
                empty_property.clone(),
            ),
            (
                format!(
                    "{}{}",
                    test_dir.clone(),
                    "/ten_packages/extension/extension_3"
                ),
                ext3_manifest,
                empty_property.clone(),
            ),
            (
                format!(
                    "{}{}",
                    test_dir.clone(),
                    "/ten_packages/extension/extension_4"
                ),
                ext4_manifest,
                empty_property.clone(),
            ),
        ];

        let inject_ret = inject_all_pkgs_for_mock(
            &mut designer_state.pkgs_cache,
            &mut designer_state.graphs_cache,
            all_pkgs_json,
        );
        assert!(inject_ret.is_ok());

        let (graph_id, _) =
            graphs_cache_find_by_name(&designer_state.graphs_cache, "default")
                .unwrap();

        // Add a connection between existing nodes in the default graph.
        let request_payload = AddGraphConnectionRequestPayload {
            graph_id: *graph_id,
            src_app: None,
            src_extension: "aio_http_server_python".to_string(),
            msg_type: MsgType::Cmd,
            msg_name: "c1".to_string(),
            dest_app: None,
            dest_extension: "simple_echo_cpp".to_string(),
            msg_conversion: None,
        };

        let designer_state = Arc::new(RwLock::new(designer_state));

        let app = test::init_service(
            App::new().app_data(web::Data::new(designer_state)).route(
                "/api/designer/v1/graphs/connections/add",
                web::post().to(add_graph_connection_endpoint),
            ),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/graphs/connections/add")
            .set_json(request_payload)
            .to_request();
        let resp = test::call_service(&app, req).await;

        // Print the status and body for debugging.
        let status = resp.status();
        println!("Response status: {:?}", status);
        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();
        println!("Response body: {}", body_str);

        assert!(status.is_success());

        let response: ApiResponse<AddGraphConnectionResponsePayload> =
            serde_json::from_str(body_str).unwrap();

        assert!(response.data.success);

        // Define expected property.json content after adding the connection.
        let expected_property_json_str = include_str!(
            "test_data_embed/expected_property_add_connection_2.json"
        );

        // Read the actual property.json file generated during the test.
        let property_path =
            std::path::Path::new(&test_dir).join(PROPERTY_JSON_FILENAME);
        let actual_property = std::fs::read_to_string(property_path).unwrap();

        // Normalize both JSON strings to handle formatting differences.
        let expected_value: serde_json::Value =
            serde_json::from_str(expected_property_json_str).unwrap();
        let actual_value: serde_json::Value =
            serde_json::from_str(&actual_property).unwrap();

        // Compare the normalized JSON values.
        assert_eq!(
            expected_value, actual_value,
            "Property file doesn't match expected content.\nExpected:\n{}\nActual:\n{}",
            serde_json::to_string_pretty(&expected_value).unwrap(),
            serde_json::to_string_pretty(&actual_value).unwrap()
        );
    }

    #[actix_web::test]
    async fn test_add_graph_connection_invalid_graph() {
        let mut designer_state = DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
            graphs_cache: HashMap::new(),
        };

        // Create a temporary directory for our test to store the generated
        // property.json.
        let temp_dir = tempfile::tempdir().unwrap();
        let test_dir = temp_dir.path().to_str().unwrap().to_string();

        let all_pkgs_json_str = vec![(
            test_dir.clone(),
            include_str!("../test_data_embed/app_manifest.json").to_string(),
            include_str!("../test_data_embed/app_property.json").to_string(),
        )];

        // Create the property.json file in the temporary directory.
        let property_path =
            std::path::Path::new(&test_dir).join(PROPERTY_JSON_FILENAME);
        std::fs::write(
            &property_path,
            include_str!("../test_data_embed/app_property.json"),
        )
        .unwrap();

        let inject_ret = inject_all_pkgs_for_mock(
            &mut designer_state.pkgs_cache,
            &mut designer_state.graphs_cache,
            all_pkgs_json_str,
        );
        assert!(inject_ret.is_ok());

        let designer_state = Arc::new(RwLock::new(designer_state));

        let app = test::init_service(
            App::new().app_data(web::Data::new(designer_state)).route(
                "/api/designer/v1/graphs/connections/add",
                web::post().to(add_graph_connection_endpoint),
            ),
        )
        .await;

        // Try to add a connection to a non-existent graph.
        let request_payload = AddGraphConnectionRequestPayload {
            graph_id: Uuid::new_v4(),
            src_app: None,
            src_extension: "extension_1".to_string(),
            msg_type: MsgType::Cmd,
            msg_name: "test_cmd".to_string(),
            dest_app: None,
            dest_extension: "extension_2".to_string(),
            msg_conversion: None,
        };

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/graphs/connections/add")
            .set_json(request_payload)
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), 404);
    }

    #[actix_web::test]
    async fn test_add_graph_connection_preserves_order() {
        let mut designer_state = DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
            graphs_cache: HashMap::new(),
        };

        // Create a temporary directory for our test to store the generated
        // property.json.
        let temp_dir = tempfile::tempdir().unwrap();
        let test_dir = temp_dir.path().to_str().unwrap().to_string();

        // Load both the app package JSON and extension addon package JSONs.
        let app_manifest_json_str =
            include_str!("../test_data_embed/app_manifest.json").to_string();
        let app_property_json_str =
            include_str!("../test_data_embed/app_property.json").to_string();

        // Create the property.json file in the temporary directory.
        let property_path =
            std::path::Path::new(&test_dir).join(PROPERTY_JSON_FILENAME);
        std::fs::write(&property_path, &app_property_json_str).unwrap();

        // Create extension addon manifest strings.
        let ext1_manifest = r#"{
            "type": "extension",
            "name": "extension_1",
            "version": "0.1.0"
        }"#
        .to_string();

        let ext2_manifest = r#"{
            "type": "extension",
            "name": "extension_2",
            "version": "0.1.0"
        }"#
        .to_string();

        let ext3_manifest = r#"{
            "type": "extension",
            "name": "extension_3",
            "version": "0.1.0"
        }"#
        .to_string();

        // The empty property for addons.
        let empty_property = r#"{"_ten":{}}"#.to_string();

        let all_pkgs_json = vec![
            (
                test_dir.clone(),
                app_manifest_json_str,
                app_property_json_str,
            ),
            (
                format!(
                    "{}{}",
                    test_dir.clone(),
                    "/ten_packages/extension/extension_1"
                ),
                ext1_manifest,
                empty_property.clone(),
            ),
            (
                format!(
                    "{}{}",
                    test_dir.clone(),
                    "/ten_packages/extension/extension_2"
                ),
                ext2_manifest,
                empty_property.clone(),
            ),
            (
                format!(
                    "{}{}",
                    test_dir.clone(),
                    "/ten_packages/extension/extension_3"
                ),
                ext3_manifest,
                empty_property.clone(),
            ),
        ];

        let inject_ret = inject_all_pkgs_for_mock(
            &mut designer_state.pkgs_cache,
            &mut designer_state.graphs_cache,
            all_pkgs_json,
        );
        assert!(inject_ret.is_ok());

        let (graph_id, _) = graphs_cache_find_by_name(
            &designer_state.graphs_cache,
            "default_with_app_uri",
        )
        .unwrap();

        // Add first connection.
        let request_payload1 = AddGraphConnectionRequestPayload {
            graph_id: *graph_id,
            src_app: Some("http://example.com:8000".to_string()),
            src_extension: "extension_1".to_string(),
            msg_type: MsgType::Cmd,
            msg_name: "test_cmd1".to_string(),
            dest_app: Some("http://example.com:8000".to_string()),
            dest_extension: "extension_2".to_string(),
            msg_conversion: None,
        };

        // Add second connection to create a sequence.
        let request_payload2 = AddGraphConnectionRequestPayload {
            graph_id: *graph_id,
            src_app: Some("http://example.com:8000".to_string()),
            src_extension: "extension_1".to_string(),
            msg_type: MsgType::Cmd,
            msg_name: "test_cmd2".to_string(),
            dest_app: Some("http://example.com:8000".to_string()),
            dest_extension: "extension_3".to_string(),
            msg_conversion: None,
        };

        let designer_state_arc = Arc::new(RwLock::new(designer_state));

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(designer_state_arc.clone()))
                .route(
                    "/api/designer/v1/graphs/connections/add",
                    web::post().to(add_graph_connection_endpoint),
                ),
        )
        .await;

        let req1 = test::TestRequest::post()
            .uri("/api/designer/v1/graphs/connections/add")
            .set_json(request_payload1)
            .to_request();
        let resp1 = test::call_service(&app, req1).await;

        assert!(resp1.status().is_success());

        let req2 = test::TestRequest::post()
            .uri("/api/designer/v1/graphs/connections/add")
            .set_json(request_payload2)
            .to_request();
        let resp2 = test::call_service(&app, req2).await;

        assert!(resp2.status().is_success());

        // Define expected property.json content after adding both connections.
        let expected_property_json_str = include_str!("test_data_embed/expected_json__test_add_graph_connection_preserves_order.json");

        // Read the actual property.json file generated during the test.
        let property_path =
            std::path::Path::new(&test_dir).join(PROPERTY_JSON_FILENAME);
        let actual_property = std::fs::read_to_string(property_path).unwrap();

        // Normalize both JSON strings to handle formatting differences.
        let expected_value: serde_json::Value =
            serde_json::from_str(expected_property_json_str).unwrap();
        let actual_value: serde_json::Value =
            serde_json::from_str(&actual_property).unwrap();

        // Compare the normalized JSON values.
        assert_eq!(
            expected_value, actual_value,
            "Property file doesn't match expected content.\nExpected:\n{}\nActual:\n{}",
            serde_json::to_string_pretty(&expected_value).unwrap(),
            serde_json::to_string_pretty(&actual_value).unwrap()
        );
    }

    #[actix_web::test]
    async fn test_add_graph_connection_file_comparison() {
        let mut designer_state = DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
            graphs_cache: HashMap::new(),
        };

        // Create a temporary directory for our test to store the generated
        // property.json.
        let temp_dir = tempfile::tempdir().unwrap();
        let test_dir = temp_dir.path().to_str().unwrap().to_string();

        // Load both the app package JSON and extension addon package JSONs.
        let app_manifest_json_str =
            include_str!("../test_data_embed/app_manifest.json").to_string();
        let app_property_json_str =
            include_str!("../test_data_embed/app_property.json").to_string();

        // Create the property.json file in the temporary directory.
        let property_path =
            std::path::Path::new(&test_dir).join(PROPERTY_JSON_FILENAME);
        std::fs::write(&property_path, &app_property_json_str).unwrap();

        // Create extension addon manifest strings.
        let ext1_manifest = r#"{
            "type": "extension",
            "name": "extension_1",
            "version": "0.1.0"
        }"#
        .to_string();

        let ext2_manifest = r#"{
            "type": "extension",
            "name": "extension_2",
            "version": "0.1.0"
        }"#
        .to_string();

        let ext3_manifest = r#"{
            "type": "extension",
            "name": "extension_3",
            "version": "0.1.0"
        }"#
        .to_string();

        // The empty property for addons.
        let empty_property = r#"{"_ten":{}}"#.to_string();

        let all_pkgs_json = vec![
            (
                test_dir.clone(),
                app_manifest_json_str,
                app_property_json_str,
            ),
            (
                format!(
                    "{}{}",
                    test_dir.clone(),
                    "/ten_packages/extension/extension_1"
                ),
                ext1_manifest,
                empty_property.clone(),
            ),
            (
                format!(
                    "{}{}",
                    test_dir.clone(),
                    "/ten_packages/extension/extension_2"
                ),
                ext2_manifest,
                empty_property.clone(),
            ),
            (
                format!(
                    "{}{}",
                    test_dir.clone(),
                    "/ten_packages/extension/extension_3"
                ),
                ext3_manifest,
                empty_property.clone(),
            ),
        ];

        let inject_ret = inject_all_pkgs_for_mock(
            &mut designer_state.pkgs_cache,
            &mut designer_state.graphs_cache,
            all_pkgs_json,
        );
        assert!(inject_ret.is_ok());

        let (graph_id, _) = graphs_cache_find_by_name(
            &designer_state.graphs_cache,
            "default_with_app_uri",
        )
        .unwrap();

        // Add first connection.
        let request_payload1 = AddGraphConnectionRequestPayload {
            graph_id: *graph_id,
            src_app: Some("http://example.com:8000".to_string()),
            src_extension: "extension_1".to_string(),
            msg_type: MsgType::Cmd,
            msg_name: "test_cmd1".to_string(),
            dest_app: Some("http://example.com:8000".to_string()),
            dest_extension: "extension_2".to_string(),
            msg_conversion: None,
        };

        // Add second connection to create a sequence.
        let request_payload2 = AddGraphConnectionRequestPayload {
            graph_id: *graph_id,
            src_app: Some("http://example.com:8000".to_string()),
            src_extension: "extension_1".to_string(),
            msg_type: MsgType::Cmd,
            msg_name: "test_cmd2".to_string(),
            dest_app: Some("http://example.com:8000".to_string()),
            dest_extension: "extension_3".to_string(),
            msg_conversion: None,
        };

        let designer_state_arc = Arc::new(RwLock::new(designer_state));

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(designer_state_arc.clone()))
                .route(
                    "/api/designer/v1/graphs/connections/add",
                    web::post().to(add_graph_connection_endpoint),
                ),
        )
        .await;

        let req1 = test::TestRequest::post()
            .uri("/api/designer/v1/graphs/connections/add")
            .set_json(request_payload1)
            .to_request();
        let resp1 = test::call_service(&app, req1).await;

        assert!(resp1.status().is_success());

        let req2 = test::TestRequest::post()
            .uri("/api/designer/v1/graphs/connections/add")
            .set_json(request_payload2)
            .to_request();
        let resp2 = test::call_service(&app, req2).await;

        assert!(resp2.status().is_success());

        // Define expected property.json content after adding both connections.
        let expected_property_json_str = include_str!("test_data_embed/expected_json__test_add_graph_connection_file_comparison.json");

        // Read the actual property.json file generated during the test.
        let property_path =
            std::path::Path::new(&test_dir).join(PROPERTY_JSON_FILENAME);
        let actual_property = std::fs::read_to_string(property_path).unwrap();

        // Normalize both JSON strings to handle formatting differences
        let expected_value: serde_json::Value =
            serde_json::from_str(expected_property_json_str).unwrap();
        let actual_value: serde_json::Value =
            serde_json::from_str(&actual_property).unwrap();

        // Compare the normalized JSON values
        assert_eq!(
            expected_value, actual_value,
            "Property file doesn't match expected content.\nExpected:\n{}\nActual:\n{}",
            serde_json::to_string_pretty(&expected_value).unwrap(),
            serde_json::to_string_pretty(&actual_value).unwrap()
        );
    }

    #[actix_web::test]
    async fn test_add_graph_connection_data_type() {
        let mut designer_state = DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
            graphs_cache: HashMap::new(),
        };

        // Create a temporary directory for our test to store the generated
        // property.json.
        let temp_dir = tempfile::tempdir().unwrap();
        let test_dir = temp_dir.path().to_str().unwrap().to_string();

        // Load both the app package JSON and extension addon package JSONs.
        let app_manifest_json_str =
            include_str!("../test_data_embed/app_manifest.json").to_string();
        let app_property_json_str =
            include_str!("../test_data_embed/app_property.json").to_string();

        // Create the property.json file in the temporary directory.
        let property_path =
            std::path::Path::new(&test_dir).join(PROPERTY_JSON_FILENAME);
        std::fs::write(&property_path, &app_property_json_str).unwrap();

        // Create extension addon manifest strings.
        let ext1_manifest = r#"{
            "type": "extension",
            "name": "extension_1",
            "version": "0.1.0"
        }"#
        .to_string();

        let ext2_manifest = r#"{
            "type": "extension",
            "name": "extension_2",
            "version": "0.1.0"
        }"#
        .to_string();

        // The empty property for addons.
        let empty_property = r#"{"_ten":{}}"#.to_string();

        let all_pkgs_json = vec![
            (
                test_dir.clone(),
                app_manifest_json_str,
                app_property_json_str,
            ),
            (
                format!(
                    "{}{}",
                    test_dir.clone(),
                    "/ten_packages/extension/extension_1"
                ),
                ext1_manifest,
                empty_property.clone(),
            ),
            (
                format!(
                    "{}{}",
                    test_dir.clone(),
                    "/ten_packages/extension/extension_2"
                ),
                ext2_manifest,
                empty_property,
            ),
        ];

        let inject_ret = inject_all_pkgs_for_mock(
            &mut designer_state.pkgs_cache,
            &mut designer_state.graphs_cache,
            all_pkgs_json,
        );
        assert!(inject_ret.is_ok());

        let (graph_id, _) = graphs_cache_find_by_name(
            &designer_state.graphs_cache,
            "default_with_app_uri",
        )
        .unwrap();

        // Add a DATA type connection between extensions.
        let request_payload = AddGraphConnectionRequestPayload {
            graph_id: *graph_id,
            src_app: Some("http://example.com:8000".to_string()),
            src_extension: "extension_1".to_string(),
            msg_type: MsgType::Data,
            msg_name: "test_data".to_string(),
            dest_app: Some("http://example.com:8000".to_string()),
            dest_extension: "extension_2".to_string(),
            msg_conversion: None,
        };

        let designer_state_arc = Arc::new(RwLock::new(designer_state));

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(designer_state_arc.clone()))
                .route(
                    "/api/designer/v1/graphs/connections/add",
                    web::post().to(add_graph_connection_endpoint),
                ),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/graphs/connections/add")
            .set_json(request_payload)
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();
        let response: ApiResponse<AddGraphConnectionResponsePayload> =
            serde_json::from_str(body_str).unwrap();
        assert!(response.data.success);

        // Define expected property.json content after adding the connection.
        let expected_property_json_str = include_str!("test_data_embed/expected_json__test_add_graph_connection_data_type.json");

        // Read the actual property.json file generated during the test.
        let property_path =
            std::path::Path::new(&test_dir).join(PROPERTY_JSON_FILENAME);
        let actual_property = std::fs::read_to_string(property_path).unwrap();

        // Normalize both JSON strings to handle formatting differences.
        let expected_value: serde_json::Value =
            serde_json::from_str(expected_property_json_str).unwrap();
        let actual_value: serde_json::Value =
            serde_json::from_str(&actual_property).unwrap();

        // Compare the normalized JSON values.
        assert_eq!(
            expected_value, actual_value,
            "Property file doesn't match expected content.\nExpected:\n{}\nActual:\n{}",
            serde_json::to_string_pretty(&expected_value).unwrap(),
            serde_json::to_string_pretty(&actual_value).unwrap()
        );
    }

    #[actix_web::test]
    async fn test_add_graph_connection_frame_types() {
        let mut designer_state = DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
            graphs_cache: HashMap::new(),
        };

        // Create a temporary directory for our test to store the generated
        // property.json.
        let temp_dir = tempfile::tempdir().unwrap();
        let test_dir = temp_dir.path().to_str().unwrap().to_string();

        // Load both the app package JSON and extension addon package JSONs.
        let app_manifest_json_str =
            include_str!("../test_data_embed/app_manifest.json").to_string();
        let app_property_json_str =
            include_str!("../test_data_embed/app_property.json").to_string();

        // Create the property.json file in the temporary directory.
        let property_path =
            std::path::Path::new(&test_dir).join(PROPERTY_JSON_FILENAME);
        std::fs::write(&property_path, &app_property_json_str).unwrap();

        // Create extension addon manifest strings.
        let ext1_manifest = r#"{
            "type": "extension",
            "name": "extension_1",
            "version": "0.1.0"
        }"#
        .to_string();

        let ext2_manifest = r#"{
            "type": "extension",
            "name": "extension_2",
            "version": "0.1.0"
        }"#
        .to_string();

        // The empty property for addons.
        let empty_property = r#"{"_ten":{}}"#.to_string();

        let all_pkgs_json = vec![
            (
                test_dir.clone(),
                app_manifest_json_str,
                app_property_json_str,
            ),
            (
                format!(
                    "{}{}",
                    test_dir.clone(),
                    "/ten_packages/extension/extension_1"
                ),
                ext1_manifest,
                empty_property.clone(),
            ),
            (
                format!(
                    "{}{}",
                    test_dir.clone(),
                    "/ten_packages/extension/extension_2"
                ),
                ext2_manifest,
                empty_property,
            ),
        ];

        let inject_ret = inject_all_pkgs_for_mock(
            &mut designer_state.pkgs_cache,
            &mut designer_state.graphs_cache,
            all_pkgs_json,
        );
        assert!(inject_ret.is_ok());

        let (graph_id, _) = graphs_cache_find_by_name(
            &designer_state.graphs_cache,
            "default_with_app_uri",
        )
        .unwrap();

        // First add an AUDIO_FRAME type connection.
        let audio_request = AddGraphConnectionRequestPayload {
            graph_id: *graph_id,
            src_app: Some("http://example.com:8000".to_string()),
            src_extension: "extension_1".to_string(),
            msg_type: MsgType::AudioFrame,
            msg_name: "audio_stream".to_string(),
            dest_app: Some("http://example.com:8000".to_string()),
            dest_extension: "extension_2".to_string(),
            msg_conversion: None,
        };

        // Then add a VIDEO_FRAME type connection.
        let video_request = AddGraphConnectionRequestPayload {
            graph_id: *graph_id,
            src_app: Some("http://example.com:8000".to_string()),
            src_extension: "extension_1".to_string(),
            msg_type: MsgType::VideoFrame,
            msg_name: "video_stream".to_string(),
            dest_app: Some("http://example.com:8000".to_string()),
            dest_extension: "extension_2".to_string(),
            msg_conversion: None,
        };

        let designer_state_arc = Arc::new(RwLock::new(designer_state));

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(designer_state_arc.clone()))
                .route(
                    "/api/designer/v1/graphs/connections/add",
                    web::post().to(add_graph_connection_endpoint),
                ),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/graphs/connections/add")
            .set_json(audio_request)
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/graphs/connections/add")
            .set_json(video_request)
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());

        // Define expected property.json content after adding both connections.
        let expected_property_json_str = include_str!("test_data_embed/expected_json__test_add_graph_connection_frame_types.json");

        // Read the actual property.json file generated during the test.
        let property_path =
            std::path::Path::new(&test_dir).join(PROPERTY_JSON_FILENAME);
        let actual_property = std::fs::read_to_string(property_path).unwrap();

        // Normalize both JSON strings to handle formatting differences.
        let expected_value: serde_json::Value =
            serde_json::from_str(expected_property_json_str).unwrap();
        let actual_value: serde_json::Value =
            serde_json::from_str(&actual_property).unwrap();

        // Compare the normalized JSON values.
        assert_eq!(
            expected_value, actual_value,
            "Property file doesn't match expected content.\nExpected:\n{}\nActual:\n{}",
            serde_json::to_string_pretty(&expected_value).unwrap(),
            serde_json::to_string_pretty(&actual_value).unwrap()
        );
    }

    #[actix_web::test]
    async fn test_add_multiple_connections_preservation_order() {
        let mut designer_state = DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
            graphs_cache: HashMap::new(),
        };

        // Create a temporary directory for our test to store the generated
        // property.json.
        let temp_dir = tempfile::tempdir().unwrap();
        let test_dir = temp_dir.path().to_str().unwrap().to_string();

        // Load both the app package JSON and extension addon package JSONs.
        let app_manifest_json_str =
            include_str!("../test_data_embed/app_manifest.json").to_string();
        let app_property_json_str =
            include_str!("../test_data_embed/app_property.json").to_string();

        // Create the property.json file in the temporary directory.
        let property_path =
            std::path::Path::new(&test_dir).join(PROPERTY_JSON_FILENAME);
        std::fs::write(&property_path, &app_property_json_str).unwrap();

        // Create three extension addon manifest strings.
        let ext1_manifest = r#"{
            "type": "extension",
            "name": "extension_1",
            "version": "0.1.0"
        }"#
        .to_string();

        let ext2_manifest = r#"{
            "type": "extension",
            "name": "extension_2",
            "version": "0.1.0"
        }"#
        .to_string();

        let ext3_manifest = r#"{
            "type": "extension",
            "name": "extension_3",
            "version": "0.1.0"
        }"#
        .to_string();

        // The empty property for addons.
        let empty_property = r#"{"_ten":{}}"#.to_string();

        let all_pkgs_json = vec![
            (
                test_dir.clone(),
                app_manifest_json_str,
                app_property_json_str,
            ),
            (
                format!(
                    "{}{}",
                    test_dir.clone(),
                    "/ten_packages/extension/extension_1"
                ),
                ext1_manifest,
                empty_property.clone(),
            ),
            (
                format!(
                    "{}{}",
                    test_dir.clone(),
                    "/ten_packages/extension/extension_2"
                ),
                ext2_manifest,
                empty_property.clone(),
            ),
            (
                format!(
                    "{}{}",
                    test_dir.clone(),
                    "/ten_packages/extension/extension_3"
                ),
                ext3_manifest,
                empty_property,
            ),
        ];

        let inject_ret = inject_all_pkgs_for_mock(
            &mut designer_state.pkgs_cache,
            &mut designer_state.graphs_cache,
            all_pkgs_json,
        );
        assert!(inject_ret.is_ok());

        let (graph_id, _) = graphs_cache_find_by_name(
            &designer_state.graphs_cache,
            "default_with_app_uri",
        )
        .unwrap();

        // Add first command.
        let cmd1_request = AddGraphConnectionRequestPayload {
            graph_id: *graph_id,
            src_app: Some("http://example.com:8000".to_string()),
            src_extension: "extension_1".to_string(),
            msg_type: MsgType::Cmd,
            msg_name: "cmd_1".to_string(),
            dest_app: Some("http://example.com:8000".to_string()),
            dest_extension: "extension_2".to_string(),
            msg_conversion: None,
        };

        // Add second command.
        let cmd2_request = AddGraphConnectionRequestPayload {
            graph_id: *graph_id,
            src_app: Some("http://example.com:8000".to_string()),
            src_extension: "extension_1".to_string(),
            msg_type: MsgType::Cmd,
            msg_name: "cmd_2".to_string(),
            dest_app: Some("http://example.com:8000".to_string()),
            dest_extension: "extension_3".to_string(),
            msg_conversion: None,
        };

        // Add third command.
        let cmd3_request = AddGraphConnectionRequestPayload {
            graph_id: *graph_id,
            src_app: Some("http://example.com:8000".to_string()),
            src_extension: "extension_1".to_string(),
            msg_type: MsgType::Cmd,
            msg_name: "cmd_3".to_string(),
            dest_app: Some("http://example.com:8000".to_string()),
            dest_extension: "extension_2".to_string(),
            msg_conversion: None,
        };

        let designer_state_arc = Arc::new(RwLock::new(designer_state));

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(designer_state_arc.clone()))
                .route(
                    "/api/designer/v1/graphs/connections/add",
                    web::post().to(add_graph_connection_endpoint),
                ),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/graphs/connections/add")
            .set_json(cmd1_request)
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/graphs/connections/add")
            .set_json(cmd2_request)
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/graphs/connections/add")
            .set_json(cmd3_request)
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());

        // Define expected property.json content after adding all three
        // connections.
        let expected_property_json_str = include_str!("test_data_embed/expected_json__test_add_multiple_connections_preservation_order.json");

        // Read the actual property.json file generated during the test.
        let property_path =
            std::path::Path::new(&test_dir).join(PROPERTY_JSON_FILENAME);
        let actual_property = std::fs::read_to_string(property_path).unwrap();

        // Normalize both JSON strings to handle formatting differences.
        let expected_value: serde_json::Value =
            serde_json::from_str(expected_property_json_str).unwrap();
        let actual_value: serde_json::Value =
            serde_json::from_str(&actual_property).unwrap();

        // Compare the normalized JSON values.
        assert_eq!(
            expected_value, actual_value,
            "Property file doesn't match expected content.\nExpected:\n{}\nActual:\n{}",
            serde_json::to_string_pretty(&expected_value).unwrap(),
            serde_json::to_string_pretty(&actual_value).unwrap()
        );
    }
}
