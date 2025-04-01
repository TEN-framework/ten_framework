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
}
