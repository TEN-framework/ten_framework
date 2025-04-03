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
    graph::{node::GraphNode, Graph},
    pkg_info::{
        pkg_type::PkgType, pkg_type_and_name::PkgTypeAndName,
        property::predefined_graph::PredefinedGraph,
    },
};

use crate::designer::{
    graphs::util,
    response::{ApiResponse, ErrorResponse, Status},
    DesignerState,
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

/// Adds a new extension node to a graph.
fn add_extension_node_to_graph(
    predefined_graph: &PredefinedGraph,
    node_name: &str,
    addon_name: &str,
    app_uri: &Option<String>,
) -> Result<Graph, String> {
    let mut graph = predefined_graph.graph.clone();

    // Add the extension node.
    match graph.add_extension_node(
        node_name.to_string(),
        addon_name.to_string(),
        app_uri.clone(),
        None,
        None,
    ) {
        Ok(_) => Ok(graph),
        Err(e) => Err(e.to_string()),
    }
}

/// Creates a new GraphNode for an extension.
fn create_extension_node(
    node_name: &str,
    addon_name: &str,
    app_uri: &Option<String>,
) -> GraphNode {
    GraphNode {
        type_and_name: PkgTypeAndName {
            pkg_type: PkgType::Extension,
            name: node_name.to_string(),
        },
        addon: addon_name.to_string(),
        extension_group: None,
        app: app_uri.clone(),
        property: None,
    }
}

/// Updates the property.json file with the new graph node.
fn update_node_property_file(
    base_dir: &str,
    property: &mut ten_rust::pkg_info::property::Property,
    graph_name: &str,
    node: &GraphNode,
) -> Result<(), Box<dyn std::error::Error>> {
    let nodes_to_add = vec![node.clone()];
    crate::graph::update_graph_node_all_fields(
        base_dir,
        &mut property.all_fields,
        graph_name,
        Some(&nodes_to_add),
        None,
    )?;
    Ok(())
}

pub async fn add_graph_node_endpoint(
    request_payload: web::Json<AddGraphNodeRequestPayload>,
    state: web::Data<Arc<RwLock<DesignerState>>>,
) -> Result<impl Responder, actix_web::Error> {
    // Get a write lock on the state since we need to modify the graph.
    let mut state_write = state.write().unwrap();

    // Get the packages for this base_dir.
    if let Some(base_dir_pkg_info) =
        state_write.pkgs_cache.get_mut(&request_payload.base_dir)
    {
        // Find the app package.
        if let Some(app_pkg) =
            util::find_app_package_from_base_dir(base_dir_pkg_info)
        {
            // Get the specified graph from predefined_graphs.
            if let Some(predefined_graph) = util::find_predefined_graph(
                app_pkg,
                &request_payload.graph_name,
            ) {
                // Add the node to the graph
                match add_extension_node_to_graph(
                    predefined_graph,
                    &request_payload.node_name,
                    &request_payload.addon_name,
                    &request_payload.app_uri,
                ) {
                    Ok(graph) => {
                        // Update the predefined_graph in the app_pkg.
                        let mut new_graph = predefined_graph.clone();
                        new_graph.graph = graph;
                        app_pkg.update_predefined_graph(&new_graph);

                        // Create the graph node.
                        let new_node = create_extension_node(
                            &request_payload.node_name,
                            &request_payload.addon_name,
                            &request_payload.app_uri,
                        );

                        // Update property.json file with the new graph node.
                        if let Some(property) = &mut app_pkg.property {
                            // Write the updated property_all_fields map to
                            // property.json.
                            if let Err(e) = update_node_property_file(
                                &request_payload.base_dir,
                                property,
                                &request_payload.graph_name,
                                &new_node,
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
    use std::path::Path;

    use actix_web::{test, App};
    use serde_json::Value;
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

        // Read test data from embedded JSON files
        let input_property = include_str!(
            "test_data_embed/input_property_with_ordered_fields.json"
        );
        let input_manifest =
            include_str!("test_data_embed/test_app_manifest.json");
        let expected_property = include_str!(
            "test_data_embed/expected_property_with_new_node.json"
        );

        // Write input files to temp directory.
        let property_path =
            Path::new(&temp_dir_path).join(PROPERTY_JSON_FILENAME);
        fs::write(&property_path, input_property).unwrap();

        let manifest_path = Path::new(&temp_dir_path).join("manifest.json");
        fs::write(&manifest_path, input_manifest).unwrap();

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
}
