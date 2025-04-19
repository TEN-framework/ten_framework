//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::{collections::HashMap, sync::Arc};

use actix_web::{http::StatusCode, test, web, App};

use ten_manager::{
    config::{metadata::TmanMetadata, TmanConfig},
    constants::{DEFAULT_APP_NODEJS, DEFAULT_EXTENSION_CPP},
    designer::{
        response::{ApiResponse, Status},
        template_pkgs::{
            get_template_endpoint, GetTemplateRequestPayload,
            GetTemplateResponseData, TemplateLanguage,
        },
        DesignerState,
    },
    output::TmanOutputCli,
};
use ten_rust::pkg_info::pkg_type::PkgType;

#[actix_web::test]
async fn test_get_template_app_typescript() {
    let designer_state = DesignerState {
        tman_config: Arc::new(tokio::sync::RwLock::new(TmanConfig::default())),
        tman_metadata: Arc::new(tokio::sync::RwLock::new(
            TmanMetadata::default(),
        )),
        out: Arc::new(Box::new(TmanOutputCli)),
        pkgs_cache: tokio::sync::RwLock::new(HashMap::new()),
        graphs_cache: tokio::sync::RwLock::new(HashMap::new()),
    };
    let designer_state = Arc::new(designer_state);

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(designer_state.clone()))
            .route(
                "/api/designer/v1/template-pkgs",
                web::post().to(get_template_endpoint),
            ),
    )
    .await;

    let request_payload = GetTemplateRequestPayload {
        pkg_type: PkgType::App,
        language: TemplateLanguage::TypeScript,
    };

    let req = test::TestRequest::post()
        .uri("/api/designer/v1/template-pkgs")
        .set_json(&request_payload)
        .to_request();

    let resp: ApiResponse<GetTemplateResponseData> =
        test::call_and_read_body_json(&app, req).await;

    assert_eq!(resp.status, Status::Ok);
    assert_eq!(
        resp.data.template_name,
        vec![DEFAULT_APP_NODEJS.to_string()]
    );
}

#[actix_web::test]
async fn test_get_template_extension_cpp() {
    let designer_state = DesignerState {
        tman_config: Arc::new(tokio::sync::RwLock::new(TmanConfig::default())),
        tman_metadata: Arc::new(tokio::sync::RwLock::new(
            TmanMetadata::default(),
        )),
        out: Arc::new(Box::new(TmanOutputCli)),
        pkgs_cache: tokio::sync::RwLock::new(HashMap::new()),
        graphs_cache: tokio::sync::RwLock::new(HashMap::new()),
    };
    let designer_state = Arc::new(designer_state);

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(designer_state.clone()))
            .route(
                "/api/designer/v1/template-pkgs",
                web::post().to(get_template_endpoint),
            ),
    )
    .await;

    let request_payload = GetTemplateRequestPayload {
        pkg_type: PkgType::Extension,
        language: TemplateLanguage::Cpp,
    };

    let req = test::TestRequest::post()
        .uri("/api/designer/v1/template-pkgs")
        .set_json(&request_payload)
        .to_request();

    let resp: ApiResponse<GetTemplateResponseData> =
        test::call_and_read_body_json(&app, req).await;

    assert_eq!(resp.status, Status::Ok);
    assert_eq!(
        resp.data.template_name,
        vec![DEFAULT_EXTENSION_CPP.to_string()]
    );
}

#[actix_web::test]
async fn test_get_template_unsupported() {
    let designer_state = DesignerState {
        tman_config: Arc::new(tokio::sync::RwLock::new(TmanConfig::default())),
        tman_metadata: Arc::new(tokio::sync::RwLock::new(
            TmanMetadata::default(),
        )),
        out: Arc::new(Box::new(TmanOutputCli)),
        pkgs_cache: tokio::sync::RwLock::new(HashMap::new()),
        graphs_cache: tokio::sync::RwLock::new(HashMap::new()),
    };
    let designer_state = Arc::new(designer_state);

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(designer_state.clone()))
            .route(
                "/api/designer/v1/template-pkgs",
                web::post().to(get_template_endpoint),
            ),
    )
    .await;

    // Create a request with an unsupported PkgType and Language
    // combination.
    let request_payload = GetTemplateRequestPayload {
        pkg_type: PkgType::Invalid,
        language: TemplateLanguage::TypeScript,
    };

    let req = test::TestRequest::post()
        .uri("/api/designer/v1/template-pkgs")
        .set_json(&request_payload)
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Expect a 400 Bad Request response.
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}
