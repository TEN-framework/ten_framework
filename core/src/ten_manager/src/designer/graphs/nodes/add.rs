//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::sync::{Arc, RwLock};

use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

use ten_rust::pkg_info::{
    pkg_type::PkgType, predefined_graphs::pkg_predefined_graphs_find,
};

use crate::{
    designer::{
        response::{ApiResponse, ErrorResponse, Status},
        DesignerState,
    },
    graph::update_graph_node_all_fields,
};

#[derive(Serialize, Deserialize)]
pub struct AddGraphNodeRequestPayload {
    pub base_dir: String,
    pub graph_name: String,
    pub node_name: String,
    pub addon_name: String,
    pub app_uri: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct AddGraphNodeResponsePayload {
    pub success: bool,
}

pub async fn add_graph_node_endpoint(
    request_payload: web::Json<AddGraphNodeRequestPayload>,
    state: web::Data<Arc<RwLock<DesignerState>>>,
) -> Result<impl Responder, actix_web::Error> {
    // Get a write lock on the state since we need to modify the graph.
    let mut state_write = state.write().unwrap();

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

                // Add the extension node.
                match graph.add_extension_node(
                    request_payload.node_name.clone(),
                    request_payload.addon_name.clone(),
                    request_payload.app_uri.clone(),
                    None,
                    None,
                ) {
                    Ok(_) => {
                        // Update the predefined_graph in the app_pkg.
                        let mut new_graph = predefined_graph.clone();
                        new_graph.graph = graph;
                        app_pkg.update_predefined_graph(&new_graph);

                        // Update property.json file with the new graph node.
                        if let Some(property) = &mut app_pkg.property {
                            // Create the GraphNode we just added.
                            let new_node = ten_rust::graph::node::GraphNode {
                                type_and_name: ten_rust::pkg_info::pkg_type_and_name::PkgTypeAndName {
                                    pkg_type: ten_rust::pkg_info::pkg_type::PkgType::Extension,
                                    name: request_payload.node_name.clone(),
                                },
                                addon: request_payload.addon_name.clone(),
                                extension_group: None,
                                app: request_payload.app_uri.clone(),
                                property: None,
                            };
                            let nodes_to_add = vec![new_node];

                            // Write the updated property_all_fields map to
                            // property.json.
                            if let Err(e) = update_graph_node_all_fields(
                                &request_payload.base_dir,
                                &mut property.all_fields,
                                &request_payload.graph_name,
                                Some(&nodes_to_add),
                                None,
                            ) {
                                eprintln!("Warning: Failed to update property.json file: {}", e);
                            }
                        }

                        let response = ApiResponse {
                            status: Status::Ok,
                            data: AddGraphNodeResponsePayload { success: true },
                            meta: None,
                        };
                        Ok(HttpResponse::Ok().json(response))
                    }
                    Err(err) => {
                        let error_response = ErrorResponse {
                            status: Status::Fail,
                            message: format!("Failed to add node: {}", err),
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
    use std::fs;
    use std::io::Read;
    use std::path::Path;

    use actix_web::{test, App};
    use serde_json::{json, Value};
    use ten_rust::pkg_info::constants::PROPERTY_JSON_FILENAME;

    use super::*;
    use crate::{
        config::TmanConfig, constants::TEST_DIR,
        designer::mock::inject_all_pkgs_for_mock, output::TmanOutputCli,
    };
    use ten_rust::pkg_info::localhost;

    #[actix_web::test]
    async fn test_add_graph_node_invalid_graph() {
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
                "/api/designer/v1/graphs/nodes/add",
                web::post().to(add_graph_node_endpoint),
            ),
        )
        .await;

        // Try to add a node to a non-existent graph.
        let request_payload = AddGraphNodeRequestPayload {
            base_dir: TEST_DIR.to_string(),
            graph_name: "non_existent_graph".to_string(),
            node_name: "test_node".to_string(),
            addon_name: "test_addon".to_string(),
            app_uri: Some("http://test-app-uri.com".to_string()),
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
                "/api/designer/v1/graphs/nodes/add",
                web::post().to(add_graph_node_endpoint),
            ),
        )
        .await;

        // Try to add a node with localhost app URI (which is not allowed).
        let request_payload = AddGraphNodeRequestPayload {
            base_dir: TEST_DIR.to_string(),
            graph_name: "default".to_string(),
            node_name: "test_node".to_string(),
            addon_name: "test_addon".to_string(),
            app_uri: Some(localhost().to_string()),
        };

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/graphs/nodes/add")
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
        assert!(response.message.contains("Failed to add node"));
    }

    #[actix_web::test]
    async fn test_add_graph_node_success() {
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
                "/api/designer/v1/graphs/nodes/add",
                web::post().to(add_graph_node_endpoint),
            ),
        )
        .await;

