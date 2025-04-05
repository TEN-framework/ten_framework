//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        sync::{Arc, RwLock},
    };

    use actix_web::{test, web, App};
    use ten_manager::{
        config::TmanConfig,
        constants::TEST_DIR,
        designer::{
            graphs::connections::{
                get::{
                    get_graph_connections_endpoint,
                    GetGraphConnectionsRequestPayload,
                    GraphConnectionsSingleResponseData,
                },
                DesignerDestination, DesignerMessageFlow,
            },
            mock::inject_all_pkgs_for_mock,
            response::ApiResponse,
            DesignerState,
        },
        output::TmanOutputCli,
    };

    #[actix_web::test]
    async fn test_get_connections_success() {
        let mut designer_state = DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
        };

        let all_pkgs_json = vec![
            (
                include_str!("../test_data_embed/app_manifest.json")
                    .to_string(),
                include_str!("../test_data_embed/app_property.json")
                    .to_string(),
            ),
            (
                include_str!(
                    "../test_data_embed/extension_addon_1_manifest.json"
                )
                .to_string(),
                "{}".to_string(),
            ),
            (
                include_str!(
                    "../test_data_embed/extension_addon_2_manifest.json"
                )
                .to_string(),
                "{}".to_string(),
            ),
            (
                include_str!(
                    "../test_data_embed/extension_addon_3_manifest.json"
                )
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
                web::post().to(get_graph_connections_endpoint),
            ),
        )
        .await;

        let request_payload = GetGraphConnectionsRequestPayload {
            base_dir: TEST_DIR.to_string(),
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

        let connections: ApiResponse<Vec<GraphConnectionsSingleResponseData>> =
            serde_json::from_str(body_str).unwrap();

        let expected_connections = vec![GraphConnectionsSingleResponseData {
            app: None,
            extension: "extension_1".to_string(),
            cmd: Some(vec![DesignerMessageFlow {
                name: "hello_world".to_string(),
                dest: vec![DesignerDestination {
                    app: None,
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

        let json: ApiResponse<Vec<GraphConnectionsSingleResponseData>> =
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
                web::post().to(get_graph_connections_endpoint),
            ),
        )
        .await;

        let request_payload = GetGraphConnectionsRequestPayload {
            base_dir: TEST_DIR.to_string(),
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

        let connections: ApiResponse<Vec<GraphConnectionsSingleResponseData>> =
            serde_json::from_str(body_str).unwrap();

        let expected_connections = vec![GraphConnectionsSingleResponseData {
            app: None,
            extension: "extension_1".to_string(),
            cmd: Some(vec![DesignerMessageFlow {
                name: "hello_world".to_string(),
                dest: vec![DesignerDestination {
                    app: None,
                    extension: "extension_2".to_string(),
                    msg_conversion: None,
                }],
            }]),
            data: Some(vec![DesignerMessageFlow {
                name: "data".to_string(),
                dest: vec![DesignerDestination {
                    app: None,
                    extension: "extension_2".to_string(),
                    msg_conversion: None,
                }],
            }]),
            audio_frame: Some(vec![DesignerMessageFlow {
                name: "pcm".to_string(),
                dest: vec![DesignerDestination {
                    app: None,
                    extension: "extension_2".to_string(),
                    msg_conversion: None,
                }],
            }]),
            video_frame: Some(vec![DesignerMessageFlow {
                name: "image".to_string(),
                dest: vec![DesignerDestination {
                    app: None,
                    extension: "extension_2".to_string(),
                    msg_conversion: None,
                }],
            }]),
        }];

        assert_eq!(connections.data, expected_connections);
        assert!(!connections.data.is_empty());

        let json: ApiResponse<Vec<GraphConnectionsSingleResponseData>> =
            serde_json::from_str(body_str).unwrap();
        let pretty_json = serde_json::to_string_pretty(&json).unwrap();
        println!("Response body: {}", pretty_json);
    }
}
