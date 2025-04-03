//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use actix_web::{web, HttpResponse, Responder};
use anyhow::Result;
use serde::{Deserialize, Serialize};

use ten_rust::{
    base_dir_pkg_info::BaseDirPkgInfo,
    graph::connection::{GraphConnection, GraphDestination, GraphMessageFlow},
    pkg_info::message::MsgType,
};

use crate::designer::graphs::util::find_predefined_graph;
use crate::designer::{
    graphs::util::find_app_package_from_base_dir,
    response::{ApiResponse, ErrorResponse, Status},
    DesignerState,
};

use crate::graph::update_graph_connections_all_fields;

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

/// Create a new GraphConnection from request params.
fn create_graph_connection(
    request_payload: &AddGraphConnectionRequestPayload,
) -> GraphConnection {
    // Create the destination.
    let destination = GraphDestination {
        app: request_payload.dest_app.clone(),
        extension: request_payload.dest_extension.clone(),
        msg_conversion: None,
    };

    // Create the message flow.
    let message_flow = GraphMessageFlow {
        name: request_payload.msg_name.clone(),
        dest: vec![destination],
    };

    // Create the connection object.
    let mut connection = GraphConnection {
        app: request_payload.src_app.clone(),
        extension: request_payload.src_extension.clone(),
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
            connection.audio_frame = Some(vec![message_flow]);
        }
        MsgType::VideoFrame => {
            connection.video_frame = Some(vec![message_flow]);
        }
    }

    connection
}

/// Update the property.json file with the new connection.
fn update_property_file(
    base_dir: &str,
    property: &mut ten_rust::pkg_info::property::Property,
    graph_name: &str,
    connection: &GraphConnection,
) -> Result<()> {
    // Update only with the new connection.
    let connections_to_add = vec![connection.clone()];

    // Update the property_all_fields map and write to property.json.
    update_graph_connections_all_fields(
        base_dir,
        &mut property.all_fields,
        graph_name,
        Some(&connections_to_add),
        None,
    )
}

