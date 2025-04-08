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
    use serde_json::json;
    use ten_manager::{
        config::TmanConfig,
        constants::TEST_DIR,
        designer::{
            graphs::nodes::{
                get::{
                    get_graph_nodes_endpoint, GetGraphNodesRequestPayload,
                    GraphNodesSingleResponseData,
                },
                DesignerApi, DesignerApiCmdLike, DesignerApiDataLike,
                DesignerPropertyAttributes,
            },
            mock::inject_all_pkgs_for_mock,
            response::{ApiResponse, ErrorResponse},
            DesignerState,
        },
        output::TmanOutputCli,
    };
    use ten_rust::pkg_info::value_type::ValueType;

    #[actix_web::test]
    async fn test_get_extensions_success() {
        let mut designer_state = DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
            graphs_cache: HashMap::new(),
        };

        let all_pkgs_json_str = vec![
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
            &mut designer_state.graphs_cache,
            all_pkgs_json_str,
        );
        assert!(inject_ret.is_ok());

        let designer_state = Arc::new(RwLock::new(designer_state));

        let app = test::init_service(
            App::new().app_data(web::Data::new(designer_state)).route(
                "/api/designer/v1/graphs/nodes",
                web::post().to(get_graph_nodes_endpoint),
            ),
        )
        .await;

        let request_payload = GetGraphNodesRequestPayload {
            base_dir: TEST_DIR.to_string(),
            graph_name: "default".to_string(),
        };

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/graphs/nodes")
            .set_json(request_payload)
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();

        let extensions: ApiResponse<Vec<GraphNodesSingleResponseData>> =
            serde_json::from_str(body_str).unwrap();

        // Create the expected Version struct
        let expected_extensions = vec![
            GraphNodesSingleResponseData {
                addon: "extension_addon_1".to_string(),
                name: "extension_1".to_string(),
                extension_group: Some("extension_group_1".to_string()),
                app: None,
                api: Some(DesignerApi {
                    property: None,
                    cmd_in: None,
                    cmd_out: Some(vec![
                        DesignerApiCmdLike {
                            name: "test_cmd".to_string(),
                            property: Some({
                                let mut map = HashMap::new();
                                map.insert(
                                    "test_property".to_string(),
                                    DesignerPropertyAttributes {
                                        prop_type: ValueType::Int8,
                                        items: None,
                                        properties: None,
                                        required: None,
                                    },
                                );
                                map
                            }),
                            required: None,
                            result: None,
                        },
                        DesignerApiCmdLike {
                            name: "has_required".to_string(),
                            property: Some({
                                let mut map = HashMap::new();
                                map.insert(
                                    "foo".to_string(),
                                    DesignerPropertyAttributes {
                                        prop_type: ValueType::String,
                                        items: None,
                                        properties: None,
                                        required: None,
                                    },
                                );
                                map
                            }),
                            required: Some(vec!["foo".to_string()]),
                            result: None,
                        },
                        DesignerApiCmdLike {
                            name: "has_required_mismatch".to_string(),
                            property: Some({
                                let mut map = HashMap::new();
                                map.insert(
                                    "foo".to_string(),
                                    DesignerPropertyAttributes {
                                        prop_type: ValueType::String,
                                        items: None,
                                        properties: None,
                                        required: None,
                                    },
                                );
                                map
                            }),
                            required: Some(vec!["foo".to_string()]),
                            result: None,
                        },
                    ]),
                    data_in: None,
                    data_out: None,
                    audio_frame_in: None,
                    audio_frame_out: None,
                    video_frame_in: None,
                    video_frame_out: None,
                }),
                property: None,
                is_installed: true,
            },
            GraphNodesSingleResponseData {
                addon: "extension_addon_2".to_string(),
                name: "extension_2".to_string(),
                extension_group: Some("extension_group_1".to_string()),
                app: None,
                api: Some(DesignerApi {
                    property: None,
                    cmd_in: Some(vec![
                        DesignerApiCmdLike {
                            name: "test_cmd".to_string(),
                            property: Some({
                                let mut map = HashMap::new();
                                map.insert(
                                    "test_property".to_string(),
                                    DesignerPropertyAttributes {
                                        prop_type: ValueType::Int32,
                                        items: None,
                                        properties: None,
                                        required: None,
                                    },
                                );
                                map
                            }),
                            required: None,
                            result: None,
                        },
                        DesignerApiCmdLike {
                            name: "another_test_cmd".to_string(),
                            property: Some({
                                let mut map = HashMap::new();
                                map.insert(
                                    "test_property".to_string(),
                                    DesignerPropertyAttributes {
                                        prop_type: ValueType::Int8,
                                        items: None,
                                        properties: None,
                                        required: None,
                                    },
                                );
                                map
                            }),
                            required: None,
                            result: None,
                        },
                        DesignerApiCmdLike {
                            name: "has_required".to_string(),
                            property: Some({
                                let mut map = HashMap::new();
                                map.insert(
                                    "foo".to_string(),
                                    DesignerPropertyAttributes {
                                        prop_type: ValueType::String,
                                        items: None,
                                        properties: None,
                                        required: None,
                                    },
                                );
                                map
                            }),
                            required: Some(vec!["foo".to_string()]),
                            result: None,
                        },
                        DesignerApiCmdLike {
                            name: "has_required_mismatch".to_string(),
                            property: Some({
                                let mut map = HashMap::new();
                                map.insert(
                                    "foo".to_string(),
                                    DesignerPropertyAttributes {
                                        prop_type: ValueType::String,
                                        items: None,
                                        properties: None,
                                        required: None,
                                    },
                                );
                                map
                            }),
                            required: None,
                            result: None,
                        },
                    ]),
                    cmd_out: None,
                    data_in: None,
                    data_out: Some(vec![DesignerApiDataLike {
                        name: "data_has_required".to_string(),
                        property: Some({
                            let mut map = HashMap::new();
                            map.insert(
                                "foo".to_string(),
                                DesignerPropertyAttributes {
                                    prop_type: ValueType::Int8,
                                    items: None,
                                    properties: None,
                                    required: None,
                                },
                            );
                            map
                        }),
                        required: Some(vec!["foo".to_string()]),
                    }]),
                    audio_frame_in: None,
                    audio_frame_out: None,
                    video_frame_in: None,
                    video_frame_out: None,
                }),
                property: Some(json!({
                    "a": 1
                })),
                is_installed: true,
            },
            GraphNodesSingleResponseData {
                addon: "extension_addon_3".to_string(),
                name: "extension_3".to_string(),
                extension_group: Some("extension_group_1".to_string()),
                app: None,
                api: Some(DesignerApi {
                    property: None,
                    cmd_in: Some(vec![DesignerApiCmdLike {
                        name: "test_cmd".to_string(),
                        property: Some({
                            let mut map = HashMap::new();
                            map.insert(
                                "test_property".to_string(),
                                DesignerPropertyAttributes {
                                    prop_type: ValueType::String,
                                    items: None,
                                    properties: None,
                                    required: None,
                                },
                            );
                            map
                        }),
                        required: None,
                        result: None,
                    }]),
                    cmd_out: None,
                    data_in: Some(vec![DesignerApiDataLike {
                        name: "data_has_required".to_string(),
                        property: Some({
                            let mut map = HashMap::new();
                            map.insert(
                                "foo".to_string(),
                                DesignerPropertyAttributes {
                                    prop_type: ValueType::Int8,
                                    items: None,
                                    properties: None,
                                    required: None,
                                },
                            );
                            map
                        }),
                        required: Some(vec!["foo".to_string()]),
                    }]),
                    data_out: None,
                    audio_frame_in: None,
                    audio_frame_out: None,
                    video_frame_in: None,
                    video_frame_out: None,
                }),
                property: None,
                is_installed: true,
            },
        ];

        assert_eq!(extensions.data, expected_extensions);
        assert!(!extensions.data.is_empty());

        let json: ApiResponse<Vec<GraphNodesSingleResponseData>> =
            serde_json::from_str(body_str).unwrap();
        let pretty_json = serde_json::to_string_pretty(&json).unwrap();
        println!("Response body: {}", pretty_json);
    }

    #[actix_web::test]
    async fn test_get_extensions_no_graph() {
        let designer_state = Arc::new(RwLock::new(DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
            graphs_cache: HashMap::new(),
        }));

        let app = test::init_service(
            App::new().app_data(web::Data::new(designer_state)).route(
                "/api/designer/v1/graphs/nodes",
                web::post().to(get_graph_nodes_endpoint),
            ),
        )
        .await;

        let request_payload = GetGraphNodesRequestPayload {
            base_dir: TEST_DIR.to_string(),
            graph_name: "no_existing_graph".to_string(),
        };

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/graphs/nodes")
            .set_json(request_payload)
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_client_error());

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();

        let error: ErrorResponse = serde_json::from_str(body_str).unwrap();

        assert!(error.message.contains("Package information is missing"));
    }

    #[actix_web::test]
    async fn test_get_extensions_has_wrong_addon() {
        let mut designer_state = DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
            graphs_cache: HashMap::new(),
        };

        let all_pkgs_json_str = vec![
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
            &mut designer_state.graphs_cache,
            all_pkgs_json_str,
        );
        assert!(inject_ret.is_ok());

        let designer_state = Arc::new(RwLock::new(designer_state));

        let app = test::init_service(
            App::new().app_data(web::Data::new(designer_state)).route(
                "/api/designer/v1/graphs/nodes",
                web::post().to(get_graph_nodes_endpoint),
            ),
        )
        .await;

        let request_payload = GetGraphNodesRequestPayload {
            base_dir: TEST_DIR.to_string(),
            graph_name: "addon_not_found".to_string(),
        };

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/graphs/nodes")
            .set_json(request_payload)
            .to_request();

        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();

        let json: ApiResponse<Vec<GraphNodesSingleResponseData>> =
            serde_json::from_str(body_str).unwrap();
        let pretty_json = serde_json::to_string_pretty(&json).unwrap();
        println!("Response body: {}", pretty_json);
    }
}
