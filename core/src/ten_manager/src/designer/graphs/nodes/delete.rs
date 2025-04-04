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
    graph::node::GraphNode,
    pkg_info::{
        pkg_type::PkgType, pkg_type_and_name::PkgTypeAndName,
        predefined_graphs::pkg_predefined_graphs_find,
    },
};

use crate::{
    designer::{
        graphs::util::find_app_package_from_base_dir,
        response::{ApiResponse, ErrorResponse, Status},
        DesignerState,
    },
    graph::update_graph_node_all_fields,
};

#[derive(Serialize, Deserialize)]
pub struct DeleteGraphNodeRequestPayload {
    pub base_dir: String,
    pub graph_name: String,
    pub node_name: String,
    pub addon_name: String,
    pub extension_group_name: Option<String>,
    pub app_uri: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct DeleteGraphNodeResponsePayload {
    pub success: bool,
}

pub async fn delete_graph_node_endpoint(
    request_payload: web::Json<DeleteGraphNodeRequestPayload>,
    state: web::Data<Arc<RwLock<DesignerState>>>,
) -> Result<impl Responder, actix_web::Error> {
    // Get a write lock on the state since we need to modify the graph.
    let mut state_write = state.write().unwrap();

    // Get the packages for this base_dir.
    if let Some(base_dir_pkg_info) =
        state_write.pkgs_cache.get_mut(&request_payload.base_dir)
    {
        // Find the app package.
        if let Some(app_pkg) = find_app_package_from_base_dir(base_dir_pkg_info)
        {
            // Get the specified graph from predefined_graphs.
            if let Some(predefined_graph) = pkg_predefined_graphs_find(
                app_pkg.get_predefined_graphs(),
                |g| g.name == request_payload.graph_name,
            ) {
                let mut graph = predefined_graph.graph.clone();

                // Delete the extension node.
                match graph.delete_extension_node(
                    request_payload.node_name.clone(),
                    request_payload.addon_name.clone(),
                    request_payload.app_uri.clone(),
                    request_payload.extension_group_name.clone(),
                ) {
                    Ok(_) => {
                        // Update the predefined_graph in the app_pkg.
                        let mut new_graph = predefined_graph.clone();
                        new_graph.graph = graph;
                        app_pkg.update_predefined_graph(&new_graph);

                        // Update property.json file to remove the graph node.
                        if let Some(property) = &mut app_pkg.property {
                            // Create the GraphNode we want to remove.
                            let node_to_remove = GraphNode {
                                type_and_name: PkgTypeAndName {
                                    pkg_type: PkgType::Extension,
                                    name: request_payload.node_name.clone(),
                                },
                                addon: request_payload.addon_name.clone(),
                                extension_group: request_payload
                                    .extension_group_name
                                    .clone(),
                                app: request_payload.app_uri.clone(),
                                property: None,
                            };
                            let nodes_to_remove = vec![node_to_remove];

                            // Write the updated property_all_fields map to
                            // property.json.
                            if let Err(e) = update_graph_node_all_fields(
                                &request_payload.base_dir,
                                &mut property.all_fields,
                                &request_payload.graph_name,
                                None,
                                Some(&nodes_to_remove),
                            ) {
                                eprintln!("Warning: Failed to update property.json file: {}", e);
                            }
                        }

                        let response = ApiResponse {
                            status: Status::Ok,
                            data: DeleteGraphNodeResponsePayload {
                                success: true,
                            },
                            meta: None,
                        };
                        Ok(HttpResponse::Ok().json(response))
                    }
                    Err(err) => {
                        let error_response = ErrorResponse {
                            status: Status::Fail,
                            message: format!("Failed to delete node: {}", err),
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
    async fn test_delete_graph_node_invalid_graph() {
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
                "/api/designer/v1/graphs/nodes/delete",
                web::post().to(delete_graph_node_endpoint),
            ),
        )
        .await;

        // Try to delete a node from a non-existent graph.
        let request_payload = DeleteGraphNodeRequestPayload {
            base_dir: TEST_DIR.to_string(),
            graph_name: "non_existent_graph".to_string(),
            node_name: "test_node".to_string(),
            addon_name: "test_addon".to_string(),
            extension_group_name: None,
            app_uri: Some("http://test-app-uri.com".to_string()),
        };

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/graphs/nodes/delete")
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
    async fn test_delete_graph_node_nonexistent_node() {
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
                "/api/designer/v1/graphs/nodes/delete",
                web::post().to(delete_graph_node_endpoint),
            ),
        )
        .await;

        // Try to delete a non-existent node from an existing graph.
        let request_payload = DeleteGraphNodeRequestPayload {
            base_dir: TEST_DIR.to_string(),
            graph_name: "default_with_app_uri".to_string(),
            node_name: "non_existent_node".to_string(),
            addon_name: "test_addon".to_string(),
            extension_group_name: None,
            app_uri: Some("http://example.com:8000".to_string()),
        };

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/graphs/nodes/delete")
            .set_json(request_payload)
            .to_request();
        let resp = test::call_service(&app, req).await;

        // Should succeed with 200 OK, since deleting a non-existent node is not
        // an error (the node is already gone).
        assert_eq!(resp.status(), 200);

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();
        println!("Response body: {}", body_str);

        let response: ApiResponse<DeleteGraphNodeResponsePayload> =
            serde_json::from_str(body_str).unwrap();
        assert_eq!(response.status, Status::Ok);
        assert!(response.data.success);
    }

    #[actix_web::test]
    async fn test_delete_graph_node_success() {
        // Create a test directory with property.json file.
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_dir_path = temp_dir.path().to_str().unwrap().to_string();

        // Read test data from embedded JSON files.
        let input_property =
            include_str!("../test_data_embed/app_property.json");
        let input_manifest =
            include_str!("../test_data_embed/app_manifest.json");

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

        // First add a node, then delete it.
        // Setup the add endpoint.
        let app_add = test::init_service(
            App::new().app_data(web::Data::new(designer_state.clone())).route(
                "/api/designer/v1/graphs/nodes/add",
                web::post().to(crate::designer::graphs::nodes::add::add_graph_node_endpoint),
            ),
        )
        .await;

        // Add a node to the default graph.
        let add_request_payload =
            crate::designer::graphs::nodes::add::AddGraphNodeRequestPayload {
                base_dir: temp_dir_path.clone(),
                graph_name: "default_with_app_uri".to_string(),
                node_name: "test_delete_node".to_string(),
                addon_name: "test_addon".to_string(),
                app_uri: Some("http://example.com:8000".to_string()),
            };

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/graphs/nodes/add")
            .set_json(add_request_payload)
            .to_request();
        let resp = test::call_service(&app_add, req).await;

        // Ensure add was successful.
        assert_eq!(resp.status(), 200);

        let updated_property_content =
            std::fs::read_to_string(&property_path).unwrap();

        let expected_property_content =
            include_str!("test_data_embed/expected_property_after_adding_in_test_delete_graph_node_success.json");

        // Parse the contents as JSON for proper comparison.
        let updated_property: serde_json::Value =
            serde_json::from_str(&updated_property_content).unwrap();
        let expected_property: serde_json::Value =
            serde_json::from_str(expected_property_content).unwrap();

        // Compare the updated property with the expected property
        assert_eq!(
            updated_property, expected_property,
            "Updated property does not match expected property"
        );

        // Setup the delete endpoint.
        let app_delete = test::init_service(
            App::new()
                .app_data(web::Data::new(designer_state.clone()))
                .route(
                    "/api/designer/v1/graphs/nodes/delete",
                    web::post().to(delete_graph_node_endpoint),
                ),
        )
        .await;

        // Now delete the node we just added.
        let delete_request_payload = DeleteGraphNodeRequestPayload {
            base_dir: temp_dir_path.clone(),
            graph_name: "default_with_app_uri".to_string(),
            node_name: "test_delete_node".to_string(),
            addon_name: "test_addon".to_string(),
            extension_group_name: None,
            app_uri: Some("http://example.com:8000".to_string()),
        };

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/graphs/nodes/delete")
            .set_json(delete_request_payload)
            .to_request();
        let resp = test::call_service(&app_delete, req).await;

        // Should succeed with a 200 OK.
        assert_eq!(resp.status(), 200);

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();
        println!("Response body: {}", body_str);

        let response: ApiResponse<DeleteGraphNodeResponsePayload> =
            serde_json::from_str(body_str).unwrap();
        assert_eq!(response.status, Status::Ok);
        assert!(response.data.success);

        // Verify the node was actually removed from the data.
        let state_read = designer_state.read().unwrap();
        if let Some(base_dir_pkg_info) =
            state_read.pkgs_cache.get(&temp_dir_path)
        {
            if let Some(app_pkg) = &base_dir_pkg_info.app_pkg_info {
                if let Some(predefined_graph) = pkg_predefined_graphs_find(
                    app_pkg.get_predefined_graphs(),
                    |g| g.name == "default_with_app_uri",
                ) {
                    // Check if the node is gone.
                    let node_exists =
                        predefined_graph.graph.nodes.iter().any(|node| {
                            node.type_and_name.name == "test_delete_node"
                                && node.addon == "test_addon"
                        });
                    assert!(!node_exists, "Node should have been deleted");
                } else {
                    panic!("Graph 'default_with_app_uri' not found");
                }
            } else {
                panic!("App package not found");
            }
        } else {
            panic!("Base directory not found");
        }

        let updated_property_content =
            std::fs::read_to_string(&property_path).unwrap();

        // Parse the contents as JSON for proper comparison.
        let updated_property: serde_json::Value =
            serde_json::from_str(&updated_property_content).unwrap();
        let expected_property: serde_json::Value =
            serde_json::from_str(input_property).unwrap();

        // Compare the updated property with the expected property
        assert_eq!(
            updated_property, expected_property,
            "Updated property does not match expected property"
        );
    }
}
