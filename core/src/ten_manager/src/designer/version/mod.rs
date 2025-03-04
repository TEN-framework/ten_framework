//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::sync::{Arc, RwLock};

use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

use super::{
    response::{ApiResponse, Status},
    DesignerState,
};
use crate::{version::VERSION, version_utils::check_update};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct DesignerVersion {
    version: String,
}

pub async fn get_version(
    _state: web::Data<Arc<RwLock<DesignerState>>>,
) -> impl Responder {
    let version_info = DesignerVersion {
        version: VERSION.to_string(),
    };

    let response = ApiResponse {
        status: Status::Ok,
        data: version_info,
        meta: None,
    };

    HttpResponse::Ok().json(response)
}

pub async fn check_update_endpoint() -> impl Responder {
    match check_update().await {
        Ok((true, latest)) => {
            let success_response = serde_json::json!({
                "status": "ok",
                "update_available": true,
                "latest_version": latest,
            });
            HttpResponse::Ok().json(success_response)
        }
        Ok((false, _)) => {
            let success_response = serde_json::json!({
                "status": "ok",
                "update_available": false,
            });
            HttpResponse::Ok().json(success_response)
        }
        Err(err_msg) => {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": err_msg,
            });
            HttpResponse::Ok().json(error_response)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::config::TmanConfig;

    use super::*;
    use actix_web::{http::StatusCode, test, App};

    #[actix_web::test]
    async fn test_get_version() {
        // Initialize the DesignerState.
        let state = web::Data::new(Arc::new(RwLock::new(DesignerState {
            base_dir: None,
            all_pkgs: None,
            tman_config: TmanConfig::default(),
        })));

        // Create the App with the routes configured.
        let app = test::init_service(
            App::new()
                .app_data(state.clone())
                .route("/api/designer/v1/version", web::get().to(get_version)),
        )
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

        let version: ApiResponse<DesignerVersion> =
            serde_json::from_str(body_str).unwrap();

        // Create the expected Version struct
        let expected_version = DesignerVersion {
            version: VERSION.to_string(),
        };

        // Compare the actual Version struct with the expected one
        assert_eq!(version.data, expected_version);

        let json: ApiResponse<DesignerVersion> =
            serde_json::from_str(body_str).unwrap();
        let pretty_json = serde_json::to_string_pretty(&json).unwrap();
        println!("Response body: {}", pretty_json);
    }

    // Helper function that allows injecting a custom `check_update` function
    // for testing.
    pub async fn check_update_endpoint_with<F, Fut>(
        check_update_fn: F,
    ) -> impl Responder
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<(bool, String), String>>,
    {
        match check_update_fn().await {
            Ok((true, latest)) => {
                let success_response = serde_json::json!({
                    "status": "ok",
                    "update_available": true,
                    "latest_version": latest,
                });
                HttpResponse::Ok().json(success_response)
            }
            Ok((false, _)) => {
                let success_response = serde_json::json!({
                    "status": "ok",
                    "update_available": false,
                });
                HttpResponse::Ok().json(success_response)
            }
            Err(err_msg) => {
                let error_response = serde_json::json!({
                    "status": "fail",
                    "message": err_msg,
                });
                HttpResponse::Ok().json(error_response)
            }
        }
    }

    #[cfg(test)]
    mod check_update_tests {
        use super::*;
        use actix_web::{test, web, App};

        #[actix_web::test]
        async fn test_check_update_endpoint_update_available() {
            let dummy_check_update =
                || async { Ok((true, "v1.2.3".to_string())) };

            let app =
                test::init_service(App::new().route(
                    "/api/check_update",
                    web::get().to(move || {
                        check_update_endpoint_with(dummy_check_update)
                    }),
                ))
                .await;

            let req = test::TestRequest::get()
                .uri("/api/check_update")
                .to_request();
            let resp = test::call_service(&app, req).await;
            assert!(resp.status().is_success());

            let body = test::read_body(resp).await;
            let json_body: serde_json::Value =
                serde_json::from_slice(&body).unwrap();
            assert_eq!(json_body["status"], "ok");
            assert_eq!(json_body["update_available"], true);
            assert_eq!(json_body["latest_version"], "v1.2.3");
        }

        #[actix_web::test]
        async fn test_check_update_endpoint_no_update() {
            let dummy_check_update =
                || async { Ok((false, "v1.0.0".to_string())) };

            let app =
                test::init_service(App::new().route(
                    "/api/check_update",
                    web::get().to(move || {
                        check_update_endpoint_with(dummy_check_update)
                    }),
                ))
                .await;

            let req = test::TestRequest::get()
                .uri("/api/check_update")
                .to_request();
            let resp = test::call_service(&app, req).await;
            assert!(resp.status().is_success());

            let body = test::read_body(resp).await;
            let json_body: serde_json::Value =
                serde_json::from_slice(&body).unwrap();
            assert_eq!(json_body["status"], "ok");
            assert_eq!(json_body["update_available"], false);
            assert!(json_body.get("latest_version").is_none());
        }

        #[actix_web::test]
        async fn test_check_update_endpoint_error() {
            let dummy_check_update =
                || async { Err("Simulated update check failure".to_string()) };

            let app =
                test::init_service(App::new().route(
                    "/api/check_update",
                    web::get().to(move || {
                        check_update_endpoint_with(dummy_check_update)
                    }),
                ))
                .await;

            let req = test::TestRequest::get()
                .uri("/api/check_update")
                .to_request();
            let resp = test::call_service(&app, req).await;
            assert!(resp.status().is_success());

            let body = test::read_body(resp).await;
            let json_body: serde_json::Value =
                serde_json::from_slice(&body).unwrap();
            assert_eq!(json_body["status"], "fail");
            assert_eq!(json_body["message"], "Simulated update check failure");
        }
    }
}