pub async fn add_graph_connection_endpoint(
    request_payload: web::Json<AddGraphConnectionRequestPayload>,
    state: web::Data<Arc<RwLock<DesignerState>>>,
) -> Result<impl Responder, actix_web::Error> {
    let mut state_write = state.write().unwrap();

    // Clone the pkgs_cache for this base_dir.
    let pkgs_cache_clone = state_write.pkgs_cache.clone();

    // Create a hash map from app URIs to BaseDirPkgInfo for use with
    // add_connection.
    let mut uri_to_pkg_info: HashMap<String, BaseDirPkgInfo> = HashMap::new();

    // Process all available apps to map URIs to BaseDirPkgInfo.
    for base_dir_pkg_info in pkgs_cache_clone.values() {
        if let Some(app_pkg) = &base_dir_pkg_info.app_pkg_info {
            if let Some(property) = &app_pkg.property {
                if let Some(ten) = &property._ten {
                    if let Some(uri) = &ten.uri {
                        // Map the URI to the BaseDirPkgInfo.
                        uri_to_pkg_info
                            .insert(uri.clone(), base_dir_pkg_info.clone());
                    }
                }
            }
        }
    }

    // Get the packages for this base_dir.
    if let Some(base_dir_pkg_info) =
        state_write.pkgs_cache.get_mut(&request_payload.base_dir)
    {
        // Find the app package.
        if let Some(app_pkg) = find_app_package_from_base_dir(base_dir_pkg_info)
        {
            // Get the specified graph from predefined_graphs.
            if let Some(predefined_graph) =
                find_predefined_graph(app_pkg, &request_payload.graph_name)
            {
                let mut graph = predefined_graph.graph.clone();

                // Add the connection using the converted BaseDirPkgInfo map.
                match graph.add_connection(
                    request_payload.src_app.clone(),
                    request_payload.src_extension.clone(),
                    request_payload.msg_type.clone(),
                    request_payload.msg_name.clone(),
                    request_payload.dest_app.clone(),
                    request_payload.dest_extension.clone(),
                    &uri_to_pkg_info,
                ) {
                    Ok(_) => {
                        // Update the predefined_graph in the app_pkg.
                        let mut new_graph = predefined_graph.clone();
                        new_graph.graph = graph;
                        app_pkg.update_predefined_graph(&new_graph);

                        // Update property.json file with the updated graph.
                        if let Some(property) = &mut app_pkg.property {
                            // Create a new connection object.
                            let connection =
                                create_graph_connection(&request_payload);

                            // Update the property.json file.
                            if let Err(e) = update_property_file(
                                &request_payload.base_dir,
                                property,
                                &request_payload.graph_name,
                                &connection,
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
        config::TmanConfig, designer::mock::inject_all_pkgs_for_mock,
        output::TmanOutputCli,
    };

    #[actix_web::test]
    async fn test_add_graph_connection_success() {
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

        // The empty property for addons
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
            base_dir: test_dir.clone(),
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

        // Define expected property.json content after adding the connection.
        let expected_property = include_str!("test_data_embed/expected_json__test_add_graph_connection_success.json");

        // Read the actual property.json file generated during the test.
        let property_path =
            std::path::Path::new(&test_dir).join(PROPERTY_JSON_FILENAME);
        let actual_property = std::fs::read_to_string(property_path).unwrap();

        // Normalize both JSON strings to handle formatting differences.
        let expected_value: serde_json::Value =
            serde_json::from_str(expected_property).unwrap();
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
        };

        // Create a temporary directory for our test to store the generated
        // property.json.
        let temp_dir = tempfile::tempdir().unwrap();
        let test_dir = temp_dir.path().to_str().unwrap().to_string();

        let all_pkgs_json = vec![(
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
            &test_dir,
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
            base_dir: test_dir.to_string(),
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
        let expected_property = include_str!("test_data_embed/expected_json__test_add_graph_connection_preserves_order.json");

        // Read the actual property.json file generated during the test.
        let property_path =
            std::path::Path::new(&test_dir).join(PROPERTY_JSON_FILENAME);
        let actual_property = std::fs::read_to_string(property_path).unwrap();

        // Normalize both JSON strings to handle formatting differences.
        let expected_value: serde_json::Value =
            serde_json::from_str(expected_property).unwrap();
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

        // The empty property for addons.
        let empty_property = r#"{"_ten":{}}"#.to_string();

        let all_pkgs_json = vec![
            (app_manifest, app_property),
            (ext1_manifest, empty_property.clone()),
            (ext2_manifest, empty_property),
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

        // Add a DATA type connection between extensions.
        let request_payload = AddGraphConnectionRequestPayload {
            base_dir: test_dir.clone(),
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

        // Define expected property.json content after adding the connection.
        let expected_property = include_str!("test_data_embed/expected_json__test_add_graph_connection_data_type.json");

        // Read the actual property.json file generated during the test.
        let property_path =
            std::path::Path::new(&test_dir).join(PROPERTY_JSON_FILENAME);
        let actual_property = std::fs::read_to_string(property_path).unwrap();

        // Normalize both JSON strings to handle formatting differences.
        let expected_value: serde_json::Value =
            serde_json::from_str(expected_property).unwrap();
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

        // The empty property for addons.
        let empty_property = r#"{"_ten":{}}"#.to_string();

        let all_pkgs_json = vec![
            (app_manifest, app_property),
            (ext1_manifest, empty_property.clone()),
            (ext2_manifest, empty_property),
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

        // First add an AUDIO_FRAME type connection.
        let audio_request = AddGraphConnectionRequestPayload {
            base_dir: test_dir.clone(),
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
            base_dir: test_dir.clone(),
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

        // Define expected property.json content after adding both connections.
        let expected_property = include_str!("test_data_embed/expected_json__test_add_graph_connection_frame_types.json");

        // Read the actual property.json file generated during the test.
        let property_path =
            std::path::Path::new(&test_dir).join(PROPERTY_JSON_FILENAME);
        let actual_property = std::fs::read_to_string(property_path).unwrap();

        // Normalize both JSON strings to handle formatting differences.
        let expected_value: serde_json::Value =
            serde_json::from_str(expected_property).unwrap();
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

        // Add first command.
        let cmd1_request = AddGraphConnectionRequestPayload {
            base_dir: test_dir.clone(),
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
            base_dir: test_dir.clone(),
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
            base_dir: test_dir.clone(),
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

        // Define expected property.json content after adding all three
        // connections.
        let expected_property = include_str!("test_data_embed/expected_json__test_add_multiple_connections_preservation_order.json");

        // Read the actual property.json file generated during the test.
        let property_path =
            std::path::Path::new(&test_dir).join(PROPERTY_JSON_FILENAME);
        let actual_property = std::fs::read_to_string(property_path).unwrap();

        // Normalize both JSON strings to handle formatting differences.
        let expected_value: serde_json::Value =
            serde_json::from_str(expected_property).unwrap();
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
