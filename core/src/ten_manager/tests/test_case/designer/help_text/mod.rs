//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use actix_web::{http::StatusCode, test, web};

use ten_manager::{
    config::{internal::TmanInternalConfig, TmanConfig},
    designer::{
        help_text::{
            get_help_text_endpoint, GetHelpTextRequestPayload,
            GetHelpTextResponseData, HelpTextKey,
        },
        locale::Locale,
        response::ApiResponse,
        DesignerState,
    },
    output::TmanOutputCli,
};

#[actix_web::test]
async fn test_get_help_text_success() {
    // Create test data with default locale.
    let payload = GetHelpTextRequestPayload {
        key: HelpTextKey::TenAgent,
        locale: Locale::EnUs,
    };

    // Get mock state.
    let designer_state = DesignerState {
        tman_config: Arc::new(TmanConfig::default()),
        tman_internal_config: Arc::new(TmanInternalConfig::default()),
        out: Arc::new(Box::new(TmanOutputCli)),
        pkgs_cache: HashMap::new(),
        graphs_cache: HashMap::new(),
    };
    let state = web::Data::new(Arc::new(RwLock::new(designer_state)));

    // Create app with the endpoint.
    let app = test::init_service(
        actix_web::App::new()
            .app_data(state.clone())
            .route("/help", web::post().to(get_help_text_endpoint)),
    )
    .await;

    // Make the request.
    let req = test::TestRequest::post()
        .uri("/help")
        .set_json(&payload)
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert status code.
    assert_eq!(resp.status(), StatusCode::OK);

    // Extract and check the response body.
    let body = test::read_body(resp).await;
    let result: ApiResponse<GetHelpTextResponseData> =
        serde_json::from_slice(&body).unwrap();

    // Assert data.
    assert_eq!(result.data.key, HelpTextKey::TenAgent);
    assert_eq!(result.data.locale, Locale::EnUs);
    assert!(result.data.text.contains("TEN Agent is"));
}

#[actix_web::test]
async fn test_get_help_text_chinese() {
    // Create test data with Chinese locale.
    let payload = GetHelpTextRequestPayload {
        key: HelpTextKey::TenAgent,
        locale: Locale::ZhCn,
    };

    // Get mock state.
    let designer_state = DesignerState {
        tman_config: Arc::new(TmanConfig::default()),
        tman_internal_config: Arc::new(TmanInternalConfig::default()),
        out: Arc::new(Box::new(TmanOutputCli)),
        pkgs_cache: HashMap::new(),
        graphs_cache: HashMap::new(),
    };
    let state = web::Data::new(Arc::new(RwLock::new(designer_state)));

    // Create app with the endpoint.
    let app = test::init_service(
        actix_web::App::new()
            .app_data(state.clone())
            .route("/help", web::post().to(get_help_text_endpoint)),
    )
    .await;

    // Make the request.
    let req = test::TestRequest::post()
        .uri("/help")
        .set_json(&payload)
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert status code.
    assert_eq!(resp.status(), StatusCode::OK);

    // Extract and check the response body.
    let body = test::read_body(resp).await;
    let result: ApiResponse<GetHelpTextResponseData> =
        serde_json::from_slice(&body).unwrap();

    // Assert data.
    assert_eq!(result.data.key, HelpTextKey::TenAgent);
    assert_eq!(result.data.locale, Locale::ZhCn);
    // Chinese text should be different from English.
    assert!(result.data.text.contains("TEN Agent 是"));
}

#[actix_web::test]
async fn test_get_help_text_language_fallback() {
    // Use a non-existent locale.
    let payload = GetHelpTextRequestPayload {
        key: HelpTextKey::TenFramework,
        locale: Locale::JaJp, /* Use a locale that definitely doesn't have
                               * content */
    };

    // Get mock state.
    let designer_state = DesignerState {
        tman_config: Arc::new(TmanConfig::default()),
        tman_internal_config: Arc::new(TmanInternalConfig::default()),
        out: Arc::new(Box::new(TmanOutputCli)),
        pkgs_cache: HashMap::new(),
        graphs_cache: HashMap::new(),
    };
    let state = web::Data::new(Arc::new(RwLock::new(designer_state)));

    // Create app with the endpoint.
    let app = test::init_service(
        actix_web::App::new()
            .app_data(state.clone())
            .route("/help", web::post().to(get_help_text_endpoint)),
    )
    .await;

    // Make the request.
    let req = test::TestRequest::post()
        .uri("/help")
        .set_json(&payload)
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert status code.
    assert_eq!(resp.status(), StatusCode::OK);

    // Extract and check the response body.
    let body = test::read_body(resp).await;
    let result: ApiResponse<GetHelpTextResponseData> =
        serde_json::from_slice(&body).unwrap();

    // Assert data - should fall back to another locale.
    assert_eq!(result.data.key, HelpTextKey::TenFramework);
    // The locale in the response should be the one that was actually used
    assert!(result.data.locale != Locale::JaJp);
}

#[actix_web::test]
async fn test_get_help_text_not_found() {
    // Use an unknown key.
    let payload = GetHelpTextRequestPayload {
        key: HelpTextKey::Unknown,
        locale: Locale::EnUs,
    };

    // Get mock state.
    let designer_state = DesignerState {
        tman_config: Arc::new(TmanConfig::default()),
        tman_internal_config: Arc::new(TmanInternalConfig::default()),
        out: Arc::new(Box::new(TmanOutputCli)),
        pkgs_cache: HashMap::new(),
        graphs_cache: HashMap::new(),
    };
    let state = web::Data::new(Arc::new(RwLock::new(designer_state)));

    // Create app with the endpoint.
    let app = test::init_service(
        actix_web::App::new()
            .app_data(state.clone())
            .route("/help", web::post().to(get_help_text_endpoint)),
    )
    .await;

    // Make the request.
    let req = test::TestRequest::post()
        .uri("/help")
        .set_json(&payload)
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert status code.
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[actix_web::test]
async fn test_default_locale() {
    // Test the default_locale function indirectly by deserializing JSON without
    // a locale field.
    let json = r#"{"key": "ten_agent"}"#;
    let request: GetHelpTextRequestPayload =
        serde_json::from_str(json).unwrap();

    assert_eq!(request.locale, Locale::EnUs);
}