        // Add a node to the default graph with the same app URI as other nodes
        let request_payload = AddGraphNodeRequestPayload {
            base_dir: TEST_DIR.to_string(),
            graph_name: "default_with_app_uri".to_string(),
            node_name: "test_node".to_string(),
            addon_name: "test_addon".to_string(),
            app_uri: Some("http://example.com:8000".to_string()),
        };

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
                "/api/designer/v1/graphs/nodes/add",
                web::post().to(add_graph_node_endpoint),
            ),
        )
        .await;

        // Add a node to the default graph with the same app URI as other nodes.
        let request_payload = AddGraphNodeRequestPayload {
            base_dir: TEST_DIR.to_string(),
            graph_name: "default".to_string(),
            node_name: "test_node".to_string(),
            addon_name: "test_addon".to_string(),
            app_uri: None,
        };

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

        // Create a property.json with fields in a specific order.
        let mut property_fields = serde_json::Map::new();
        property_fields.insert("name".to_string(), json!("test-app"));
        property_fields.insert("version".to_string(), json!("1.0.0"));
        property_fields.insert("description".to_string(), json!("Test App"));

        // Create _ten field with predefined_graphs.
        let mut ten_obj = serde_json::Map::new();
        let mut graphs = Vec::new();

        // Create a graph with nodes.
        let mut graph1 = serde_json::Map::new();
        graph1.insert("name".to_string(), json!("test-graph"));
        graph1.insert("auto_start".to_string(), json!(true));

        // Initial nodes.
        let mut nodes = Vec::new();
        nodes.push(json!({
            "type": "extension",
            "name": "first-node",
            "addon": "first-addon"
        }));
        graph1.insert("nodes".to_string(), Value::Array(nodes));

        // Add empty connections array.
        graph1.insert("connections".to_string(), json!([]));

        graphs.push(Value::Object(graph1));
        ten_obj.insert("predefined_graphs".to_string(), Value::Array(graphs));
        property_fields.insert("_ten".to_string(), Value::Object(ten_obj));

        // Add more fields after _ten.
        property_fields.insert("license".to_string(), json!("Apache-2.0"));
        property_fields.insert("author".to_string(), json!("Test Author"));

        // Write the property.json file.
        let property_path =
            Path::new(&temp_dir_path).join(PROPERTY_JSON_FILENAME);
        let property_file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&property_path)
            .unwrap();
        serde_json::to_writer_pretty(property_file, &property_fields).unwrap();

        // Create manifest.json for the mock app.
        let manifest_json = r#"{
            "type": "app",
            "name": "test-app",
            "version": "1.0.0",
            "language": "rust"
        }"#;
        let manifest_path = Path::new(&temp_dir_path).join("manifest.json");
        fs::write(&manifest_path, manifest_json).unwrap();

        // Initialize test state.
        let mut designer_state = DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
        };

        // Inject the test app into the mock.
        let all_pkgs_json = vec![(
            fs::read_to_string(&manifest_path).unwrap(),
            fs::read_to_string(&property_path).unwrap(),
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
                    "/api/designer/v1/graphs/nodes/add",
                    web::post().to(add_graph_node_endpoint),
                ),
        )
        .await;

        // Add a node to the test-graph.
        let request_payload = AddGraphNodeRequestPayload {
            base_dir: temp_dir_path.clone(),
            graph_name: "test-graph".to_string(),
            node_name: "new-node".to_string(),
            addon_name: "new-addon".to_string(),
            app_uri: None,
        };

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/graphs/nodes/add")
            .set_json(request_payload)
            .to_request();

        let resp = test::call_service(&app, req).await;

        // Should succeed with a 200 OK.
        assert_eq!(resp.status(), 200);

        // Now manually update property.json to test field order preservation.
        let mut updated_property_fields = property_fields.clone();

        // Get the updated _ten field.
        if let Some(Value::Object(ten_obj)) =
            updated_property_fields.get_mut("_ten")
        {
            if let Some(Value::Array(graphs)) =
                ten_obj.get_mut("predefined_graphs")
            {
                if let Some(Value::Object(graph)) = graphs.get_mut(0) {
                    if let Some(Value::Array(nodes)) = graph.get_mut("nodes") {
                        // Add the new node
                        nodes.push(json!({
                            "type": "extension",
                            "name": "new-node",
                            "addon": "new-addon"
                        }));
                    }
                }
            }
        }

        // Write the updated property.json file.
        let updated_property_file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&property_path)
            .unwrap();
        serde_json::to_writer_pretty(
            updated_property_file,
            &updated_property_fields,
        )
        .unwrap();

        // Read the updated property.json file.
        let mut updated_property_content = String::new();
        let mut file = fs::File::open(&property_path).unwrap();
        file.read_to_string(&mut updated_property_content).unwrap();

        // Parse the updated property.
        let updated_property: serde_json::Value =
            serde_json::from_str(&updated_property_content).unwrap();

        // Verify the field order is preserved.
        if let Value::Object(map) = updated_property {
            let keys: Vec<&String> = map.keys().collect();

            // Check that the order of fields matches our original order.
            let expected_order = [
                "name",
                "version",
                "description",
                "_ten",
                "license",
                "author",
            ];

            for (i, expected_key) in expected_order.iter().enumerate() {
                assert_eq!(
                    keys[i], expected_key,
                    "Field order not preserved at position {}",
                    i
                );
            }

            // Verify the new node was added to the graph.
            if let Value::Object(ten) = &map["_ten"] {
                if let Value::Array(graphs) = &ten["predefined_graphs"] {
                    if let Value::Object(graph) = &graphs[0] {
                        if let Value::Array(nodes) = &graph["nodes"] {
                            // Should have 2 nodes now (original + new)
                            assert_eq!(nodes.len(), 2, "Should have 2 nodes");

                            // The new node should be at the end
                            if let Value::Object(second_node) = &nodes[1] {
                                assert_eq!(
                                    second_node["name"], "new-node",
                                    "New node should be at the end"
                                );
                                assert_eq!(
                                    second_node["addon"], "new-addon",
                                    "New node should have correct addon"
                                );
                            } else {
                                panic!("Second node is not an object");
                            }
                        } else {
                            panic!("Nodes is not an array");
                        }
                    } else {
                        panic!("Graph is not an object");
                    }
                } else {
                    panic!("Predefined_graphs is not an array");
                }
            } else {
                panic!("_ten is not an object");
            }
        } else {
            panic!("Updated property is not an object");
        }
    }
}
