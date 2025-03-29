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

use crate::designer::{
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

    use actix_web::{test, App};

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
            app_uri: Some(localhost()),
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

    // #[actix_web::test]
    // async fn test_add_graph_node_without_app_uri_success() {
    //     let mut designer_state = DesignerState {
    //         tman_config: Arc::new(TmanConfig::default()),
    //         out: Arc::new(Box::new(TmanOutputCli)),
    //         pkgs_cache: HashMap::new(),
    //     };

    //     let all_pkgs_json = vec![(
    //         include_str!("../test_data_embed/app_manifest.json").to_string(),
    //         include_str!("../test_data_embed/app_property.json").to_string(),
    //     )];

    //     let inject_ret = inject_all_pkgs_for_mock(
    //         TEST_DIR,
    //         &mut designer_state.pkgs_cache,
    //         all_pkgs_json,
    //     );
    //     assert!(inject_ret.is_ok());

    //     let designer_state = Arc::new(RwLock::new(designer_state));

    //     let app = test::init_service(
    //         App::new().app_data(web::Data::new(designer_state)).route(
    //             "/api/designer/v1/graphs/nodes/add",
    //             web::post().to(add_graph_node_endpoint),
    //         ),
    //     )
    //     .await;

    //     // Add a node to the default graph with the same app URI as other
    // nodes     let request_payload = AddGraphNodeRequestPayload {
    //         base_dir: TEST_DIR.to_string(),
    //         graph_name: "default".to_string(),
    //         node_name: "test_node".to_string(),
    //         addon_name: "test_addon".to_string(),
    //         app_uri: None,
    //     };

    //     let req = test::TestRequest::post()
    //         .uri("/api/designer/v1/graphs/nodes/add")
    //         .set_json(request_payload)
    //         .to_request();

    //     let resp = test::call_service(&app, req).await;

    //     let status = resp.status();
    //     println!("Response status: {}", status);

    //     let body = test::read_body(resp).await;
    //     let body_str = std::str::from_utf8(&body).unwrap();
    //     println!("Response body: {}", body_str);

    //     // Should succeed with a 200 OK.
    //     assert_eq!(status, 200);

    //     let response: ApiResponse<AddGraphNodeResponsePayload> =
    //         serde_json::from_str(body_str).unwrap();
    //     assert_eq!(response.status, Status::Ok);
    //     assert!(response.data.success);
    // }
}
