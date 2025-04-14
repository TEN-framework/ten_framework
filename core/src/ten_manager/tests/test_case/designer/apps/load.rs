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

    use ten_manager::config::internal::TmanInternalConfig;
    use ten_manager::config::TmanConfig;
    use ten_manager::designer::apps::load::{
        load_app_endpoint, LoadAppRequestPayload, LoadAppResponseData,
    };
    use ten_manager::designer::response::ApiResponse;
    use ten_manager::designer::DesignerState;
    use ten_manager::output::TmanOutputCli;

    #[actix_web::test]
    async fn test_load_app_fail() {
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
                .route(
                    "/test_load_app_fail",
                    web::post().to(load_app_endpoint),
                ),
        )
        .await;

        let new_base_dir = LoadAppRequestPayload {
            base_dir: "/not/a/correct/app/folder/path".to_string(),
        };

        let req = test::TestRequest::post()
            .uri("/test_load_app_fail")
            .set_json(&new_base_dir)
            .to_request();
        let resp: Result<
            ApiResponse<LoadAppResponseData>,
            std::boxed::Box<dyn std::error::Error>,
        > = test::try_call_and_read_body_json(&app, req).await;

        assert!(resp.is_err());
    }
}
