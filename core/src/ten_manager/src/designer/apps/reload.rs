//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::sync::{Arc, RwLock};

use crate::{
    designer::{
        response::{ApiResponse, ErrorResponse, Status},
        DesignerState,
    },
    pkg_info::get_all_pkgs::get_all_pkgs,
};
use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ReloadPkgsRequestPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub base_dir: Option<String>,
}

pub async fn reload_app_endpoint(
    request_payload: web::Json<ReloadPkgsRequestPayload>,
    state: web::Data<Arc<RwLock<DesignerState>>>,
) -> Result<impl Responder, actix_web::Error> {
    let mut state_write = state.write().unwrap();
    let DesignerState {
        tman_config,
        pkgs_cache,
        out,
    } = &mut *state_write;

    if let Some(base_dir) = &request_payload.base_dir {
        // Case 1: base_dir is specified in the request payload.

        // Check if the base_dir exists in pkgs_cache.
        if !pkgs_cache.contains_key(base_dir) {
            let error_response = ErrorResponse {
                status: Status::Fail,
                message: format!(
                    "App with base_dir '{}' is not loaded",
                    base_dir
                ),
                error: None,
            };
            return Ok(HttpResponse::BadRequest().json(error_response));
        }

        // Clear the existing packages for this base_dir.
        pkgs_cache.remove(base_dir);

        // Reload packages for this base_dir.
        if let Err(err) =
            get_all_pkgs(tman_config.clone(), pkgs_cache, base_dir, out)
        {
            return Ok(HttpResponse::InternalServerError().json(
                ErrorResponse::from_error(&err, "Failed to reload packages:"),
            ));
        }
    } else {
        // Case 2: base_dir is not specified, reload all apps.

        if pkgs_cache.is_empty() {
            return Ok(HttpResponse::Ok().json(ApiResponse {
                status: Status::Ok,
                data: "No apps to reload",
                meta: None,
            }));
        }

        // Collect all base_dirs first (to avoid borrowing issues).
        let base_dirs: Vec<String> = pkgs_cache.keys().cloned().collect();

        // For each base_dir, clear its packages and reload them.
        for base_dir in base_dirs {
            // Clear the existing packages for this base_dir.
            pkgs_cache.remove(&base_dir);

            // Reload packages for this base_dir.
            if let Err(err) =
                get_all_pkgs(tman_config.clone(), pkgs_cache, &base_dir, out)
            {
                return Ok(HttpResponse::InternalServerError().json(
                    ErrorResponse::from_error(
                        &err,
                        "Failed to reload packages:",
                    ),
                ));
            }
        }
    }

    Ok(HttpResponse::Ok().json(ApiResponse {
        status: Status::Ok,
        data: "Packages reloaded successfully",
        meta: None,
    }))
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use actix_web::{http::StatusCode, test, App};
    use ten_rust::base_dir_pkg_info::BaseDirPkgInfo;

    use super::*;
    use crate::{
        config::TmanConfig, constants::TEST_DIR,
        designer::mock::inject_all_pkgs_for_mock, output::TmanOutputCli,
    };

    /// Test error case when the specified base_dir doesn't exist in pkgs_cache.
    #[actix_web::test]
    async fn test_reload_app_error_base_dir_not_found() {
        // Set up the designer state.
        let mut designer_state = DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
        };

        // Inject initial package data.
        let initial_pkgs_json = vec![(
            include_str!("test_data_embed/app_manifest.json").to_string(),
            include_str!("test_data_embed/app_property.json").to_string(),
        )];

        let inject_ret = inject_all_pkgs_for_mock(
            TEST_DIR,
            &mut designer_state.pkgs_cache,
            initial_pkgs_json,
        );
        assert!(inject_ret.is_ok());

        let designer_state = Arc::new(RwLock::new(designer_state));

        // Set up the test service.
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(designer_state.clone()))
                .route(
                    "/api/designer/v1/apps/reload",
                    web::post().to(reload_app_endpoint),
                ),
        )
        .await;

        // Create request with non-existent base_dir.
        let request_payload = ReloadPkgsRequestPayload {
            base_dir: Some("/non/existent/base/dir".to_string()),
        };

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/apps/reload")
            .set_json(request_payload)
            .to_request();

        // Send the request and get the response.
        let resp = test::call_service(&app, req).await;

        // Verify the response is an error.
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();

        let error_response: ErrorResponse =
            serde_json::from_str(body_str).unwrap();
        assert_eq!(error_response.status, Status::Fail);
        assert_eq!(
            error_response.message,
            "App with base_dir '/non/existent/base/dir' is not loaded"
        );
    }

    /// Test the scenario where get_all_pkgs fails during reload.
    #[actix_web::test]
    async fn test_reload_app_error_get_all_pkgs_fails() {
        // For this test, we'll need to mock get_all_pkgs to fail. This is a
        // bit complicated in Rust without mocking frameworks. As a basic
        // approach, we'll remove the base_dir from pkgs_cache and then use a
        // non-existent path in base_dir that will likely cause get_all_pkgs to
        // fail.

        // Set up the designer state.
        let mut designer_state = DesignerState {
            tman_config: Arc::new(TmanConfig {
                verbose: true,
                ..TmanConfig::default()
            }),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
        };

        // We'll use a special path that should cause get_all_pkgs to fail.
        // This is a bit of a hack but should work for testing purposes.
        let invalid_path = "/definitely/invalid/path/that/doesnt/exist";

        // Create an empty BaseDirPkgInfo.
        let empty_pkg_info = BaseDirPkgInfo::default();

        // Inject an entry with the invalid path.
        designer_state
            .pkgs_cache
            .insert(invalid_path.to_string(), empty_pkg_info);

        let designer_state = Arc::new(RwLock::new(designer_state));

        // Set up the test service.
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(designer_state.clone()))
                .route(
                    "/api/designer/v1/apps/reload",
                    web::post().to(reload_app_endpoint),
                ),
        )
        .await;

        // Create request with the invalid base_dir.
        let request_payload = ReloadPkgsRequestPayload {
            base_dir: Some(invalid_path.to_string()),
        };

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/apps/reload")
            .set_json(request_payload)
            .to_request();

        // Send the request and get the response.
        let resp = test::call_service(&app, req).await;

        // Verify the response is an error - this should be 500 Internal Server
        // Error.
        assert!(resp.status().is_server_error());

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();

        let error_response: ErrorResponse =
            serde_json::from_str(body_str).unwrap();
        assert_eq!(error_response.status, Status::Fail);
        // The exact error message will depend on the implementation of
        // get_all_pkgs.
        assert!(error_response
            .message
            .contains("Failed to reload packages:"));
    }
}
