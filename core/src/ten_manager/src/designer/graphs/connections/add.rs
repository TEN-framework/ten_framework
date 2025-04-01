//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::sync::{Arc, RwLock};

use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

use ten_rust::{
    graph::connection::{GraphConnection, GraphDestination, GraphMessageFlow},
    pkg_info::{
        message::MsgType, pkg_type::PkgType,
        predefined_graphs::pkg_predefined_graphs_find,
    },
};

use crate::{
    designer::{
        response::{ApiResponse, ErrorResponse, Status},
        DesignerState,
    },
    graph::update_graph_connections_all_fields,
};

#[derive(Serialize, Deserialize)]
pub struct AddGraphConnectionRequestPayload {
    pub base_dir: String,
    pub graph_name: String,
    pub src_app: Option<String>,
    pub src_extension: String,
    pub msg_type: MsgType,
    pub msg_name: String,
    pub dest_app: Option<String>,
    pub dest_extension: String,
}

#[derive(Serialize, Deserialize)]
pub struct AddGraphConnectionResponsePayload {
    pub success: bool,
}

pub async fn add_graph_connection_endpoint(
    request_payload: web::Json<AddGraphConnectionRequestPayload>,
    state: web::Data<Arc<RwLock<DesignerState>>>,
) -> Result<impl Responder, actix_web::Error> {
    let mut state_write = state.write().unwrap();

    // Clone the pkgs_cache for this base_dir.
    let pkgs_cache_clone = state_write.pkgs_cache.clone();

    // Get the packages for this base_dir.
    if let Some(pkgs) =
        state_write.pkgs_cache.get_mut(&request_payload.base_dir)
    {
        // Find the app package.
        if let Some(app_pkg) = pkgs.iter_mut().find(|pkg| {
            pkg.manifest
                .as_ref()
                .is_some_and(|m| m.type_and_name.pkg_type == PkgType::App)
        }) {
            // Get the specified graph from predefined_graphs.
            if let Some(predefined_graph) = pkg_predefined_graphs_find(
                app_pkg.get_predefined_graphs(),
                |g| g.name == request_payload.graph_name,
            ) {
                let mut graph = predefined_graph.graph.clone();

                // Add the connection.
                match graph.add_connection(
                    request_payload.src_app.clone(),
                    request_payload.src_extension.clone(),
                    request_payload.msg_type.clone(),
                    request_payload.msg_name.clone(),
                    request_payload.dest_app.clone(),
                    request_payload.dest_extension.clone(),
                    &pkgs_cache_clone,
                ) {
                    Ok(_) => {
                        // Update the predefined_graph in the app_pkg.
                        let mut new_graph = predefined_graph.clone();
                        new_graph.graph = graph;
                        app_pkg.update_predefined_graph(&new_graph);

                        // Update property.json file with the updated graph.
                        if let Some(property) = &mut app_pkg.property {
                            // Create a new connection object with the same
                            // parameters used in add_connection.

                            // Create the destination.
                            let destination = GraphDestination {
                                app: request_payload.dest_app.clone(),
                                extension: request_payload
                                    .dest_extension
                                    .clone(),
                                msg_conversion: None,
                            };

                            // Create the message flow based on the message
                            // type.
                            let message_flow = GraphMessageFlow {
                                name: request_payload.msg_name.clone(),
                                dest: vec![destination],
                            };

                            // Create the connection object
                            let mut connection = GraphConnection {
                                app: request_payload.src_app.clone(),
                                extension: request_payload
                                    .src_extension
                                    .clone(),
                                cmd: None,
                                data: None,
                                audio_frame: None,
                                video_frame: None,
                            };

                            // Add the message flow to the appropriate field.
                            match request_payload.msg_type {
                                MsgType::Cmd => {
                                    connection.cmd = Some(vec![message_flow]);
                                }
                                MsgType::Data => {
                                    connection.data = Some(vec![message_flow]);
                                }
                                MsgType::AudioFrame => {
                                    connection.audio_frame =
                                        Some(vec![message_flow]);
                                }
                                MsgType::VideoFrame => {
                                    connection.video_frame =
                                        Some(vec![message_flow]);
                                }
                            }

                            // Update only with the new connection.
                            let connections_to_add = vec![connection];

                            // Update the property_all_fields map and write to
                            // property.json.
                            if let Err(e) = update_graph_connections_all_fields(
                                &request_payload.base_dir,
                                &mut property.all_fields,
                                &request_payload.graph_name,
                                Some(&connections_to_add),
                                None,
                            ) {
                                eprintln!("Warning: Failed to update property.json file: {}", e);
                            }
                        }

                        let response = ApiResponse {
                            status: Status::Ok,
                            data: AddGraphConnectionResponsePayload {
                                success: true,
                            },
                            meta: None,
                        };
                        Ok(HttpResponse::Ok().json(response))
                    }
                    Err(err) => {
                        let error_response = ErrorResponse {
                            status: Status::Fail,
                            message: format!(
                                "Failed to add connection: {}",
                                err
                            ),
                            error: None,
                        };
                        Ok(HttpResponse::BadRequest().json(error_response))
                    }
                }
            } else {
                let error_response = ErrorResponse {
                    status: Status::Fail,
                    message: format!(
                        "Graph '{}' not found",
                        request_payload.graph_name
                    ),
                    error: None,
                };
                Ok(HttpResponse::NotFound().json(error_response))
            }
        } else {
            let error_response = ErrorResponse {
                status: Status::Fail,
                message: "App package not found".to_string(),
                error: None,
            };
            Ok(HttpResponse::NotFound().json(error_response))
        }
    } else {
        let error_response = ErrorResponse {
            status: Status::Fail,
            message: "Base directory not found".to_string(),
            error: None,
        };
        Ok(HttpResponse::NotFound().json(error_response))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use actix_web::{test, App};
    use ten_rust::pkg_info::constants::PROPERTY_JSON_FILENAME;

    use super::*;
    use crate::{
        config::TmanConfig, constants::TEST_DIR,
        designer::mock::inject_all_pkgs_for_mock, output::TmanOutputCli,
    };

    #[actix_web::test]
    async fn test_add_graph_connection_success() {
        let mut designer_state = DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
        };

        // Load both the app package JSON and extension addon package JSONs.
        let app_manifest =
            include_str!("../test_data_embed/app_manifest.json").to_string();
        let app_property =
            include_str!("../test_data_embed/app_property.json").to_string();

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
            (app_manifest, app_property),
            (ext1_manifest, empty_property.clone()),
            (ext2_manifest, empty_property.clone()),
            (ext3_manifest, empty_property.clone()),
        ];

        let inject_ret = inject_all_pkgs_for_mock(
            TEST_DIR,
            &mut designer_state.pkgs_cache,
            all_pkgs_json,
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

        // Add a connection between existing nodes in the default graph.
        // Use "http://example.com:8000" for both src_app and dest_app to match the test data.
        let request_payload = AddGraphConnectionRequestPayload {
            base_dir: TEST_DIR.to_string(),
            graph_name: "default_with_app_uri".to_string(),
            src_app: Some("http://example.com:8000".to_string()),
            src_extension: "extension_1".to_string(),
            msg_type: MsgType::Cmd,
            msg_name: "test_cmd".to_string(),
            dest_app: Some("http://example.com:8000".to_string()),
            dest_extension: "extension_2".to_string(),
        };

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
    }

    #[actix_web::test]
    async fn test_add_graph_connection_invalid_graph() {
        let mut designer_state = DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
        };

        let all_pkgs_json = vec![(
            include_str!("../test_data_embed/app_manifest.json").to_string(),
            include_str!("../test_data_embed/app_property.json").to_string(),
        )];

        let inject_ret = inject_all_pkgs_for_mock(
            TEST_DIR,
            &mut designer_state.pkgs_cache,
            all_pkgs_json,
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
            base_dir: TEST_DIR.to_string(),
            graph_name: "non_existent_graph".to_string(),
            src_app: None,
            src_extension: "extension_1".to_string(),
            msg_type: MsgType::Cmd,
            msg_name: "test_cmd".to_string(),
            dest_app: None,
            dest_extension: "extension_2".to_string(),
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
        };

        // Load both the app package JSON and extension addon package JSONs.
        let app_manifest =
            include_str!("../test_data_embed/app_manifest.json").to_string();
        let app_property =
            include_str!("../test_data_embed/app_property.json").to_string();

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
            (app_manifest, app_property),
            (ext1_manifest, empty_property.clone()),
            (ext2_manifest, empty_property.clone()),
            (ext3_manifest, empty_property.clone()),
        ];

        let inject_ret = inject_all_pkgs_for_mock(
            TEST_DIR,
            &mut designer_state.pkgs_cache,
            all_pkgs_json,
        );
        assert!(inject_ret.is_ok());

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

        // Add first connection.
        let request_payload1 = AddGraphConnectionRequestPayload {
            base_dir: TEST_DIR.to_string(),
            graph_name: "default_with_app_uri".to_string(),
            src_app: Some("http://example.com:8000".to_string()),
            src_extension: "extension_1".to_string(),
            msg_type: MsgType::Cmd,
            msg_name: "test_cmd1".to_string(),
            dest_app: Some("http://example.com:8000".to_string()),
            dest_extension: "extension_2".to_string(),
        };

        let req1 = test::TestRequest::post()
            .uri("/api/designer/v1/graphs/connections/add")
            .set_json(request_payload1)
            .to_request();
        let resp1 = test::call_service(&app, req1).await;

        assert!(resp1.status().is_success());

        // Add second connection to create a sequence.
        let request_payload2 = AddGraphConnectionRequestPayload {
            base_dir: TEST_DIR.to_string(),
            graph_name: "default_with_app_uri".to_string(),
            src_app: Some("http://example.com:8000".to_string()),
            src_extension: "extension_1".to_string(),
            msg_type: MsgType::Cmd,
            msg_name: "test_cmd2".to_string(),
            dest_app: Some("http://example.com:8000".to_string()),
            dest_extension: "extension_3".to_string(),
        };

        let req2 = test::TestRequest::post()
            .uri("/api/designer/v1/graphs/connections/add")
            .set_json(request_payload2)
            .to_request();
        let resp2 = test::call_service(&app, req2).await;

        assert!(resp2.status().is_success());

        let state = designer_state_arc.read().unwrap();
        let pkgs = state.pkgs_cache.get(TEST_DIR).unwrap();

        // Find the app package.
        let app_pkg = pkgs
            .iter()
            .find(|pkg| {
                pkg.manifest
                    .as_ref()
                    .is_some_and(|m| m.type_and_name.pkg_type == PkgType::App)
            })
            .unwrap();

        // Get the predefined graph from app_pkg to check if connections were
        // added correctly.
        let predefined_graph =
            pkg_predefined_graphs_find(app_pkg.get_predefined_graphs(), |g| {
                g.name == "default_with_app_uri"
            })
            .unwrap();

        // Now check the connections in memory.
        if let Some(connections) = &predefined_graph.graph.connections {
            // Find the connection for extension_1.
            let ext1_connection = connections
                .iter()
                .find(|conn| {
                    conn.extension == "extension_1"
                        && conn.app
                            == Some("http://example.com:8000".to_string())
                })
                .unwrap();

            // Check that we have both commands and they are in order.
            if let Some(cmd_flows) = &ext1_connection.cmd {
                // Find both commands.
                let cmd1_position = cmd_flows
                    .iter()
                    .position(|flow| flow.name == "test_cmd1")
                    .unwrap();
                let cmd2_position = cmd_flows
                    .iter()
                    .position(|flow| flow.name == "test_cmd2")
                    .unwrap();

                // First command should appear before second command (preserving
                // order).
                assert!(cmd1_position < cmd2_position,
                         "test_cmd1 should come before test_cmd2, but found at positions {} and {}",
                         cmd1_position, cmd2_position);

                // Verify that test_cmd1 points to extension_2.
                let cmd1 = &cmd_flows[cmd1_position];
                assert_eq!(cmd1.dest[0].extension, "extension_2");

                // Verify that test_cmd2 points to extension_3.
                let cmd2 = &cmd_flows[cmd2_position];
                assert_eq!(cmd2.dest[0].extension, "extension_3");
            } else {
                panic!("No cmd flows found in connection");
            }
        } else {
            panic!("No connections found in graph");
        }

        // Now check the property.all_fields to verify that the connections are
        // in the correct order and the new connections are appended to the end.
        if let Some(property) = &app_pkg.property {
            // Check if we have the graph in property.all_fields.
            if let Some(ten_value) = property.all_fields.get("_ten") {
                if let Some(predefined_graphs) = ten_value
                    .get("predefined_graphs")
                    .and_then(|v| v.as_array())
                {
                    // Find our graph.
                    let graph_value = predefined_graphs
                        .iter()
                        .find(|g| {
                            g.get("name").and_then(|n| n.as_str())
                                == Some("default_with_app_uri")
                        })
                        .unwrap();

                    // Check the connections array.
                    if let Some(connections) = graph_value
                        .get("connections")
                        .and_then(|c| c.as_array())
                    {
                        // Find the connection with the matching extension and
                        // app.
                        let ext1_connection = connections
                            .iter()
                            .find(|conn| {
                                conn.get("extension").and_then(|e| e.as_str())
                                    == Some("extension_1")
                                    && conn.get("app").and_then(|a| a.as_str())
                                        == Some("http://example.com:8000")
                            })
                            .unwrap();

                        // Check the cmd array to verify our new connections are
                        // at the end.
                        if let Some(cmd_array) = ext1_connection
                            .get("cmd")
                            .and_then(|c| c.as_array())
                        {
                            // The last two entries should be our new test_cmd1
                            // and test_cmd2.
                            let last_entries = cmd_array
                                .iter()
                                .rev()
                                .take(2)
                                .collect::<Vec<_>>();

                            // The last entry should be test_cmd2.
                            assert_eq!(
                                last_entries[0]
                                    .get("name")
                                    .and_then(|n| n.as_str()),
                                Some("test_cmd2"),
                                "Last entry in cmd array should be test_cmd2"
                            );

                            // The second-to-last entry should be test_cmd1.
                            assert_eq!(
                                    last_entries[1].get("name").and_then(|n| n.as_str()),
                                    Some("test_cmd1"),
                                    "Second-to-last entry in cmd array should be test_cmd1"
                                );

                            // Verify destinations.
                            let test_cmd1_dest = last_entries[1]
                                .get("dest")
                                .and_then(|d| d.as_array())
                                .unwrap();
                            assert_eq!(
                                test_cmd1_dest[0]
                                    .get("extension")
                                    .and_then(|e| e.as_str()),
                                Some("extension_2"),
                                "test_cmd1 should point to extension_2"
                            );

                            let test_cmd2_dest = last_entries[0]
                                .get("dest")
                                .and_then(|d| d.as_array())
                                .unwrap();
                            assert_eq!(
                                test_cmd2_dest[0]
                                    .get("extension")
                                    .and_then(|e| e.as_str()),
                                Some("extension_3"),
                                "test_cmd2 should point to extension_3"
                            );
                        } else {
                            panic!("No cmd array found in the connection in property.all_fields");
                        }
                    } else {
                        panic!("No connections array found in property.all_fields for the graph");
                    }
                } else {
                    panic!("No predefined_graphs array found in property.all_fields");
                }
            } else {
                panic!("No _ten object found in property.all_fields");
            }
        } else {
            panic!("No property found for app_pkg");
        }
    }

    #[actix_web::test]
    async fn test_add_graph_connection_file_comparison() {
        let mut designer_state = DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
        };

        // Create a temporary directory for our test to store the generated
        // property.json.
        let temp_dir = tempfile::tempdir().unwrap();
        let test_dir = temp_dir.path().to_str().unwrap().to_string();

        // Load both the app package JSON and extension addon package JSONs.
        let app_manifest =
            include_str!("../test_data_embed/app_manifest.json").to_string();
        let app_property =
            include_str!("../test_data_embed/app_property.json").to_string();

        // Create the property.json file in the temporary directory.
        let property_path =
            std::path::Path::new(&test_dir).join(PROPERTY_JSON_FILENAME);
        std::fs::write(&property_path, &app_property).unwrap();

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
            (app_manifest, app_property),
            (ext1_manifest, empty_property.clone()),
            (ext2_manifest, empty_property.clone()),
            (ext3_manifest, empty_property.clone()),
        ];

        let inject_ret = inject_all_pkgs_for_mock(
            &test_dir,
            &mut designer_state.pkgs_cache,
            all_pkgs_json,
        );
        assert!(inject_ret.is_ok());

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

        // Add first connection.
        let request_payload1 = AddGraphConnectionRequestPayload {
            base_dir: test_dir.clone(),
            graph_name: "default_with_app_uri".to_string(),
            src_app: Some("http://example.com:8000".to_string()),
            src_extension: "extension_1".to_string(),
            msg_type: MsgType::Cmd,
            msg_name: "test_cmd1".to_string(),
            dest_app: Some("http://example.com:8000".to_string()),
            dest_extension: "extension_2".to_string(),
        };

        let req1 = test::TestRequest::post()
            .uri("/api/designer/v1/graphs/connections/add")
            .set_json(request_payload1)
            .to_request();
        let resp1 = test::call_service(&app, req1).await;

        assert!(resp1.status().is_success());

        // Add second connection to create a sequence.
        let request_payload2 = AddGraphConnectionRequestPayload {
            base_dir: test_dir.clone(),
            graph_name: "default_with_app_uri".to_string(),
            src_app: Some("http://example.com:8000".to_string()),
            src_extension: "extension_1".to_string(),
            msg_type: MsgType::Cmd,
            msg_name: "test_cmd2".to_string(),
            dest_app: Some("http://example.com:8000".to_string()),
            dest_extension: "extension_3".to_string(),
        };

        let req2 = test::TestRequest::post()
            .uri("/api/designer/v1/graphs/connections/add")
            .set_json(request_payload2)
            .to_request();
        let resp2 = test::call_service(&app, req2).await;

        assert!(resp2.status().is_success());

        // Define expected property.json content after adding both connections.
        let expected_property = include_str!("test_data_embed/expected_json__test_add_graph_connection_file_comparison.json");

        // Read the actual property.json file generated during the test.
        let property_path =
            std::path::Path::new(&test_dir).join(PROPERTY_JSON_FILENAME);
        let actual_property = std::fs::read_to_string(property_path).unwrap();

        // Normalize both JSON strings to handle formatting differences
        let expected_value: serde_json::Value =
            serde_json::from_str(expected_property).unwrap();
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
        };

        // Load both the app package JSON and extension addon package JSONs.
        let app_manifest =
            include_str!("../test_data_embed/app_manifest.json").to_string();
        let app_property =
            include_str!("../test_data_embed/app_property.json").to_string();

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
            (app_manifest, app_property),
            (ext1_manifest, empty_property.clone()),
            (ext2_manifest, empty_property),
        ];

        let inject_ret = inject_all_pkgs_for_mock(
            TEST_DIR,
            &mut designer_state.pkgs_cache,
            all_pkgs_json,
        );
        assert!(inject_ret.is_ok());

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

        // Add a DATA type connection between extensions.
        let request_payload = AddGraphConnectionRequestPayload {
            base_dir: TEST_DIR.to_string(),
            graph_name: "default_with_app_uri".to_string(),
            src_app: Some("http://example.com:8000".to_string()),
            src_extension: "extension_1".to_string(),
            msg_type: MsgType::Data,
            msg_name: "test_data".to_string(),
            dest_app: Some("http://example.com:8000".to_string()),
            dest_extension: "extension_2".to_string(),
        };

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

        // Verify the connection was added correctly in memory.
        let state = designer_state_arc.read().unwrap();
        let pkgs = state.pkgs_cache.get(TEST_DIR).unwrap();
        let app_pkg = pkgs
            .iter()
            .find(|pkg| {
                pkg.manifest
                    .as_ref()
                    .is_some_and(|m| m.type_and_name.pkg_type == PkgType::App)
            })
            .unwrap();

        // Get the predefined graph.
        let predefined_graph =
            pkg_predefined_graphs_find(app_pkg.get_predefined_graphs(), |g| {
                g.name == "default_with_app_uri"
            })
            .unwrap();

        // Check the data field in the graph's connections.
        if let Some(connections) = &predefined_graph.graph.connections {
            // Find the connection for extension_1.
            let ext1_connection = connections
                .iter()
                .find(|conn| {
                    conn.extension == "extension_1"
                        && conn.app
                            == Some("http://example.com:8000".to_string())
                })
                .unwrap();

            // Check that we have the data message field.
            assert!(
                ext1_connection.data.is_some(),
                "Data field should be present in connection"
            );

            // Verify the data field contains our message.
            if let Some(data_flows) = &ext1_connection.data {
                let data_msg = data_flows
                    .iter()
                    .find(|flow| flow.name == "test_data")
                    .unwrap();

                // Verify destination is correct.
                assert_eq!(data_msg.dest[0].extension, "extension_2");
                assert_eq!(
                    data_msg.dest[0].app,
                    Some("http://example.com:8000".to_string())
                );
            } else {
                panic!("Data flows not found in connection");
            }
        } else {
            panic!("No connections found in graph");
        }

        // Verify the property.all_fields.
        if let Some(property) = &app_pkg.property {
            if let Some(ten_value) = property.all_fields.get("_ten") {
                if let Some(predefined_graphs) = ten_value
                    .get("predefined_graphs")
                    .and_then(|v| v.as_array())
                {
                    // Find our graph.
                    let graph_value = predefined_graphs
                        .iter()
                        .find(|g| {
                            g.get("name").and_then(|n| n.as_str())
                                == Some("default_with_app_uri")
                        })
                        .unwrap();

                    // Check the connections array.
                    if let Some(connections) = graph_value
                        .get("connections")
                        .and_then(|c| c.as_array())
                    {
                        // Find the connection with the matching extension and
                        // app.
                        let ext1_connection = connections
                            .iter()
                            .find(|conn| {
                                conn.get("extension").and_then(|e| e.as_str())
                                    == Some("extension_1")
                                    && conn.get("app").and_then(|a| a.as_str())
                                        == Some("http://example.com:8000")
                            })
                            .unwrap();

                        // Check the data array.
                        let data_array = ext1_connection
                            .get("data")
                            .and_then(|d| d.as_array())
                            .unwrap();

                        // Find our data message.
                        let data_msg = data_array
                            .iter()
                            .find(|d| {
                                d.get("name").and_then(|n| n.as_str())
                                    == Some("test_data")
                            })
                            .unwrap();

                        // Verify destination.
                        let dest_array = data_msg
                            .get("dest")
                            .and_then(|d| d.as_array())
                            .unwrap();
                        assert_eq!(
                            dest_array[0]
                                .get("extension")
                                .and_then(|e| e.as_str()),
                            Some("extension_2"),
                            "test_data should point to extension_2"
                        );
                    } else {
                        panic!("No connections array found in property.all_fields for the graph");
                    }
                } else {
                    panic!("No predefined_graphs array found in property.all_fields");
                }
            } else {
                panic!("No _ten object found in property.all_fields");
            }
        } else {
            panic!("No property found for app_pkg");
        }
    }

    #[actix_web::test]
    async fn test_add_graph_connection_frame_types() {
        let mut designer_state = DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
        };

        // Load both the app package JSON and extension addon package JSONs.
        let app_manifest =
            include_str!("../test_data_embed/app_manifest.json").to_string();
        let app_property =
            include_str!("../test_data_embed/app_property.json").to_string();

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
            (app_manifest, app_property),
            (ext1_manifest, empty_property.clone()),
            (ext2_manifest, empty_property),
        ];

        let inject_ret = inject_all_pkgs_for_mock(
            TEST_DIR,
            &mut designer_state.pkgs_cache,
            all_pkgs_json,
        );
        assert!(inject_ret.is_ok());

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

        // First add an AUDIO_FRAME type connection.
        let audio_request = AddGraphConnectionRequestPayload {
            base_dir: TEST_DIR.to_string(),
            graph_name: "default_with_app_uri".to_string(),
            src_app: Some("http://example.com:8000".to_string()),
            src_extension: "extension_1".to_string(),
            msg_type: MsgType::AudioFrame,
            msg_name: "audio_stream".to_string(),
            dest_app: Some("http://example.com:8000".to_string()),
            dest_extension: "extension_2".to_string(),
        };

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/graphs/connections/add")
            .set_json(audio_request)
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());

        // Then add a VIDEO_FRAME type connection.
        let video_request = AddGraphConnectionRequestPayload {
            base_dir: TEST_DIR.to_string(),
            graph_name: "default_with_app_uri".to_string(),
            src_app: Some("http://example.com:8000".to_string()),
            src_extension: "extension_1".to_string(),
            msg_type: MsgType::VideoFrame,
            msg_name: "video_stream".to_string(),
            dest_app: Some("http://example.com:8000".to_string()),
            dest_extension: "extension_2".to_string(),
        };

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/graphs/connections/add")
            .set_json(video_request)
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());

        // Verify both connections were added correctly in memory.
        let state = designer_state_arc.read().unwrap();
        let pkgs = state.pkgs_cache.get(TEST_DIR).unwrap();
        let app_pkg = pkgs
            .iter()
            .find(|pkg| {
                pkg.manifest
                    .as_ref()
                    .is_some_and(|m| m.type_and_name.pkg_type == PkgType::App)
            })
            .unwrap();

        // Get the predefined graph.
        let predefined_graph =
            pkg_predefined_graphs_find(app_pkg.get_predefined_graphs(), |g| {
                g.name == "default_with_app_uri"
            })
            .unwrap();

        // Check the connections.
        if let Some(connections) = &predefined_graph.graph.connections {
            // Find the connection for extension_1.
            let ext1_connection = connections
                .iter()
                .find(|conn| {
                    conn.extension == "extension_1"
                        && conn.app
                            == Some("http://example.com:8000".to_string())
                })
                .unwrap();

            // Check that we have the audio_frame message field.
            assert!(
                ext1_connection.audio_frame.is_some(),
                "audio_frame field should be present in connection"
            );

            // Verify the audio_frame field contains our message.
            if let Some(audio_flows) = &ext1_connection.audio_frame {
                let audio_msg = audio_flows
                    .iter()
                    .find(|flow| flow.name == "audio_stream")
                    .unwrap();

                // Verify destination is correct.
                assert_eq!(audio_msg.dest[0].extension, "extension_2");
                assert_eq!(
                    audio_msg.dest[0].app,
                    Some("http://example.com:8000".to_string())
                );
            } else {
                panic!("Audio flows not found in connection");
            }

            // Check that we have the video_frame message field.
            assert!(
                ext1_connection.video_frame.is_some(),
                "video_frame field should be present in connection"
            );

            // Verify the video_frame field contains our message.
            if let Some(video_flows) = &ext1_connection.video_frame {
                let video_msg = video_flows
                    .iter()
                    .find(|flow| flow.name == "video_stream")
                    .unwrap();

                // Verify destination is correct.
                assert_eq!(video_msg.dest[0].extension, "extension_2");
                assert_eq!(
                    video_msg.dest[0].app,
                    Some("http://example.com:8000".to_string())
                );
            } else {
                panic!("Video flows not found in connection");
            }
        } else {
            panic!("No connections found in graph");
        }

        // Verify the property.all_fields.
        if let Some(property) = &app_pkg.property {
            if let Some(ten_value) = property.all_fields.get("_ten") {
                if let Some(predefined_graphs) = ten_value
                    .get("predefined_graphs")
                    .and_then(|v| v.as_array())
                {
                    // Find our graph.
                    let graph_value = predefined_graphs
                        .iter()
                        .find(|g| {
                            g.get("name").and_then(|n| n.as_str())
                                == Some("default_with_app_uri")
                        })
                        .unwrap();

                    // Check the connections array.
                    if let Some(connections) = graph_value
                        .get("connections")
                        .and_then(|c| c.as_array())
                    {
                        // Find the connection with the matching extension and
                        // app.
                        let ext1_connection = connections
                            .iter()
                            .find(|conn| {
                                conn.get("extension").and_then(|e| e.as_str())
                                    == Some("extension_1")
                                    && conn.get("app").and_then(|a| a.as_str())
                                        == Some("http://example.com:8000")
                            })
                            .unwrap();

                        // Check the audio_frame array.
                        let audio_array = ext1_connection
                            .get("audio_frame")
                            .and_then(|d| d.as_array())
                            .unwrap();

                        // Find our audio message.
                        let audio_msg = audio_array
                            .iter()
                            .find(|d| {
                                d.get("name").and_then(|n| n.as_str())
                                    == Some("audio_stream")
                            })
                            .unwrap();

                        // Verify destination.
                        let audio_dest = audio_msg
                            .get("dest")
                            .and_then(|d| d.as_array())
                            .unwrap();
                        assert_eq!(
                            audio_dest[0]
                                .get("extension")
                                .and_then(|e| e.as_str()),
                            Some("extension_2"),
                            "audio_stream should point to extension_2"
                        );

                        // Check the video_frame array.
                        let video_array = ext1_connection
                            .get("video_frame")
                            .and_then(|d| d.as_array())
                            .unwrap();

                        // Find our video message.
                        let video_msg = video_array
                            .iter()
                            .find(|d| {
                                d.get("name").and_then(|n| n.as_str())
                                    == Some("video_stream")
                            })
                            .unwrap();

                        // Verify destination.
                        let video_dest = video_msg
                            .get("dest")
                            .and_then(|d| d.as_array())
                            .unwrap();
                        assert_eq!(
                            video_dest[0]
                                .get("extension")
                                .and_then(|e| e.as_str()),
                            Some("extension_2"),
                            "video_stream should point to extension_2"
                        );
                    } else {
                        panic!("No connections array found in property.all_fields for the graph");
                    }
                } else {
                    panic!("No predefined_graphs array found in property.all_fields");
                }
            } else {
                panic!("No _ten object found in property.all_fields");
            }
        } else {
            panic!("No property found for app_pkg");
        }
    }

    #[actix_web::test]
    async fn test_add_multiple_connections_preservation_order() {
        let mut designer_state = DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
        };

        // Load both the app package JSON and extension addon package JSONs.
        let app_manifest =
            include_str!("../test_data_embed/app_manifest.json").to_string();
        let app_property =
            include_str!("../test_data_embed/app_property.json").to_string();

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
            (app_manifest, app_property),
            (ext1_manifest, empty_property.clone()),
            (ext2_manifest, empty_property.clone()),
            (ext3_manifest, empty_property),
        ];

        let inject_ret = inject_all_pkgs_for_mock(
            TEST_DIR,
            &mut designer_state.pkgs_cache,
            all_pkgs_json,
        );
        assert!(inject_ret.is_ok());

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

        // Add first command.
        let cmd1_request = AddGraphConnectionRequestPayload {
            base_dir: TEST_DIR.to_string(),
            graph_name: "default_with_app_uri".to_string(),
            src_app: Some("http://example.com:8000".to_string()),
            src_extension: "extension_1".to_string(),
            msg_type: MsgType::Cmd,
            msg_name: "cmd_1".to_string(),
            dest_app: Some("http://example.com:8000".to_string()),
            dest_extension: "extension_2".to_string(),
        };

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/graphs/connections/add")
            .set_json(cmd1_request)
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());

        // Add second command.
        let cmd2_request = AddGraphConnectionRequestPayload {
            base_dir: TEST_DIR.to_string(),
            graph_name: "default_with_app_uri".to_string(),
            src_app: Some("http://example.com:8000".to_string()),
            src_extension: "extension_1".to_string(),
            msg_type: MsgType::Cmd,
            msg_name: "cmd_2".to_string(),
            dest_app: Some("http://example.com:8000".to_string()),
            dest_extension: "extension_3".to_string(),
        };

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/graphs/connections/add")
            .set_json(cmd2_request)
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());

        // Add third command.
        let cmd3_request = AddGraphConnectionRequestPayload {
            base_dir: TEST_DIR.to_string(),
            graph_name: "default_with_app_uri".to_string(),
            src_app: Some("http://example.com:8000".to_string()),
            src_extension: "extension_1".to_string(),
            msg_type: MsgType::Cmd,
            msg_name: "cmd_3".to_string(),
            dest_app: Some("http://example.com:8000".to_string()),
            dest_extension: "extension_2".to_string(),
        };

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/graphs/connections/add")
            .set_json(cmd3_request)
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());

        // Verify connections were added in the correct order in memory.
        let state = designer_state_arc.read().unwrap();
        let pkgs = state.pkgs_cache.get(TEST_DIR).unwrap();
        let app_pkg = pkgs
            .iter()
            .find(|pkg| {
                pkg.manifest
                    .as_ref()
                    .is_some_and(|m| m.type_and_name.pkg_type == PkgType::App)
            })
            .unwrap();

        // Get the predefined graph.
        let predefined_graph =
            pkg_predefined_graphs_find(app_pkg.get_predefined_graphs(), |g| {
                g.name == "default_with_app_uri"
            })
            .unwrap();

        // Check the connections.
        if let Some(connections) = &predefined_graph.graph.connections {
            // Find the connection for extension_1.
            let ext1_connection = connections
                .iter()
                .find(|conn| {
                    conn.extension == "extension_1"
                        && conn.app
                            == Some("http://example.com:8000".to_string())
                })
                .unwrap();

            // Check the cmd array exists.
            assert!(
                ext1_connection.cmd.is_some(),
                "cmd field should be present in connection"
            );

            if let Some(cmd_flows) = &ext1_connection.cmd {
                // Find the command messages.
                let cmd1_index = cmd_flows
                    .iter()
                    .position(|flow| flow.name == "cmd_1")
                    .unwrap();
                let cmd2_index = cmd_flows
                    .iter()
                    .position(|flow| flow.name == "cmd_2")
                    .unwrap();
                let cmd3_index = cmd_flows
                    .iter()
                    .position(|flow| flow.name == "cmd_3")
                    .unwrap();

                // Verify order (cmd1, cmd2, cmd3).
                assert!(
                    cmd1_index < cmd2_index,
                    "cmd_1 should be before cmd_2"
                );
                assert!(
                    cmd2_index < cmd3_index,
                    "cmd_2 should be before cmd_3"
                );

                // Verify destinations.
                assert_eq!(
                    cmd_flows[cmd1_index].dest[0].extension,
                    "extension_2"
                );
                assert_eq!(
                    cmd_flows[cmd2_index].dest[0].extension,
                    "extension_3"
                );
                assert_eq!(
                    cmd_flows[cmd3_index].dest[0].extension,
                    "extension_2"
                );
            } else {
                panic!("Cmd flows not found in connection");
            }
        } else {
            panic!("No connections found in graph");
        }

        // Verify the property.all_fields.
        if let Some(property) = &app_pkg.property {
            if let Some(ten_value) = property.all_fields.get("_ten") {
                if let Some(predefined_graphs) = ten_value
                    .get("predefined_graphs")
                    .and_then(|v| v.as_array())
                {
                    // Find our graph.
                    let graph_value = predefined_graphs
                        .iter()
                        .find(|g| {
                            g.get("name").and_then(|n| n.as_str())
                                == Some("default_with_app_uri")
                        })
                        .unwrap();

                    // Check the connections array.
                    if let Some(connections) = graph_value
                        .get("connections")
                        .and_then(|c| c.as_array())
                    {
                        // Find the connection with the matching extension and
                        // app.
                        let ext1_connection = connections
                            .iter()
                            .find(|conn| {
                                conn.get("extension").and_then(|e| e.as_str())
                                    == Some("extension_1")
                                    && conn.get("app").and_then(|a| a.as_str())
                                        == Some("http://example.com:8000")
                            })
                            .unwrap();

                        // Check the cmd array.
                        let cmd_array = ext1_connection
                            .get("cmd")
                            .and_then(|d| d.as_array())
                            .unwrap();

                        // Get the indexes of our commands by their names.
                        let cmd1_index = cmd_array
                            .iter()
                            .position(|cmd| {
                                cmd.get("name").and_then(|n| n.as_str())
                                    == Some("cmd_1")
                            })
                            .unwrap();

                        let cmd2_index = cmd_array
                            .iter()
                            .position(|cmd| {
                                cmd.get("name").and_then(|n| n.as_str())
                                    == Some("cmd_2")
                            })
                            .unwrap();

                        let cmd3_index = cmd_array
                            .iter()
                            .position(|cmd| {
                                cmd.get("name").and_then(|n| n.as_str())
                                    == Some("cmd_3")
                            })
                            .unwrap();

                        // Verify order (cmd1, cmd2, cmd3).
                        assert!(cmd1_index < cmd2_index, "cmd_1 should be before cmd_2 in property.all_fields");
                        assert!(cmd2_index < cmd3_index, "cmd_2 should be before cmd_3 in property.all_fields");

                        // Verify cmd1 destination.
                        let cmd1_dest = cmd_array[cmd1_index]
                            .get("dest")
                            .and_then(|d| d.as_array())
                            .unwrap();
                        assert_eq!(
                            cmd1_dest[0]
                                .get("extension")
                                .and_then(|e| e.as_str()),
                            Some("extension_2"),
                            "cmd_1 should point to extension_2"
                        );

                        // Verify cmd2 destination.
                        let cmd2_dest = cmd_array[cmd2_index]
                            .get("dest")
                            .and_then(|d| d.as_array())
                            .unwrap();
                        assert_eq!(
                            cmd2_dest[0]
                                .get("extension")
                                .and_then(|e| e.as_str()),
                            Some("extension_3"),
                            "cmd_2 should point to extension_3"
                        );

                        // Verify cmd3 destination.
                        let cmd3_dest = cmd_array[cmd3_index]
                            .get("dest")
                            .and_then(|d| d.as_array())
                            .unwrap();
                        assert_eq!(
                            cmd3_dest[0]
                                .get("extension")
                                .and_then(|e| e.as_str()),
                            Some("extension_2"),
                            "cmd_3 should point to extension_2"
                        );

                        // Verify the commands appear at the end of the array
                        // The three commands we added should be the last three
                        // elements in the array.
                        assert_eq!(
                            cmd_array.len() - 3,
                            cmd1_index,
                            "cmd_1 should be the antepenultimate element in the cmd array"
                        );
                        assert_eq!(
                            cmd_array.len() - 2,
                            cmd2_index,
                            "cmd_2 should be the penultimate element in the cmd array"
                        );
                        assert_eq!(
                            cmd_array.len() - 1,
                            cmd3_index,
                            "cmd_3 should be the last element in the cmd array"
                        );
                    } else {
                        panic!("No connections array found in property.all_fields for the graph");
                    }
                } else {
                    panic!("No predefined_graphs array found in property.all_fields");
                }
            } else {
                panic!("No _ten object found in property.all_fields");
            }
        } else {
            panic!("No property found for app_pkg");
        }
    }

    #[actix_web::test]
    async fn test_add_graph_connection_with_msg_conversion() {
        let mut designer_state = DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
        };

        // Load both the app package JSON and extension addon package JSONs.
        let app_manifest =
            include_str!("../test_data_embed/app_manifest.json").to_string();
        let app_property =
            include_str!("../test_data_embed/app_property.json").to_string();

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

        // The empty property for addons
        let empty_property = r#"{"_ten":{}}"#.to_string();

        let all_pkgs_json = vec![
            (app_manifest, app_property),
            (ext1_manifest, empty_property.clone()),
            (ext2_manifest, empty_property),
        ];

        let inject_ret = inject_all_pkgs_for_mock(
            TEST_DIR,
            &mut designer_state.pkgs_cache,
            all_pkgs_json,
        );
        assert!(inject_ret.is_ok());

        // Create the GraphDestination with msg_conversion
        let destination = GraphDestination {
            app: Some("http://example.com:8000".to_string()),
            extension: "extension_2".to_string(),
            msg_conversion: Some(ten_rust::graph::msg_conversion::MsgAndResultConversion {
                msg: ten_rust::graph::msg_conversion::MsgConversion {
                    conversion_type: ten_rust::graph::msg_conversion::MsgConversionType::PerProperty,
                    rules: ten_rust::graph::msg_conversion::MsgConversionRules {
                        keep_original: Some(false),
                        rules: vec![
                            ten_rust::graph::msg_conversion::MsgConversionRule {
                                path: "result.status".to_string(),
                                conversion_mode: ten_rust::graph::msg_conversion::MsgConversionMode::FixedValue,
                                value: Some(serde_json::Value::String("success".to_string())),
                                original_path: None,
                            },
                            ten_rust::graph::msg_conversion::MsgConversionRule {
                                path: "result.data".to_string(),
                                conversion_mode: ten_rust::graph::msg_conversion::MsgConversionMode::FromOriginal,
                                value: None,
                                original_path: Some("data".to_string()),
                            },
                        ],
                    },
                },
                result: None,
            }),
        };

        // Mock the connection object to be added - we'll bypass the endpoint
        // and directly call update_graph_connections_all_fields to test
        // msg_conversion.
        let message_flow = GraphMessageFlow {
            name: "test_cmd_conversion".to_string(),
            dest: vec![destination],
        };

        let connection = GraphConnection {
            app: Some("http://example.com:8000".to_string()),
            extension: "extension_1".to_string(),
            cmd: Some(vec![message_flow]),
            data: None,
            audio_frame: None,
            video_frame: None,
        };

        // Get state for testing.
        let state = designer_state.pkgs_cache.get_mut(TEST_DIR).unwrap();
        let app_pkg = state
            .iter_mut()
            .find(|pkg| {
                pkg.manifest
                    .as_ref()
                    .is_some_and(|m| m.type_and_name.pkg_type == PkgType::App)
            })
            .unwrap();

        // Get property to update
        let property = app_pkg.property.as_mut().unwrap();

        // Update the property.all_fields with the new connection
        let update_result = update_graph_connections_all_fields(
            TEST_DIR,
            &mut property.all_fields,
            "default_with_app_uri",
            Some(&[connection.clone()]),
            None,
        );

        assert!(
            update_result.is_ok(),
            "Failed to update graph connections: {:?}",
            update_result
        );

        // Verify the property.all_fields was updated correctly.
        if let Some(ten_value) = property.all_fields.get("_ten") {
            if let Some(predefined_graphs) = ten_value
                .get("predefined_graphs")
                .and_then(|v| v.as_array())
            {
                // Find our graph.
                let graph_value = predefined_graphs
                    .iter()
                    .find(|g| {
                        g.get("name").and_then(|n| n.as_str())
                            == Some("default_with_app_uri")
                    })
                    .unwrap();

                // Check the connections array.
                if let Some(connections) =
                    graph_value.get("connections").and_then(|c| c.as_array())
                {
                    // Find the connection with the matching extension and app.
                    let ext1_connection = connections
                        .iter()
                        .find(|conn| {
                            conn.get("extension").and_then(|e| e.as_str())
                                == Some("extension_1")
                                && conn.get("app").and_then(|a| a.as_str())
                                    == Some("http://example.com:8000")
                        })
                        .unwrap();

                    // Check the cmd array.
                    if let Some(cmd_array) =
                        ext1_connection.get("cmd").and_then(|c| c.as_array())
                    {
                        // Find our command message.
                        let cmd_msg = cmd_array
                            .iter()
                            .find(|c| {
                                c.get("name").and_then(|n| n.as_str())
                                    == Some("test_cmd_conversion")
                            })
                            .unwrap();

                        // Check the dest array.
                        let dest_array = cmd_msg
                            .get("dest")
                            .and_then(|d| d.as_array())
                            .unwrap();
                        let dest = dest_array.first().unwrap();

                        // Verify the destination
                        assert_eq!(
                            dest.get("extension").and_then(|e| e.as_str()),
                            Some("extension_2"),
                            "test_cmd_conversion should point to extension_2"
                        );

                        // Check msg_conversion exists
                        let msg_conversion =
                            dest.get("msg_conversion").unwrap();
                        assert!(
                            msg_conversion.is_object(),
                            "msg_conversion should be an object"
                        );

                        // Check msg_conversion type
                        assert_eq!(
                            msg_conversion.get("type").and_then(|t| t.as_str()),
                            Some("per_property"),
                            "msg_conversion type should be per_property"
                        );

                        // Check keep_original
                        assert_eq!(
                            msg_conversion
                                .get("keep_original")
                                .and_then(|k| k.as_bool()),
                            Some(false),
                            "keep_original should be false"
                        );

                        // Check rules array exists
                        let rules = msg_conversion
                            .get("rules")
                            .and_then(|r| r.as_array())
                            .unwrap();
                        assert_eq!(rules.len(), 2, "There should be 2 rules");

                        // Check first rule
                        let rule1 = rules.first().unwrap();
                        assert_eq!(
                            rule1.get("path").and_then(|p| p.as_str()),
                            Some("result.status"),
                            "First rule path should be result.status"
                        );
                        assert_eq!(
                            rule1
                                .get("conversion_mode")
                                .and_then(|m| m.as_str()),
                            Some("fixed_value"),
                            "First rule conversion_mode should be fixed_value"
                        );
                        assert_eq!(
                            rule1.get("value").and_then(|v| v.as_str()),
                            Some("success"),
                            "First rule value should be 'success'"
                        );

                        // Check second rule
                        let rule2 = rules.get(1).unwrap();
                        assert_eq!(
                            rule2.get("path").and_then(|p| p.as_str()),
                            Some("result.data"),
                            "Second rule path should be result.data"
                        );
                        assert_eq!(
                            rule2.get("conversion_mode").and_then(|m| m.as_str()),
                            Some("from_original"),
                            "Second rule conversion_mode should be from_original"
                        );
                        assert_eq!(
                            rule2.get("original_path").and_then(|p| p.as_str()),
                            Some("data"),
                            "Second rule original_path should be 'data'"
                        );
                    } else {
                        panic!("No cmd array found in the connection");
                    }
                } else {
                    panic!("No connections array found in the graph");
                }
            } else {
                panic!(
                    "No predefined_graphs array found in property.all_fields"
                );
            }
        } else {
            panic!("No _ten object found in property.all_fields");
        }
    }
}
