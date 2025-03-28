//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::sync::{Arc, RwLock};

use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

use ten_rust::graph::connection::GraphConnection;
use ten_rust::graph::node::GraphNode;
use ten_rust::pkg_info::pkg_type::PkgType;
use ten_rust::pkg_info::predefined_graphs::get_pkg_predefined_graph_from_nodes_and_connections;

use super::{
    connections::GraphConnectionsSingleResponseData,
    nodes::get::GraphNodesSingleResponseData,
};
use crate::designer::response::{ApiResponse, ErrorResponse, Status};
use crate::designer::DesignerState;

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
pub struct GraphUpdateRequestPayload {
    pub base_dir: String,

    pub graph_name: String,

    pub auto_start: bool,
    pub nodes: Vec<GraphNodesSingleResponseData>,
    pub connections: Vec<GraphConnectionsSingleResponseData>,
}

#[derive(Serialize, Debug, PartialEq)]
pub struct GraphUpdateResponseData {
    pub success: bool,
}

pub async fn update_graph_endpoint(
    request_payload: web::Json<GraphUpdateRequestPayload>,
    state: web::Data<Arc<RwLock<DesignerState>>>,
) -> Result<impl Responder, actix_web::Error> {
    let mut state_write = state.write().unwrap();

    if let Some(pkgs) =
        state_write.pkgs_cache.get_mut(&request_payload.base_dir)
    {
        if let Some(app_pkg) = pkgs.iter_mut().find(|pkg| {
            if let Some(manifest) = &pkg.manifest {
                manifest.type_and_name.pkg_type == PkgType::App
            } else {
                false
            }
        }) {
            let graph_name = &request_payload.graph_name;

            // Collect nodes into a Vec<GraphNode>.
            let nodes: Vec<GraphNode> = request_payload
                .nodes
                .iter()
                .cloned()
                .map(|v| v.into())
                .collect();

            // Collect connections into a Vec<GraphConnection>.
            let connections: Vec<GraphConnection> = request_payload
                .connections
                .iter()
                .cloned()
                .map(|v| v.into())
                .collect();

            let new_graph =
                match get_pkg_predefined_graph_from_nodes_and_connections(
                    graph_name,
                    request_payload.auto_start,
                    &nodes,
                    &connections,
                ) {
                    Ok(graph) => graph,
                    Err(err) => {
                        let error_response = ErrorResponse::from_error(
                            &err,
                            "Invalid input data:",
                        );
                        return Ok(
                            HttpResponse::NotFound().json(error_response)
                        );
                    }
                };

            // TODO(Liu): Add graph check.
            app_pkg.update_predefined_graph(&new_graph);

            let response = ApiResponse {
                status: Status::Ok,
                data: GraphUpdateResponseData { success: true },
                meta: None,
            };

            Ok(HttpResponse::Ok().json(response))
        } else {
            let error_response = ErrorResponse {
                status: Status::Fail,
                message: "Failed to find app package.".to_string(),
                error: None,
            };
            Ok(HttpResponse::NotFound().json(error_response))
        }
    } else {
        let error_response = ErrorResponse {
            status: Status::Fail,
            message: "Package information is missing".to_string(),
            error: None,
        };
        Ok(HttpResponse::NotFound().json(error_response))
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, env, fs};

    use actix_web::{test, App};

    use super::*;
    use crate::{
        config::TmanConfig, constants::TEST_DIR,
        designer::mock::inject_all_pkgs_for_mock, output::TmanOutputCli,
    };

    #[actix_web::test]
    async fn test_update_graph_success() {
        // Get current working directory when testing.
        let exe_path =
            env::current_exe().expect("Failed to get the executable path");
        let test_dir = exe_path
            .parent()
            .expect("Failed to get the parent directory");

        env::set_current_dir(test_dir)
            .expect("Failed to set current directory");

        let test_data_dir = test_dir.join("test_data");
        fs::create_dir_all(&test_data_dir)
            .expect("Failed to create test_data directory");

        let mut state = DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
        };

        let all_pkgs_json = vec![
            (
                include_str!("test_data_embed/app_manifest.json").to_string(),
                include_str!("test_data_embed/app_property.json").to_string(),
            ),
            (
                include_str!("test_data_embed/extension_addon_1_manifest.json")
                    .to_string(),
                "{}".to_string(),
            ),
            (
                include_str!("test_data_embed/extension_addon_2_manifest.json")
                    .to_string(),
                "{}".to_string(),
            ),
            (
                include_str!("test_data_embed/extension_addon_3_manifest.json")
                    .to_string(),
                "{}".to_string(),
            ),
        ];

        let inject_ret = inject_all_pkgs_for_mock(
            TEST_DIR,
            &mut state.pkgs_cache,
            all_pkgs_json,
        );
        assert!(inject_ret.is_ok());

        let state = Arc::new(RwLock::new(state));

        let app = test::init_service(
            App::new().app_data(web::Data::new(state.clone())).route(
                "/api/designer/v1/graphs",
                web::put().to(update_graph_endpoint),
            ),
        )
        .await;

        let request_payload: GraphUpdateRequestPayload =
            serde_json::from_str(include_str!(
                "test_data_embed/update_graph_success/request_payload.json"
            ))
            .unwrap();

        let req = test::TestRequest::put()
            .uri("/api/designer/v1/graphs")
            .set_json(request_payload)
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());

        let designer_state = state.read().unwrap();

        let pkgs = designer_state.pkgs_cache.get(TEST_DIR).unwrap();
        let app_pkg = pkgs
            .iter()
            .find(|pkg| {
                if let Some(manifest) = &pkg.manifest {
                    manifest.type_and_name.pkg_type == PkgType::App
                } else {
                    false
                }
            })
            .unwrap();

        let predefined_graphs = app_pkg.get_predefined_graphs().unwrap();
        let predefined_graph = predefined_graphs
            .iter()
            .find(|g| g.name == "default")
            .unwrap();

        assert!(!predefined_graph.auto_start.unwrap());
        assert_eq!(predefined_graph.graph.nodes.len(), 2);
        assert_eq!(
            predefined_graph.graph.connections.as_ref().unwrap().len(),
            1
        );

        let property = app_pkg.property.as_ref().unwrap();
        let property_predefined_graphs = property
            ._ten
            .as_ref()
            .unwrap()
            .predefined_graphs
            .as_ref()
            .unwrap();
        let property_predefined_graph = property_predefined_graphs
            .iter()
            .find(|g| g.name == "default")
            .unwrap();

        assert_eq!(property_predefined_graph.auto_start, Some(false));
        assert_eq!(property_predefined_graph.graph.nodes.len(), 2);
        assert_eq!(
            property_predefined_graph
                .graph
                .connections
                .as_ref()
                .unwrap()
                .len(),
            1
        );
    }
}
