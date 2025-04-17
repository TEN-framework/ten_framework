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
        designer::{
            graphs::update::{
                update_graph_endpoint, GraphNodeForUpdate,
                UpdateGraphRequestPayload,
            },
            response::{ErrorResponse, Status},
            DesignerState,
        },
        graph::graphs_cache_find_by_name,
        output::TmanOutputCli,
    };
    use ten_rust::{
        graph::{
            connection::{GraphConnection, GraphDestination, GraphMessageFlow},
            node::GraphNode,
        },
        pkg_info::{
            constants::PROPERTY_JSON_FILENAME, pkg_type::PkgType,
            pkg_type_and_name::PkgTypeAndName,
        },
    };
    use uuid::Uuid;

    use crate::test_case::mock::inject_all_pkgs_for_mock;

    #[actix_web::test]
    async fn test_update_graph_success() {
        // Create a designer state with an empty graphs cache.
        let mut designer_state = DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            tman_internal_config: Arc::new(TmanInternalConfig::default()),
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
            include_str!("../../../test_data/app_manifest.json").to_string();
        let app_property_json_str =
            include_str!("../../../test_data/app_property.json").to_string();

        // Create the property.json file in the temporary directory.
        let property_path =
            std::path::Path::new(&test_dir).join(PROPERTY_JSON_FILENAME);
        std::fs::write(&property_path, &app_property_json_str).unwrap();

        let all_pkgs_json = vec![(
            test_dir.clone(),
            app_manifest_json_str,
            app_property_json_str,
        )];

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

        let graph_id_clone = *graph_id;

        let designer_state = Arc::new(RwLock::new(designer_state));

        // Create a test app with the update_graph_endpoint.
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(designer_state.clone()))
                .route(
                    "/api/designer/v1/graphs/update",
                    web::post().to(update_graph_endpoint),
                ),
        )
        .await;

        // Create test nodes and connections.
        let nodes = [GraphNode {
            type_and_name: PkgTypeAndName {
                pkg_type: PkgType::Extension,
                name: "node1".to_string(),
            },
            addon: "test_addon".to_string(),
            extension_group: None,
            app: None,
            property: None,
        }];

        // Create a connection with message flow.
        let dest = GraphDestination {
            app: None,
            extension: "node2".to_string(),
            msg_conversion: None,
        };

        let message_flow = GraphMessageFlow {
            name: "test_cmd".to_string(),
            dest: vec![dest],
        };

        let connection = GraphConnection {
            app: None,
            extension: "node1".to_string(),
            cmd: Some(vec![message_flow]),
            data: None,
            audio_frame: None,
            video_frame: None,
        };

        let connections = vec![connection];

        // Create a request payload.
        let request_payload = UpdateGraphRequestPayload {
            graph_id: graph_id_clone,
            nodes: nodes
                .iter()
                .map(|node| GraphNodeForUpdate {
                    name: node.type_and_name.name.clone(),
                    addon: node.addon.clone(),
                    extension_group: node.extension_group.clone(),
                    app: node.app.clone(),
                    property: node.property.clone(),
                })
                .collect(),
            connections: connections.clone(),
        };

        // Make the request.
        let req = test::TestRequest::post()
            .uri("/api/designer/v1/graphs/update")
            .set_json(request_payload)
            .to_request();
        let resp = test::call_service(&app, req).await;

        println!("resp: {:?}", resp);

        // Assert that the response is successful.
        // assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();
        println!("Response body: {}", body_str);

        // Define expected property.json content after adding the connection.
        let expected_property_json_str = include_str!(
            "../../../test_data/exoected_property__update_graph_1.json"
        );

        // Read the actual property.json file generated during the test.
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
    async fn test_update_graph_not_found() {
        // Create a designer state with an empty graphs cache.
        let designer_state = Arc::new(RwLock::new(DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            tman_internal_config: Arc::new(TmanInternalConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
            graphs_cache: HashMap::new(),
        }));

        // Create a test app with the update_graph_endpoint.
        let app = test::init_service(
            App::new().app_data(web::Data::new(designer_state)).route(
                "/api/designer/v1/graphs/update",
                web::post().to(update_graph_endpoint),
            ),
        )
        .await;

        // Use a random UUID that doesn't exist in the cache.
        let nonexistent_graph_id = Uuid::new_v4();

        // Create a request payload with the nonexistent graph ID.
        let request_payload = UpdateGraphRequestPayload {
            graph_id: nonexistent_graph_id,
            nodes: vec![],
            connections: vec![],
        };

        // Make the request.
        let req = test::TestRequest::post()
            .uri("/api/designer/v1/graphs/update")
            .set_json(request_payload)
            .to_request();
        let resp = test::call_service(&app, req).await;

        // Assert that the response status is 400 Bad Request (failure).
        assert_eq!(resp.status(), 400);

        // Get the response body.
        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();

        // Parse the response as an ErrorResponse.
        let error_response: ErrorResponse =
            serde_json::from_str(body_str).unwrap();

        // Verify the error response.
        assert_eq!(error_response.status, Status::Fail);
        assert!(error_response.message.contains("not found"));
    }

    #[actix_web::test]
    async fn test_update_graph_empty_connections() {
        // Create a designer state with an empty graphs cache.
        let mut designer_state = DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            tman_internal_config: Arc::new(TmanInternalConfig::default()),
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
            include_str!("../../../test_data/app_manifest.json").to_string();
        let app_property_json_str =
            include_str!("../../../test_data/app_property.json").to_string();

        // Create the property.json file in the temporary directory.
        let property_path =
            std::path::Path::new(&test_dir).join(PROPERTY_JSON_FILENAME);
        std::fs::write(&property_path, &app_property_json_str).unwrap();

        let all_pkgs_json = vec![(
            test_dir.clone(),
            app_manifest_json_str,
            app_property_json_str,
        )];

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

        let graph_id_clone = *graph_id;

        let designer_state = Arc::new(RwLock::new(designer_state));

        // Create a test app with the update_graph_endpoint.
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(designer_state.clone()))
                .route(
                    "/api/designer/v1/graphs/update",
                    web::post().to(update_graph_endpoint),
                ),
        )
        .await;

        // Create test nodes but empty connections.
        let nodes = [GraphNode {
            type_and_name: PkgTypeAndName {
                pkg_type: PkgType::Extension,
                name: "new_node".to_string(),
            },
            addon: "test_addon".to_string(),
            extension_group: None,
            app: Some("http://example.com:8000".to_string()),
            property: None,
        }];
        let connections = vec![]; // Empty connections.

        // Create a request payload.
        let request_payload = UpdateGraphRequestPayload {
            graph_id: graph_id_clone,
            nodes: nodes
                .iter()
                .map(|node| GraphNodeForUpdate {
                    name: node.type_and_name.name.clone(),
                    addon: node.addon.clone(),
                    extension_group: node.extension_group.clone(),
                    app: node.app.clone(),
                    property: node.property.clone(),
                })
                .collect(),
            connections,
        };

        // Make the request.
        let req = test::TestRequest::post()
            .uri("/api/designer/v1/graphs/update")
            .set_json(request_payload)
            .to_request();
        let resp = test::call_service(&app, req).await;

        println!("resp: {:?}", resp);

        // Assert that the response is successful.
        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();
        println!("Response body: {}", body_str);

        // Define expected property.json content after adding the connection.
        let expected_property_json_str = include_str!(
            "../../../test_data/exoected_property__update_graph.json"
        );

        // Read the actual property.json file generated during the test.
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
