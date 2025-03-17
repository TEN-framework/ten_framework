//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::sync::{Arc, RwLock};

use actix_web::{web, HttpResponse, Responder};
use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::designer::{
    response::{ApiResponse, ErrorResponse, Status},
    DesignerState,
};

use super::get_base_dir_from_pkgs_cache;

#[derive(Deserialize, Serialize, Debug)]
pub struct UnloadAppRequestPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub base_dir: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UnloadAppResponseData {
    pub success: bool,
}

pub async fn unload_app_endpoint(
    request_payload: web::Json<UnloadAppRequestPayload>,
    state: web::Data<Arc<RwLock<DesignerState>>>,
) -> Result<impl Responder, actix_web::Error> {
    let mut state_write = state.write().unwrap();

    let base_dir = match get_base_dir_from_pkgs_cache(
        request_payload.base_dir.clone(),
        &state_write.pkgs_cache,
    ) {
        Ok(base_dir) => base_dir,
        Err(e) => {
            let error_response = ErrorResponse {
                status: Status::Fail,
                message: e.to_string(),
                error: None,
            };
            return Ok(HttpResponse::BadRequest().json(error_response));
        }
    };

    // Check if the base_dir exists in pkgs_cache.
    if !state_write.pkgs_cache.contains_key(&base_dir) {
        let error_response = ErrorResponse {
            status: Status::Fail,
            message: format!("App with base_dir '{}' is not loaded", base_dir),
            error: None,
        };
        return Ok(HttpResponse::BadRequest().json(error_response));
    }

    state_write.pkgs_cache.remove(&base_dir);

    let response = ApiResponse {
        status: Status::Ok,
        data: serde_json::json!({ "success": true }),
        meta: None,
    };

    Ok(HttpResponse::Ok().json(response))
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use actix_web::{test, App};

    use super::*;
    use crate::{
        config::TmanConfig, constants::TEST_DIR,
        designer::mock::inject_all_pkgs_for_mock, output::TmanOutputCli,
    };

    #[actix_web::test]
    async fn test_unload_app_success() {
        // Create designer state with an app in pkgs_cache.
        let mut designer_state = DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
        };

        // Add a simple package to the pkgs_cache.
        let all_pkgs_json = vec![(
            include_str!("test_data_embed/app_manifest.json").to_string(),
            include_str!("test_data_embed/app_property.json").to_string(),
        )];

        let inject_ret = inject_all_pkgs_for_mock(
            TEST_DIR,
            &mut designer_state.pkgs_cache,
            all_pkgs_json,
        );
        assert!(inject_ret.is_ok());

        // Verify that the app is in the pkgs_cache.
        assert!(designer_state.pkgs_cache.contains_key(TEST_DIR));

        let designer_state = Arc::new(RwLock::new(designer_state));

        // Initialize test service.
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(designer_state.clone()))
                .route(
                    "/api/designer/v1/apps/unload",
                    web::post().to(unload_app_endpoint),
                ),
        )
        .await;

        // Create request payload.
        let request_payload = UnloadAppRequestPayload {
            base_dir: Some(TEST_DIR.to_string()),
        };

        // Send request to the test server.
        let req = test::TestRequest::post()
            .uri("/api/designer/v1/apps/unload")
            .set_json(request_payload)
            .to_request();
        let resp = test::call_service(&app, req).await;

        // Verify the response.
        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();
        let response: ApiResponse<serde_json::Value> =
            serde_json::from_str(body_str).unwrap();

        assert_eq!(response.status, Status::Ok);
        assert_eq!(response.data["success"], true);

        // Verify that the app has been removed from pkgs_cache.
        let state = designer_state.read().unwrap();
        assert!(!state.pkgs_cache.contains_key(TEST_DIR));
    }

    #[actix_web::test]
    async fn test_unload_app_invalid_base_dir() {
        // Create designer state without any apps in pkgs_cache.
        let designer_state = DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
        };

        let designer_state = Arc::new(RwLock::new(designer_state));

        // Initialize test service.
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(designer_state.clone()))
                .route(
                    "/api/designer/v1/apps/unload",
                    web::post().to(unload_app_endpoint),
                ),
        )
        .await;

        // Create request payload with a non-existent base_dir.
        let request_payload = UnloadAppRequestPayload {
            base_dir: Some("non_existent_dir".to_string()),
        };

        // Send request to the test server.
        let req = test::TestRequest::post()
            .uri("/api/designer/v1/apps/unload")
            .set_json(request_payload)
            .to_request();
        let resp = test::call_service(&app, req).await;

        // Verify the response is an error.
        assert_eq!(resp.status(), actix_web::http::StatusCode::BAD_REQUEST);

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();
        let error_response: ErrorResponse =
            serde_json::from_str(body_str).unwrap();

        assert_eq!(error_response.status, Status::Fail);
        assert!(!error_response.message.is_empty());
    }

    #[actix_web::test]
    async fn test_unload_app_multiple_apps_no_base_dir() {
        // Create designer state with multiple apps in pkgs_cache.
        let mut designer_state = DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
        };

        // Add first app to the pkgs_cache.
        let all_pkgs_json = vec![(
            include_str!("test_data_embed/app_manifest.json").to_string(),
            include_str!("test_data_embed/app_property.json").to_string(),
        )];

        let inject_ret = inject_all_pkgs_for_mock(
            TEST_DIR,
            &mut designer_state.pkgs_cache,
            all_pkgs_json,
        );
        assert!(inject_ret.is_ok());

        // Add second app to the pkgs_cache with a different base_dir.
        let second_dir = format!("{}/another_dir", TEST_DIR);
        let all_pkgs_json = vec![(
            include_str!("test_data_embed/app_manifest.json").to_string(),
            include_str!("test_data_embed/app_property.json").to_string(),
        )];

        let inject_ret = inject_all_pkgs_for_mock(
            &second_dir,
            &mut designer_state.pkgs_cache,
            all_pkgs_json,
        );
        assert!(inject_ret.is_ok());

        // Verify that we have multiple apps in pkgs_cache.
        assert_eq!(designer_state.pkgs_cache.len(), 2);

        let designer_state = Arc::new(RwLock::new(designer_state));

        // Initialize test service.
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(designer_state.clone()))
                .route(
                    "/api/designer/v1/apps/unload",
                    web::post().to(unload_app_endpoint),
                ),
        )
        .await;

        // Create request payload without specifying base_dir.
        let request_payload = UnloadAppRequestPayload { base_dir: None };

        // Send request to the test server.
        let req = test::TestRequest::post()
            .uri("/api/designer/v1/apps/unload")
            .set_json(request_payload)
            .to_request();
        let resp = test::call_service(&app, req).await;

        // Verify the response is an error since we have multiple apps but
        // didn't specify which to unload.
        assert!(resp.status().is_client_error());

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();
        let error_response: ErrorResponse =
            serde_json::from_str(body_str).unwrap();

        assert_eq!(error_response.status, Status::Fail);
        assert!(error_response.message.contains("Multiple apps available"));

        // Verify that both apps are still in pkgs_cache (nothing was unloaded).
        let state = designer_state.read().unwrap();
        assert_eq!(state.pkgs_cache.len(), 2);
    }

    #[actix_web::test]
    async fn test_unload_app_single_app_no_base_dir() {
        // Create designer state with a single app in pkgs_cache.
        let mut designer_state = DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
        };

        // Add a single package to the pkgs_cache.
        let all_pkgs_json = vec![(
            include_str!("test_data_embed/app_manifest.json").to_string(),
            include_str!("test_data_embed/app_property.json").to_string(),
        )];

        let inject_ret = inject_all_pkgs_for_mock(
            TEST_DIR,
            &mut designer_state.pkgs_cache,
            all_pkgs_json,
        );
        assert!(inject_ret.is_ok());

        // Verify that we have exactly one app in pkgs_cache.
        assert_eq!(designer_state.pkgs_cache.len(), 1);

        let designer_state = Arc::new(RwLock::new(designer_state));

        // Initialize test service.
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(designer_state.clone()))
                .route(
                    "/api/designer/v1/apps/unload",
                    web::post().to(unload_app_endpoint),
                ),
        )
        .await;

        // Create request payload without specifying base_dir.
        let request_payload = UnloadAppRequestPayload { base_dir: None };

        // Send request to the test server.
        let req = test::TestRequest::post()
            .uri("/api/designer/v1/apps/unload")
            .set_json(request_payload)
            .to_request();
        let resp = test::call_service(&app, req).await;

        // Verify the response is successful.
        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();
        let response: ApiResponse<serde_json::Value> =
            serde_json::from_str(body_str).unwrap();

        assert_eq!(response.status, Status::Ok);
        assert_eq!(response.data["success"], true);

        // Verify that the app has been removed from pkgs_cache.
        let state = designer_state.read().unwrap();
        assert_eq!(state.pkgs_cache.len(), 0);
    }

    #[actix_web::test]
    async fn test_unload_app_not_loaded() {
        // Create designer state without any apps in pkgs_cache.
        let designer_state = DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
        };

        let designer_state = Arc::new(RwLock::new(designer_state));

        // Initialize test service.
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(designer_state.clone()))
                .route(
                    "/api/designer/v1/apps/unload",
                    web::post().to(unload_app_endpoint),
                ),
        )
        .await;

        // Create request payload with a valid but not loaded base_dir.
        let request_payload = UnloadAppRequestPayload {
            base_dir: Some(TEST_DIR.to_string()),
        };

        // Send request to the test server.
        let req = test::TestRequest::post()
            .uri("/api/designer/v1/apps/unload")
            .set_json(request_payload)
            .to_request();
        let resp = test::call_service(&app, req).await;

        // Verify the response is an error.
        assert_eq!(resp.status(), actix_web::http::StatusCode::BAD_REQUEST);

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();
        let error_response: ErrorResponse =
            serde_json::from_str(body_str).unwrap();

        assert_eq!(error_response.status, Status::Fail);
        assert!(error_response.message.contains("not loaded"));
    }
}
