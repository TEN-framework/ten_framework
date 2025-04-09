//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use actix_web::{http::StatusCode, test, web, App};

use ten_manager::{
    config::TmanConfig,
    designer::{
        graphs::{
            connections::get::{
                get_graph_connections_endpoint,
                GetGraphConnectionsRequestPayload,
                GraphConnectionsSingleResponseData,
            },
            get::{
                get_graphs_endpoint, GetGraphsRequestPayload,
                GetGraphsResponseData,
            },
        },
        mock::inject_all_pkgs_for_mock,
        response::ApiResponse,
        DesignerState,
    },
    output::TmanOutputCli,
};

#[actix_rt::test]
async fn test_cmd_designer_graphs_app_property_not_exist() {
    let mut designer_state = DesignerState {
        tman_config: Arc::new(TmanConfig::default()),
        out: Arc::new(Box::new(TmanOutputCli)),
        pkgs_cache: HashMap::new(),
        graphs_cache: HashMap::new(),
    };

    let all_pkgs_json_str = vec![
      (
          include_str!("../../test_data/cmd_designer_graphs_app_property_not_exist/manifest.json").to_string(),
          "{}".to_string(),
      ),
      (
          include_str!("../../test_data/cmd_designer_graphs_app_property_not_exist/ten_packages/extension/addon_a/manifest.json")
              .to_string(),
          "{}".to_string(),
      ),
      (
          include_str!("../../test_data/cmd_designer_graphs_app_property_not_exist/ten_packages/extension/addon_b/manifest.json")
              .to_string(),
          "{}".to_string(),
      ),
    ];

    let inject_ret = inject_all_pkgs_for_mock(
        "tests/test_data/cmd_designer_graphs_app_property_not_exist",
        &mut designer_state.pkgs_cache,
        &mut designer_state.graphs_cache,
        all_pkgs_json_str,
    );
    assert!(inject_ret.is_ok());

    let designer_state = Arc::new(RwLock::new(designer_state));
    let app = test::init_service(
        App::new().app_data(web::Data::new(designer_state)).route(
            "/api/designer/v1/graphs",
            web::post().to(get_graphs_endpoint),
        ),
    )
    .await;

    let request_payload = GetGraphsRequestPayload {
        base_dir: "tests/test_data/cmd_designer_graphs_app_property_not_exist"
            .to_string(),
    };

    let req = test::TestRequest::post()
        .uri("/api/designer/v1/graphs")
        .set_json(&request_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body = test::read_body(resp).await;
    let body_str = std::str::from_utf8(&body).unwrap();
    let json: ApiResponse<Vec<GetGraphsResponseData>> =
        serde_json::from_str(body_str).unwrap();

    let pretty_json = serde_json::to_string_pretty(&json).unwrap();
    println!("Response body: {}", pretty_json);

    assert!(json.data.is_empty());
}

#[actix_rt::test]
async fn test_cmd_designer_connections_has_msg_conversion() {
    let mut designer_state = DesignerState {
        tman_config: Arc::new(TmanConfig::default()),
        out: Arc::new(Box::new(TmanOutputCli)),
        pkgs_cache: HashMap::new(),
        graphs_cache: HashMap::new(),
    };

    let all_pkgs_json_str = vec![
        (
            include_str!("../../test_data/cmd_designer_connections_has_msg_conversion/manifest.json").to_string(),
            include_str!("../../test_data/cmd_designer_connections_has_msg_conversion/property.json").to_string(),
        ),
        (
            include_str!("../../test_data/cmd_designer_connections_has_msg_conversion/ten_packages/extension/addon_a/manifest.json")
                .to_string(),
            "{}".to_string(),
        ),
        (
            include_str!("../../test_data/cmd_designer_connections_has_msg_conversion/ten_packages/extension/addon_b/manifest.json")
                .to_string(),
            "{}".to_string(),
        ),
    ];

    let inject_ret = inject_all_pkgs_for_mock(
        "tests/test_data/cmd_designer_connections_has_msg_conversion",
        &mut designer_state.pkgs_cache,
        &mut designer_state.graphs_cache,
        all_pkgs_json_str,
    );
    assert!(inject_ret.is_ok());

    let designer_state = Arc::new(RwLock::new(designer_state));
    let app = test::init_service(
        App::new().app_data(web::Data::new(designer_state)).route(
            "/api/designer/v1/graphs/connections",
            web::post().to(get_graph_connections_endpoint),
        ),
    )
    .await;

    let request_payload = GetGraphConnectionsRequestPayload {
        base_dir: "tests/test_data/cmd_designer_connections_has_msg_conversion"
            .to_string(),
        graph_name: "default".to_string(),
    };

    let req = test::TestRequest::post()
        .uri("/api/designer/v1/graphs/connections")
        .set_json(&request_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body = test::read_body(resp).await;
    let body_str = std::str::from_utf8(&body).unwrap();
    let json: ApiResponse<Vec<GraphConnectionsSingleResponseData>> =
        serde_json::from_str(body_str).unwrap();

    let pretty_json = serde_json::to_string_pretty(&json).unwrap();
    println!("Response body: {}", pretty_json);

    let connections = &json.data;
    assert_eq!(connections.len(), 1);

    let connection = connections.first().unwrap();
    assert!(connection.cmd.is_some());

    let cmd = connection.cmd.as_ref().unwrap();
    assert_eq!(cmd.len(), 1);
}
