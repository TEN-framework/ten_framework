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
        env, fs,
        sync::{Arc, RwLock},
    };

    use actix_web::{test, web, App};
    use ten_manager::{
        config::TmanConfig,
        constants::TEST_DIR,
        designer::{
            graphs::update::{
                update_graph_endpoint, GraphUpdateRequestPayload,
            },
            mock::inject_all_pkgs_for_mock,
            DesignerState,
        },
        output::TmanOutputCli,
    };

    #[actix_web::test]
    async fn test_update_graph_success() {
        // Get current working directory when testing.
        let exe_path =
            env::current_exe().expect("Failed to get the executable path");
        let test_dir = exe_path
            .parent()
            .expect("Failed to get the parent directory");

        let test_data_dir = test_dir.join("test_data");
        fs::create_dir_all(&test_data_dir)
            .expect("Failed to create test_data directory");

        let mut state = DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
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
            &mut state.pkgs_cache,
            all_pkgs_json_str,
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

        let base_dir_pkg_info =
            designer_state.pkgs_cache.get(TEST_DIR).unwrap();
        let app_pkg = base_dir_pkg_info.app_pkg_info.as_ref().unwrap();

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
