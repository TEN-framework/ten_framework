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
            graphs::update::{
                update_graph_endpoint, UpdateGraphRequestPayload,
                UpdateGraphResponseData,
            },
            response::{ApiResponse, ErrorResponse, Status},
            DesignerState,
        },
        output::TmanOutputCli,
    };
    use ten_rust::{
        graph::{
            connection::{GraphConnection, GraphDestination, GraphMessageFlow},
            graph_info::GraphInfo,
            node::GraphNode,
            Graph,
        },
        pkg_info::{pkg_type::PkgType, pkg_type_and_name::PkgTypeAndName},
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

        // Create test data for pkgs_cache.
        let all_pkgs_json_str = vec![(
            TEST_DIR.to_string(),
            include_str!("../../../test_data/app_manifest.json").to_string(),
            include_str!("../../../test_data/app_property.json").to_string(),
        )];

        let inject_ret = inject_all_pkgs_for_mock(
            &mut designer_state.pkgs_cache,
            &mut designer_state.graphs_cache,
            all_pkgs_json_str,
        );
        assert!(inject_ret.is_ok());

        // Add a test graph to the graph cache.
        let graph_id = Uuid::new_v4();
        let graph_info = GraphInfo {
            name: Some("test_graph".to_string()),
            auto_start: Some(false),
            graph: Graph {
                nodes: Vec::new(),
                connections: None,
            },
            app_base_dir: None,
            belonging_pkg_type: None,
            belonging_pkg_name: None,
        };
        designer_state.graphs_cache.insert(graph_id, graph_info);

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
        let nodes = vec![GraphNode {
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
            graph_id,
            nodes: nodes.clone(),
            connections: connections.clone(),
        };

        // Make the request.
        let req = test::TestRequest::post()
            .uri("/api/designer/v1/graphs/update")
            .set_json(request_payload)
            .to_request();
        let resp = test::call_service(&app, req).await;

        // Assert that the response is successful.
        assert!(resp.status().is_success());

        // Get the response body.
        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();

        // Parse the response.
        let response: ApiResponse<UpdateGraphResponseData> =
            serde_json::from_str(body_str).unwrap();

        // Verify the response.
        assert_eq!(response.status, Status::Ok);
        assert!(response.data.success);

        // Verify the graph was updated in the cache.
        let state_read = designer_state.read().unwrap();
        let updated_graph = state_read.graphs_cache.get(&graph_id).unwrap();

        // Verify that nodes were updated.
        assert_eq!(updated_graph.graph.nodes.len(), 1);
        assert_eq!(updated_graph.graph.nodes[0].type_and_name.name, "node1");
        assert_eq!(updated_graph.graph.nodes[0].addon, "test_addon");

        // Verify that connections were updated.
        assert!(updated_graph.graph.connections.is_some());
        let updated_connections =
            updated_graph.graph.connections.as_ref().unwrap();
        assert_eq!(updated_connections.len(), 1);
        assert_eq!(updated_connections[0].extension, "node1");
        assert!(updated_connections[0].cmd.is_some());
        let cmd_flows = updated_connections[0].cmd.as_ref().unwrap();
        assert_eq!(cmd_flows.len(), 1);
        assert_eq!(cmd_flows[0].name, "test_cmd");
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

        // Add a test graph to the graph cache.
        let graph_id = Uuid::new_v4();

        // Create an initial connection with message flow.
        let dest = GraphDestination {
            app: None,
            extension: "target_node".to_string(),
            msg_conversion: None,
        };

        let message_flow = GraphMessageFlow {
            name: "test_cmd".to_string(),
            dest: vec![dest],
        };

        let connection = GraphConnection {
            app: None,
            extension: "source_node".to_string(),
            cmd: Some(vec![message_flow]),
            data: None,
            audio_frame: None,
            video_frame: None,
        };

        let graph_info = GraphInfo {
            name: Some("test_graph".to_string()),
            auto_start: Some(false),
            graph: Graph {
                nodes: Vec::new(),
                connections: Some(vec![connection]),
            },
            app_base_dir: None,
            belonging_pkg_type: None,
            belonging_pkg_name: None,
        };

        designer_state.graphs_cache.insert(graph_id, graph_info);

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
        let nodes = vec![GraphNode {
            type_and_name: PkgTypeAndName {
                pkg_type: PkgType::Extension,
                name: "new_node".to_string(),
            },
            addon: "test_addon".to_string(),
            extension_group: None,
            app: None,
            property: None,
        }];
        let connections = vec![]; // Empty connections.

        // Create a request payload.
        let request_payload = UpdateGraphRequestPayload {
            graph_id,
            nodes: nodes.clone(),
            connections,
        };

        // Make the request.
        let req = test::TestRequest::post()
            .uri("/api/designer/v1/graphs/update")
            .set_json(request_payload)
            .to_request();
        let resp = test::call_service(&app, req).await;

        // Assert that the response is successful.
        assert!(resp.status().is_success());

        // Verify the graph was updated in the cache with connections set to
        // None.
        let state_read = designer_state.read().unwrap();
        let updated_graph = state_read.graphs_cache.get(&graph_id).unwrap();

        // Verify nodes were updated.
        assert_eq!(updated_graph.graph.nodes.len(), 1);
        assert_eq!(updated_graph.graph.nodes[0].type_and_name.name, "new_node");
        assert_eq!(updated_graph.graph.nodes[0].addon, "test_addon");

        // Verify connections were removed. Should be None since connections was
        // empty.
        assert!(updated_graph.graph.connections.is_none());
    }
}
