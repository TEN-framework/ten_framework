//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::{Arc, RwLock};

    use actix_web::{test, web, App};
    use ten_rust::base_dir_pkg_info::PkgsInfoInApp;

    use ten_manager::config::TmanConfig;
    use ten_manager::constants::TEST_DIR;
    use ten_manager::designer::apps::get::{
        get_apps_endpoint, AppInfo, GetAppsResponseData,
    };
    use ten_manager::designer::response::{ApiResponse, Status};
    use ten_manager::designer::DesignerState;
    use ten_manager::output::TmanOutputCli;

    #[actix_web::test]
    async fn test_get_apps_some() {
        let designer_state = DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            tman_internal_config: Arc::new(TmanInternalConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
            graphs_cache: HashMap::new(),
        };

        let designer_state = Arc::new(RwLock::new(designer_state));

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(designer_state.clone()))
                .route("/test_get_apps_some", web::get().to(get_apps_endpoint)),
        )
        .await;

        // Create an empty PkgsInfoInApp.
        let empty_pkg_info = PkgsInfoInApp::default();

        designer_state
            .write()
            .unwrap()
            .pkgs_cache
            .insert(TEST_DIR.to_string(), empty_pkg_info);

        let req = test::TestRequest::get()
            .uri("/test_get_apps_some")
            .to_request();

        let resp: ApiResponse<GetAppsResponseData> =
            test::call_and_read_body_json(&app, req).await;

        assert_eq!(resp.status, Status::Ok);
        assert_eq!(
            resp.data,
            GetAppsResponseData {
                app_info: vec![AppInfo {
                    base_dir: TEST_DIR.to_string(),
                    app_uri: None,
                }]
            }
        );
    }

    #[actix_web::test]
    async fn test_get_apps_none() {
        let designer_state = DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            tman_internal_config: Arc::new(TmanInternalConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
            graphs_cache: HashMap::new(),
        };
        let designer_state = Arc::new(RwLock::new(designer_state));

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(designer_state.clone()))
                .route("/test_get_apps_none", web::get().to(get_apps_endpoint)),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/test_get_apps_none")
            .to_request();

        let resp: ApiResponse<GetAppsResponseData> =
            test::call_and_read_body_json(&app, req).await;

        assert_eq!(resp.status, Status::Ok);
        assert_eq!(resp.data, GetAppsResponseData { app_info: vec![] });
    }
}
