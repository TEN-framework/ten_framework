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

    use actix_web::{http::StatusCode, test, web, App};
    use serde::{Deserialize, Serialize};

    use ten_manager::config::internal::TmanInternalConfig;
    use ten_manager::{
        config::TmanConfig,
        designer::{
            response::ApiResponse, version::get_version_endpoint, DesignerState,
        },
        output::TmanOutputCli,
        version::VERSION,
    };

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct GetVersionResponseData {
        version: String,
    }

    #[actix_web::test]
    async fn test_get_version() {
        // Initialize the DesignerState.
        let state = web::Data::new(Arc::new(RwLock::new(DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            tman_internal_config: Arc::new(TmanInternalConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
            graphs_cache: HashMap::new(),
        })));

        // Create the App with the routes configured.
        let app = test::init_service(App::new().app_data(state.clone()).route(
            "/api/designer/v1/version",
            web::get().to(get_version_endpoint),
        ))
        .await;

        // Send a request to the version endpoint.
        let req = test::TestRequest::get()
            .uri("/api/designer/v1/version")
            .to_request();
        let resp = test::call_service(&app, req).await;

        // Check that the response status is 200 OK.
        assert_eq!(resp.status(), StatusCode::OK);

        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();

        let version: ApiResponse<GetVersionResponseData> =
            serde_json::from_str(body_str).unwrap();

        // Create the expected Version struct
        let expected_version = GetVersionResponseData {
            version: VERSION.to_string(),
        };

        // Compare the actual Version struct with the expected one
        assert_eq!(version.data, expected_version);

        let json: ApiResponse<GetVersionResponseData> =
            serde_json::from_str(body_str).unwrap();
        let pretty_json = serde_json::to_string_pretty(&json).unwrap();
        println!("Response body: {}", pretty_json);
    }
}
