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
            graphs::get::{
                get_graphs_endpoint, GetGraphsRequestPayload,
                GetGraphsResponseData,
            },
            response::ApiResponse,
            DesignerState,
        },
        output::TmanOutputCli,
    };

    use crate::test_case::mock::inject_all_pkgs_for_mock;

    #[actix_web::test]
    async fn test_get_graphs_success() {
        let mut designer_state = DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
            graphs_cache: HashMap::new(),
        };

        let all_pkgs_json_str = vec![
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

        let request_payload = GetGraphsRequestPayload {};

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/graphs")
            .set_json(request_payload)
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();

        let graphs: ApiResponse<Vec<GetGraphsResponseData>> =
            serde_json::from_str(body_str).unwrap();

        let expected_graphs = vec![
            GetGraphsResponseData {
                uuid: "default".to_string(),
                name: "default".to_string(),
                auto_start: Some(true),
                base_dir: Some(TEST_DIR.to_string()),
            },
            GetGraphsResponseData {
                uuid: "default_with_app_uri".to_string(),
                name: "default_with_app_uri".to_string(),
                auto_start: Some(true),
                base_dir: Some(TEST_DIR.to_string()),
            },
            GetGraphsResponseData {
                uuid: "addon_not_found".to_string(),
                name: "addon_not_found".to_string(),
                auto_start: Some(false),
                base_dir: Some(TEST_DIR.to_string()),
            },
        ];

        assert_eq!(graphs.data.len(), expected_graphs.len());

        // Create a map of expected graphs by name for easier lookup.
        let expected_map: HashMap<_, _> = expected_graphs
            .iter()
            .map(|g| (g.name.clone(), g))
            .collect();

        for actual in graphs.data.iter() {
            let expected = expected_map
                .get(&actual.name)
                .expect("Missing expected graph");
            assert_eq!(actual.name, expected.name);
            assert_eq!(actual.auto_start, expected.auto_start);
            assert_eq!(actual.base_dir, expected.base_dir);
        }

        let json: ApiResponse<Vec<GetGraphsResponseData>> =
            serde_json::from_str(body_str).unwrap();
        let pretty_json = serde_json::to_string_pretty(&json).unwrap();
        println!("Response body: {}", pretty_json);
    }

    #[actix_web::test]
    async fn test_get_graphs_no_app_package() {
        let designer_state = Arc::new(RwLock::new(DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
            graphs_cache: HashMap::new(),
        }));

        let app = test::init_service(
            App::new().app_data(web::Data::new(designer_state)).route(
                "/api/designer/v1/graphs",
                web::post().to(get_graphs_endpoint),
            ),
        )
        .await;

        let request_payload = GetGraphsRequestPayload {};

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/graphs")
            .set_json(request_payload)
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());
        println!("Response body: {}", resp.status());

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();
        println!("Response body: {}", body_str);
    }
}
