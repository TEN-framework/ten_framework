//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use actix_web::{test, web, App};
use serde_json::json;

use ten_manager::{
    config::TmanConfig,
    constants::TEST_DIR,
    designer::{
        messages::compatible::{
            get_compatible_messages_endpoint,
            GetCompatibleMsgsSingleResponseData,
        },
        mock::inject_all_pkgs_for_mock,
        response::ApiResponse,
        DesignerState,
    },
    output::TmanOutputCli,
};
use ten_rust::pkg_info::message::{MsgDirection, MsgType};

#[actix_web::test]
async fn test_get_compatible_messages_success() {
    let mut designer_state = DesignerState {
        tman_config: Arc::new(TmanConfig::default()),
        out: Arc::new(Box::new(TmanOutputCli)),
        pkgs_cache: HashMap::new(),
    };

    let all_pkgs_json = vec![
        (
            include_str!("../../../../src/designer/messages/test_data_embed/app_manifest.json").to_string(),
            include_str!("../../../../src/designer/messages/test_data_embed/app_property.json").to_string(),
        ),
        (
            include_str!("../../../../src/designer/messages/test_data_embed/extension_addon_1_manifest.json")
                .to_string(),
            "{}".to_string(),
        ),
        (
            include_str!("../../../../src/designer/messages/test_data_embed/extension_addon_2_manifest.json")
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
            "/api/designer/v1/messages/compatible",
            web::post().to(get_compatible_messages_endpoint),
        ),
    )
    .await;

    // Define input data.
    let input_data = json!({
      "base_dir": TEST_DIR,
      "graph": "default",
      "extension_group": "extension_group_1",
      "extension": "extension_1",
      "msg_type": "cmd",
      "msg_direction": "out",
      "msg_name": "test_cmd"
    });

    // Send request to the test server.
    let req = test::TestRequest::post()
        .uri("/api/designer/v1/messages/compatible")
        .set_json(&input_data)
        .to_request();

    // Call the service and get the response.
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body = test::read_body(resp).await;
    let body_str = std::str::from_utf8(&body).unwrap();

    let compatibles: ApiResponse<Vec<GetCompatibleMsgsSingleResponseData>> =
        serde_json::from_str(body_str).unwrap();

    let expected_compatibles = vec![GetCompatibleMsgsSingleResponseData {
        app: None,
        extension_group: Some("extension_group_1".to_string()),
        extension: "extension_2".to_string(),
        msg_type: MsgType::Cmd,
        msg_direction: MsgDirection::In,
        msg_name: "test_cmd".to_string(),
    }];

    assert_eq!(compatibles.data, expected_compatibles);
}

#[actix_web::test]
async fn test_get_compatible_messages_fail() {
    let mut designer_state = DesignerState {
        tman_config: Arc::new(TmanConfig::default()),
        out: Arc::new(Box::new(TmanOutputCli)),
        pkgs_cache: HashMap::new(),
    };

    let all_pkgs_json = vec![
        (
            include_str!("../../../../src/designer/messages/test_data_embed/app_manifest.json").to_string(),
            include_str!("../../../../src/designer/messages/test_data_embed/app_property.json").to_string(),
        ),
        (
            include_str!("../../../../src/designer/messages/test_data_embed/extension_addon_1_manifest.json")
                .to_string(),
            "{}".to_string(),
        ),
        (
            include_str!("../../../../src/designer/messages/test_data_embed/extension_addon_2_manifest.json")
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
            "/api/designer/v1/messages/compatible",
            web::post().to(get_compatible_messages_endpoint),
        ),
    )
    .await;

    // Define input data.
    let input_data = json!({
      "base_dir": TEST_DIR,
      "graph": "default",
      "extension_group": "default_extension_group",
      "extension": "default_extension_cpp",
      "msg_type": "data",
      "msg_direction": "in",
      "msg_name": "not_existing_cmd"
    });

    // Send request to the test server.
    let req = test::TestRequest::post()
        .uri("/api/designer/v1/messages/compatible")
        .set_json(&input_data)
        .to_request();

    // Call the service and get the response
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_client_error());
}

#[actix_web::test]
async fn test_get_compatible_messages_cmd_has_required_success() {
    let mut designer_state = DesignerState {
        tman_config: Arc::new(TmanConfig::default()),
        out: Arc::new(Box::new(TmanOutputCli)),
        pkgs_cache: HashMap::new(),
    };

    let all_pkgs_json = vec![
        (
            include_str!("../../../../src/designer/messages/test_data_embed/app_manifest.json").to_string(),
            include_str!("../../../../src/designer/messages/test_data_embed/app_property.json").to_string(),
        ),
        (
            include_str!("../../../../src/designer/messages/test_data_embed/extension_addon_1_manifest.json")
                .to_string(),
            "{}".to_string(),
        ),
        (
            include_str!("../../../../src/designer/messages/test_data_embed/extension_addon_2_manifest.json")
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
            "/api/designer/v1/messages/compatible",
            web::post().to(get_compatible_messages_endpoint),
        ),
    )
    .await;

    // Define input data. This time we check cmd msg with required_fields.
    let input_data = json!({
      "base_dir": TEST_DIR,
      "graph": "default",
      "extension_group": "extension_group_1",
      "extension": "extension_1",
      "msg_type": "cmd",
      "msg_direction": "out",
      "msg_name": "cmd_with_required_fields"
    });

    // Send request to the test server.
    let req = test::TestRequest::post()
        .uri("/api/designer/v1/messages/compatible")
        .set_json(&input_data)
        .to_request();

    // Call the service and get the response
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body = test::read_body(resp).await;
    let body_str = std::str::from_utf8(&body).unwrap();

    let compatibles: ApiResponse<Vec<GetCompatibleMsgsSingleResponseData>> =
        serde_json::from_str(body_str).unwrap();

    // Should have 2 compatible messages
    assert_eq!(compatibles.data.len(), 2);

    // Just check the first compatible message matches expected
    let expected_compatible = GetCompatibleMsgsSingleResponseData {
        app: None,
        extension_group: Some("extension_group_1".to_string()),
        extension: "extension_2".to_string(),
        msg_type: MsgType::Cmd,
        msg_direction: MsgDirection::In,
        msg_name: "cmd_with_required_fields".to_string(),
    };

    // We're just checking that the first compatible message is in the results
    assert!(compatibles.data.contains(&expected_compatible));
}
