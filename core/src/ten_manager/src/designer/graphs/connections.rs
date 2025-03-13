//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::sync::{Arc, RwLock};

use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

use ten_rust::pkg_info::graph::msg_conversion::MsgAndResultConversion;
use ten_rust::pkg_info::graph::{
    GraphConnection, GraphDestination, GraphMessageFlow,
};
use ten_rust::pkg_info::pkg_type::PkgType;
use ten_rust::pkg_info::predefined_graphs::pkg_predefined_graphs_find;

use crate::designer::app::base_dir::get_base_dir_from_pkgs_cache;
use crate::designer::response::{ApiResponse, ErrorResponse, Status};
use crate::designer::DesignerState;

#[derive(Serialize, Deserialize)]
pub struct GetGraphConnectionsRequestPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub base_dir: Option<String>,
    pub graph_name: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct GetGraphConnectionsSingleResponseData {
    pub app: String,
    pub extension: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cmd: Option<Vec<DesignerMessageFlow>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Vec<DesignerMessageFlow>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_frame: Option<Vec<DesignerMessageFlow>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub video_frame: Option<Vec<DesignerMessageFlow>>,
}

impl From<GraphConnection> for GetGraphConnectionsSingleResponseData {
    fn from(conn: GraphConnection) -> Self {
        GetGraphConnectionsSingleResponseData {
            app: conn.get_app_uri().to_string(),
            extension: conn.extension,

            cmd: conn.cmd.map(get_designer_msg_flow_from_property),

            data: conn.data.map(get_designer_msg_flow_from_property),

            audio_frame: conn
                .audio_frame
                .map(get_designer_msg_flow_from_property),

            video_frame: conn
                .video_frame
                .map(get_designer_msg_flow_from_property),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct DesignerMessageFlow {
    pub name: String,
    pub dest: Vec<DesignerDestination>,
}

impl From<GraphMessageFlow> for DesignerMessageFlow {
    fn from(msg_flow: GraphMessageFlow) -> Self {
        DesignerMessageFlow {
            name: msg_flow.name,
            dest: get_designer_destination_from_property(msg_flow.dest),
        }
    }
}

fn get_designer_msg_flow_from_property(
    msg_flow: Vec<GraphMessageFlow>,
) -> Vec<DesignerMessageFlow> {
    if msg_flow.is_empty() {
        return vec![];
    }

    msg_flow.into_iter().map(|v| v.into()).collect()
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct DesignerDestination {
    pub app: String,
    pub extension: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub msg_conversion: Option<MsgAndResultConversion>,
}

impl From<GraphDestination> for DesignerDestination {
    fn from(destination: GraphDestination) -> Self {
        DesignerDestination {
            app: destination.get_app_uri().to_string(),
            extension: destination.extension,
            msg_conversion: destination.msg_conversion,
        }
    }
}

fn get_designer_destination_from_property(
    destinations: Vec<GraphDestination>,
) -> Vec<DesignerDestination> {
    destinations.into_iter().map(|v| v.into()).collect()
}

pub async fn get_graph_connections(
    request_payload: web::Json<GetGraphConnectionsRequestPayload>,
    state: web::Data<Arc<RwLock<DesignerState>>>,
) -> Result<impl Responder, actix_web::Error> {
    let state_read = state.read().unwrap();

    let base_dir = match get_base_dir_from_pkgs_cache(
        request_payload.base_dir.clone(),
        &state_read.pkgs_cache,
    ) {
        Ok(base_dir) => base_dir,
        Err(e) => {
            let error_response = ErrorResponse {
                status: Status::Fail,
                message: e.to_string(),
                error: None,
            };
            return Ok(HttpResponse::BadRequest().json(error_response));
        }
    };

    if let Some(pkgs) = &state_read.pkgs_cache.get(&base_dir) {
        if let Some(app_pkg) = pkgs
            .iter()
            .find(|pkg| pkg.basic_info.type_and_name.pkg_type == PkgType::App)
        {
            let graph_name = request_payload.graph_name.clone();

            // If the app package has predefined graphs, find the one with the
            // specified graph_name.
            if let Some(predefined_graph) = pkg_predefined_graphs_find(
                app_pkg.get_predefined_graphs(),
                |g| g.name == graph_name,
            ) {
                // Convert the connections field to RespConnection.
                let connections: Option<_> =
                    predefined_graph.graph.connections.as_ref();
                let resp_connections: Vec<
                    GetGraphConnectionsSingleResponseData,
                > = match connections {
                    Some(connections) => connections
                        .iter()
                        .map(|conn| conn.clone().into())
                        .collect(),
                    None => vec![],
                };

                let response = ApiResponse {
                    status: Status::Ok,
                    data: resp_connections,
                    meta: None,
                };

                return Ok(HttpResponse::Ok().json(response));
            }
        }
    }

    let error_response = ErrorResponse {
        status: Status::Fail,
        message: "Graph not found".to_string(),
        error: None,
    };
    Ok(HttpResponse::NotFound().json(error_response))
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::vec;

    use actix_web::{test, App};

    use super::*;
    use crate::{
        config::TmanConfig, constants::TEST_DIR,
        designer::mock::inject_all_pkgs_for_mock, output::TmanOutputCli,
    };
    use ten_rust::pkg_info::localhost;

    #[actix_web::test]
    async fn test_get_connections_success() {
        let mut designer_state = DesignerState {
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
            &mut designer_state.pkgs_cache,
            all_pkgs_json,
        );
        assert!(inject_ret.is_ok());

        let designer_state = Arc::new(RwLock::new(designer_state));

        let app = test::init_service(
            App::new().app_data(web::Data::new(designer_state)).route(
                "/api/designer/v1/graphs/connections",
                web::post().to(get_graph_connections),
            ),
        )
        .await;

        let request_payload = GetGraphConnectionsRequestPayload {
            base_dir: Some(TEST_DIR.to_string()),
            graph_name: "default".to_string(),
        };

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/graphs/connections")
            .set_json(request_payload)
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();

        let connections: ApiResponse<
            Vec<GetGraphConnectionsSingleResponseData>,
        > = serde_json::from_str(body_str).unwrap();

        let expected_connections =
            vec![GetGraphConnectionsSingleResponseData {
                app: localhost(),
                extension: "extension_1".to_string(),
                cmd: Some(vec![DesignerMessageFlow {
                    name: "hello_world".to_string(),
                    dest: vec![DesignerDestination {
                        app: localhost(),
                        extension: "extension_2".to_string(),
                        msg_conversion: None,
                    }],
                }]),
                data: None,
                audio_frame: None,
                video_frame: None,
            }];

        assert_eq!(connections.data, expected_connections);
        assert!(!connections.data.is_empty());

        let json: ApiResponse<Vec<GetGraphConnectionsSingleResponseData>> =
            serde_json::from_str(body_str).unwrap();
        let pretty_json = serde_json::to_string_pretty(&json).unwrap();
        println!("Response body: {}", pretty_json);
    }

    #[actix_web::test]
    async fn test_get_connections_have_all_data_type() {
        let mut designer_state = DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
        };

        // The first item is 'manifest.json', and the second item is
        // 'property.json'.
        let all_pkgs_json = vec![
            (
                include_str!("test_data_embed/get_connections_have_all_data_type/app_manifest.json")
                    .to_string(),
                include_str!("test_data_embed/get_connections_have_all_data_type/app_property.json")
                    .to_string(),
            ),
            (
                include_str!("test_data_embed/get_connections_have_all_data_type/extension_addon_1_manifest.json")
                    .to_string(),
                "{}".to_string(),
            ),
            (
                include_str!("test_data_embed/get_connections_have_all_data_type/extension_addon_2_manifest.json")
                    .to_string(),
                "{}".to_string(),
            ),
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
                "/api/designer/v1/graphs/connections",
                web::post().to(get_graph_connections),
            ),
        )
        .await;

        let request_payload = GetGraphConnectionsRequestPayload {
            base_dir: Some(TEST_DIR.to_string()),
            graph_name: "default".to_string(),
        };

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/graphs/connections")
            .set_json(request_payload)
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();

        let connections: ApiResponse<
            Vec<GetGraphConnectionsSingleResponseData>,
        > = serde_json::from_str(body_str).unwrap();

        let expected_connections =
            vec![GetGraphConnectionsSingleResponseData {
                app: localhost(),
                extension: "extension_1".to_string(),
                cmd: Some(vec![DesignerMessageFlow {
                    name: "hello_world".to_string(),
                    dest: vec![DesignerDestination {
                        app: localhost(),
                        extension: "extension_2".to_string(),
                        msg_conversion: None,
                    }],
                }]),
                data: Some(vec![DesignerMessageFlow {
                    name: "data".to_string(),
                    dest: vec![DesignerDestination {
                        app: localhost(),
                        extension: "extension_2".to_string(),
                        msg_conversion: None,
                    }],
                }]),
                audio_frame: Some(vec![DesignerMessageFlow {
                    name: "pcm".to_string(),
                    dest: vec![DesignerDestination {
                        app: localhost(),
                        extension: "extension_2".to_string(),
                        msg_conversion: None,
                    }],
                }]),
                video_frame: Some(vec![DesignerMessageFlow {
                    name: "image".to_string(),
                    dest: vec![DesignerDestination {
                        app: localhost(),
                        extension: "extension_2".to_string(),
                        msg_conversion: None,
                    }],
                }]),
            }];

        assert_eq!(connections.data, expected_connections);
        assert!(!connections.data.is_empty());

        let json: ApiResponse<Vec<GetGraphConnectionsSingleResponseData>> =
            serde_json::from_str(body_str).unwrap();
        let pretty_json = serde_json::to_string_pretty(&json).unwrap();
        println!("Response body: {}", pretty_json);
    }

    #[actix_web::test]
    async fn test_get_connections_without_base_dir_single_pkg() {
        let mut designer_state = DesignerState {
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
            &mut designer_state.pkgs_cache,
            all_pkgs_json,
        );
        assert!(inject_ret.is_ok());

        let designer_state = Arc::new(RwLock::new(designer_state));

        let app = test::init_service(
            App::new().app_data(web::Data::new(designer_state)).route(
                "/api/designer/v1/graphs/connections",
                web::post().to(get_graph_connections),
            ),
        )
        .await;

        // Create request payload without specifying base_dir
        let request_payload = GetGraphConnectionsRequestPayload {
            base_dir: None,
            graph_name: "default".to_string(),
        };

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/graphs/connections")
            .set_json(request_payload)
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();

        let connections: ApiResponse<
            Vec<GetGraphConnectionsSingleResponseData>,
        > = serde_json::from_str(body_str).unwrap();

        let expected_connections =
            vec![GetGraphConnectionsSingleResponseData {
                app: localhost(),
                extension: "extension_1".to_string(),
                cmd: Some(vec![DesignerMessageFlow {
                    name: "hello_world".to_string(),
                    dest: vec![DesignerDestination {
                        app: localhost(),
                        extension: "extension_2".to_string(),
                        msg_conversion: None,
                    }],
                }]),
                data: None,
                audio_frame: None,
                video_frame: None,
            }];

        assert_eq!(connections.data, expected_connections);
        assert!(!connections.data.is_empty());
    }

    #[actix_web::test]
    async fn test_get_connections_without_base_dir_multiple_pkgs() {
        let mut designer_state = DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
        };

        // Add first set of packages to TEST_DIR
        let all_pkgs_json = vec![(
            include_str!("test_data_embed/app_manifest.json").to_string(),
            include_str!("test_data_embed/app_property.json").to_string(),
        )];

        let inject_ret = inject_all_pkgs_for_mock(
            TEST_DIR,
            &mut designer_state.pkgs_cache,
            all_pkgs_json,
        );
        assert!(inject_ret.is_ok());

        // Add second set of packages to a different directory
        let second_dir = format!("{}/another_dir", TEST_DIR);
        let all_pkgs_json = vec![(
            include_str!("test_data_embed/app_manifest.json").to_string(),
            include_str!("test_data_embed/app_property.json").to_string(),
        )];

        let inject_ret = inject_all_pkgs_for_mock(
            &second_dir,
            &mut designer_state.pkgs_cache,
            all_pkgs_json,
        );
        assert!(inject_ret.is_ok());

        // Verify that we now have multiple entries in pkgs_cache
        assert_eq!(designer_state.pkgs_cache.len(), 2);

        let designer_state = Arc::new(RwLock::new(designer_state));

        let app = test::init_service(
            App::new().app_data(web::Data::new(designer_state)).route(
                "/api/designer/v1/graphs/connections",
                web::post().to(get_graph_connections),
            ),
        )
        .await;

        // Create request payload without specifying base_dir
        let request_payload = GetGraphConnectionsRequestPayload {
            base_dir: None,
            graph_name: "default".to_string(),
        };

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/graphs/connections")
            .set_json(request_payload)
            .to_request();
        let resp = test::call_service(&app, req).await;

        // Should return a bad request since base_dir is required when multiple
        // packages exist
        assert_eq!(resp.status(), actix_web::http::StatusCode::BAD_REQUEST);

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();
        let error_response: ErrorResponse =
            serde_json::from_str(body_str).unwrap();

        assert_eq!(error_response.status, Status::Fail);
        assert_eq!(
            error_response.message,
            "Multiple apps available. Please specify base_dir."
        );
    }
}
