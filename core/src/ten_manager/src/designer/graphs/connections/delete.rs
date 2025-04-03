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
    graph::connection::GraphConnection,
    pkg_info::{
        message::MsgType, predefined_graphs::pkg_predefined_graphs_find,
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
pub struct DeleteGraphConnectionRequestPayload {
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
pub struct DeleteGraphConnectionResponsePayload {
    pub success: bool,
}

pub async fn delete_graph_connection_endpoint(
    request_payload: web::Json<DeleteGraphConnectionRequestPayload>,
    state: web::Data<Arc<RwLock<DesignerState>>>,
) -> Result<impl Responder, actix_web::Error> {
    // Get a write lock on the state since we need to modify the graph.
    let mut state_write = state.write().unwrap();

    // Get the packages for this base_dir.
    if let Some(base_dir_pkg_info) =
        state_write.pkgs_cache.get_mut(&request_payload.base_dir)
    {
        // Find the app package using the utility function.
        if let Some(app_pkg) =
            crate::designer::graphs::util::find_app_package_from_base_dir(
                base_dir_pkg_info,
            )
        {
            // Get the specified graph from predefined_graphs.
            if let Some(predefined_graph) = pkg_predefined_graphs_find(
                app_pkg.get_predefined_graphs(),
                |g| g.name == request_payload.graph_name,
            ) {
                let mut graph = predefined_graph.graph.clone();

                // Delete the connection.
                match graph.delete_connection(
                    request_payload.src_app.clone(),
                    request_payload.src_extension.clone(),
                    request_payload.msg_type.clone(),
                    request_payload.msg_name.clone(),
                    request_payload.dest_app.clone(),
                    request_payload.dest_extension.clone(),
                ) {
                    Ok(_) => {
                        // Update the predefined_graph in the app_pkg.
                        let mut new_graph = predefined_graph.clone();
                        new_graph.graph = graph;
                        app_pkg.update_predefined_graph(&new_graph);

                        // Update property.json file to remove the connection.
                        if let Some(property) = &mut app_pkg.property {
                            // Create a GraphConnection representing what we
                            // want to remove.
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

                            // Set the appropriate message type field.
                            match request_payload.msg_type {
                                MsgType::Cmd => {
                                    connection.cmd = Some(vec![]);
                                }
                                MsgType::Data => {
                                    connection.data = Some(vec![]);
                                }
                                MsgType::AudioFrame => {
                                    connection.audio_frame = Some(vec![]);
                                }
                                MsgType::VideoFrame => {
                                    connection.video_frame = Some(vec![]);
                                }
                            }

                            let connections_to_remove = vec![connection];

                            // Write the updated property_all_fields map to
                            // property.json.
                            if let Err(e) = update_graph_connections_all_fields(
                                &request_payload.base_dir,
                                &mut property.all_fields,
                                &request_payload.graph_name,
                                None,
                                Some(&connections_to_remove),
                            ) {
                                eprintln!("Warning: Failed to update property.json file: {}", e);
                            }
                        }

                        let response = ApiResponse {
                            status: Status::Ok,
                            data: DeleteGraphConnectionResponsePayload {
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
                                "Failed to delete connection: {}",
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
    async fn test_delete_graph_connection_invalid_graph() {
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
                "/api/designer/v1/graphs/connections/delete",
                web::post().to(delete_graph_connection_endpoint),
            ),
        )
        .await;

        // Try to delete a connection from a non-existent graph.
        let request_payload = DeleteGraphConnectionRequestPayload {
            base_dir: TEST_DIR.to_string(),
            graph_name: "non_existent_graph".to_string(),
            src_app: None,
            src_extension: "source_extension".to_string(),
            msg_type: MsgType::Cmd,
            msg_name: "test_message".to_string(),
            dest_app: None,
            dest_extension: "destination_extension".to_string(),
        };

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/graphs/connections/delete")
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
        assert!(response
            .message
            .contains("Graph 'non_existent_graph' not found"));
    }

    #[actix_web::test]
    async fn test_delete_graph_connection_nonexistent_connection() {
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
                "/api/designer/v1/graphs/connections/delete",
                web::post().to(delete_graph_connection_endpoint),
            ),
        )
        .await;

        // Try to delete a non-existent connection.
        let request_payload = DeleteGraphConnectionRequestPayload {
            base_dir: TEST_DIR.to_string(),
            graph_name: "default_with_app_uri".to_string(),
            src_app: None,
            src_extension: "nonexistent_extension".to_string(),
            msg_type: MsgType::Cmd,
            msg_name: "nonexistent_message".to_string(),
            dest_app: None,
            dest_extension: "nonexistent_destination".to_string(),
        };

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/graphs/connections/delete")
            .set_json(request_payload)
            .to_request();
        let resp = test::call_service(&app, req).await;

        // Should fail with a 400 Bad Request for nonexistent connections.
        assert_eq!(resp.status(), 400);

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();
        println!("Response body: {}", body_str);

        let response: ErrorResponse = serde_json::from_str(body_str).unwrap();
        assert_eq!(response.status, Status::Fail);
    }

    #[actix_web::test]
    async fn test_delete_graph_connection_success() {
        // Create a test directory with property.json file.
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_dir_path = temp_dir.path().to_str().unwrap().to_string();

        // Read test data from embedded JSON files.
        let input_property =
            include_str!("../test_data_embed/app_property.json");
        let input_manifest =
            include_str!("../test_data_embed/app_manifest.json");

        // The expected property will be the same as input but with the
        // connection removed.
        let expected_property_str = input_property.replace(
            r#"{"app": "http://example.com:8000", "extension": "extension_1", "cmd": [{"name": "hello_world", "dest": [{"app": "http://example.com:8000", "extension": "extension_2"}]}]}"#,
            r#"{"app": "http://example.com:8000", "extension": "extension_1", "cmd": []}"#);

        // Write input files to temp directory.
        let property_path = std::path::Path::new(&temp_dir_path)
            .join(ten_rust::pkg_info::constants::PROPERTY_JSON_FILENAME);
        std::fs::write(&property_path, input_property).unwrap();

        let manifest_path =
            std::path::Path::new(&temp_dir_path).join("manifest.json");
        std::fs::write(&manifest_path, input_manifest).unwrap();

        // Initialize test state.
        let mut designer_state = DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
        };

        // Inject the test app into the mock.
        let all_pkgs_json = vec![(
            std::fs::read_to_string(&manifest_path).unwrap(),
            std::fs::read_to_string(&property_path).unwrap(),
        )];

        let inject_ret = inject_all_pkgs_for_mock(
            &temp_dir_path,
            &mut designer_state.pkgs_cache,
            all_pkgs_json,
        );
        assert!(inject_ret.is_ok());

        let designer_state = Arc::new(RwLock::new(designer_state));

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(designer_state.clone()))
                .route(
                    "/api/designer/v1/graphs/connections/delete",
                    web::post().to(delete_graph_connection_endpoint),
                ),
        )
        .await;

        // Delete a connection from the default_with_app_uri graph.
        let request_payload = DeleteGraphConnectionRequestPayload {
            base_dir: temp_dir_path.clone(),
            graph_name: "default_with_app_uri".to_string(),
            src_app: Some("http://example.com:8000".to_string()),
            src_extension: "extension_1".to_string(),
            msg_type: MsgType::Cmd,
            msg_name: "hello_world".to_string(),
            dest_app: Some("http://example.com:8000".to_string()),
            dest_extension: "extension_2".to_string(),
        };

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/graphs/connections/delete")
            .set_json(request_payload)
            .to_request();
        let resp = test::call_service(&app, req).await;

        // Should succeed with a 200 OK
        assert_eq!(resp.status(), 200);

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();
        println!("Response body: {}", body_str);

        let response: ApiResponse<DeleteGraphConnectionResponsePayload> =
            serde_json::from_str(body_str).unwrap();
        assert_eq!(response.status, Status::Ok);
        assert!(response.data.success);

        // Verify the connection was actually removed from the data
        let state_read = designer_state.read().unwrap();
        if let Some(base_dir_pkg_info) =
            state_read.pkgs_cache.get(&temp_dir_path)
        {
            if let Some(app_pkg) = &base_dir_pkg_info.app_pkg_info {
                if let Some(predefined_graph) = pkg_predefined_graphs_find(
                    app_pkg.get_predefined_graphs(),
                    |g| g.name == "default_with_app_uri",
                ) {
                    // Check if the connection is gone.
                    let connection_exists = predefined_graph
                        .graph
                        .connections
                        .as_ref()
                        .is_some_and(|connections| {
                            connections.iter().any(|conn| {
                                conn.extension == "extension_1"
                                    && conn.app.as_ref().is_some_and(|app| {
                                        app == "http://example.com:8000"
                                    })
                                    && conn.cmd.as_ref().is_some_and(|cmds| {
                                        cmds.iter().any(|cmd| {
                                            cmd.name == "hello_world"
                                        })
                                    })
                            })
                        });
                    assert!(
                        !connection_exists,
                        "Connection should have been deleted"
                    );
                } else {
                    panic!("Graph 'default_with_app_uri' not found");
                }
            } else {
                panic!("App package not found");
            }
        } else {
            panic!("Base directory not found");
        }

        // Read the updated property.json file.
        let updated_property_content =
            std::fs::read_to_string(&property_path).unwrap();

        // Parse the contents as JSON for proper comparison.
        let updated_property: serde_json::Value =
            serde_json::from_str(&updated_property_content).unwrap();
        let expected_property: serde_json::Value =
            serde_json::from_str(&expected_property_str).unwrap();

        // Compare the updated property with the expected property.
        assert_eq!(
            updated_property, expected_property,
            "Updated property does not match expected property"
        );
    }
}
