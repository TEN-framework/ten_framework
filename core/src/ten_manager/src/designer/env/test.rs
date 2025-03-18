//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use actix_web::{test, web, App};

use super::*;
use crate::config::TmanConfig;
use crate::designer::DesignerState;
use crate::output::TmanOutputCli;

#[actix_web::test]
async fn test_get_env_success() {
    // Create test state.
    let state = Arc::new(RwLock::new(DesignerState {
        tman_config: Arc::new(TmanConfig::default()),
        out: Arc::new(Box::new(TmanOutputCli)),
        pkgs_cache: Default::default(),
    }));

    // Create test app.
    let app = test::init_service(
        App::new().app_data(web::Data::new(state)).service(
            web::scope("/api/designer/v1")
                .route("/env", web::get().to(get_env_endpoint)),
        ),
    )
    .await;

    // Create test request.
    let req = test::TestRequest::get()
        .uri("/api/designer/v1/env")
        .to_request();
    let resp = test::call_service(&app, req).await;

    // Assert response status.
    assert!(resp.status().is_success());

    // Parse response body.
    let body = test::read_body(resp).await;
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Assert response structure.
    assert_eq!(json["status"], "ok");
    assert!(json["data"].is_object());
    assert!(json["data"]["os"].is_string());
    assert!(json["data"]["arch"].is_string());
}

#[actix_web::test]
async fn test_get_env_error() {
    // Create test state
    let state = Arc::new(RwLock::new(DesignerState {
        tman_config: Arc::new(TmanConfig::default()),
        out: Arc::new(Box::new(TmanOutputCli)),
        pkgs_cache: Default::default(),
    }));

    // Create test app
    let app = test::init_service(
        App::new().app_data(web::Data::new(state)).service(
            web::scope("/api/designer/v1")
                .route("/env", web::get().to(get_env_endpoint)),
        ),
    )
    .await;

    // Create test request with invalid path
    let req = test::TestRequest::get()
        .uri("/api/designer/v1/invalid")
        .to_request();
    let resp = test::call_service(&app, req).await;

    // Assert response status is 404
    assert_eq!(resp.status(), 404);
}
