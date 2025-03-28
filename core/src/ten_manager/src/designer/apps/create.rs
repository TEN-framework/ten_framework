//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::{
    fs,
    path::Path,
    sync::{Arc, RwLock},
};

use actix_web::{web, HttpResponse, Responder};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use crate::{
    designer::{
        response::{ApiResponse, ErrorResponse, Status},
        DesignerState,
    },
    fs::check_is_valid_dir,
    package_info::get_all_pkgs::get_all_pkgs,
};

#[derive(Deserialize, Serialize, Debug)]
pub struct CreateAppRequestPayload {
    pub base_dir: String,
    pub app_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateAppResponseData {
    pub app_path: String,
}

pub async fn create_app_endpoint(
    request_payload: web::Json<CreateAppRequestPayload>,
    state: web::Data<Arc<RwLock<DesignerState>>>,
) -> Result<impl Responder, actix_web::Error> {
    let CreateAppRequestPayload { base_dir, app_name } =
        request_payload.into_inner();

    // Validate base_dir exists.
    if !Path::new(&base_dir).exists() {
        let error_response = ErrorResponse::from_error(
            &anyhow!("Base directory does not exist"),
            "Base directory does not exist",
        );
        return Ok(HttpResponse::BadRequest().json(error_response));
    }

    // Validate base_dir is a directory.
    if let Err(err) = check_is_valid_dir(Path::new(&base_dir)) {
        let error_response =
            ErrorResponse::from_error(&err, "Invalid base directory");
        return Ok(HttpResponse::BadRequest().json(error_response));
    }

    // Create app directory.
    let app_path = Path::new(&base_dir).join(&app_name);

    // Check if app directory already exists.
    if app_path.exists() {
        let error_response = ErrorResponse::from_error(
            &anyhow!("App directory already exists"),
            "App directory already exists",
        );
        return Ok(HttpResponse::Conflict().json(error_response));
    }

    // Create app directory and basic structure.
    match fs::create_dir_all(&app_path) {
        Ok(_) => {
            // Create basic manifest file
            let manifest_path = app_path.join("manifest.json");
            let manifest_content = format!(
                r#"{{
  "type_and_name": {{
    "pkg_type": "app",
    "name": "{}"
  }},
  "version": "0.1.0",
  "description": "A new TEN app",
  "author": "",
  "license": "Apache-2.0"
}}"#,
                app_name
            );

            if let Err(err) = fs::write(&manifest_path, manifest_content) {
                // Clean up if manifest creation fails.
                let _ = fs::remove_dir_all(&app_path);
                let error_response = ErrorResponse::from_error(
                    &anyhow!("Failed to create manifest file: {}", err),
                    "Failed to create manifest file",
                );
                return Ok(
                    HttpResponse::InternalServerError().json(error_response)
                );
            }

            // Update cache with new app.
            let mut state_write = state.write().unwrap();
            let DesignerState {
                tman_config,
                pkgs_cache,
                out,
            } = &mut *state_write;

            let app_path_str = app_path.to_string_lossy().to_string();

            // Try to load the newly created app into the cache.
            if let Err(err) = get_all_pkgs(
                tman_config.clone(),
                pkgs_cache,
                &app_path_str,
                out,
            ) {
                // Don't delete the app directory on cache update failure.
                let error_response = ErrorResponse::from_error(
                    &err,
                    "App created but failed to update cache",
                );
                return Ok(HttpResponse::Ok().json(error_response));
            }

            let response = ApiResponse {
                status: Status::Ok,
                data: CreateAppResponseData {
                    app_path: app_path_str,
                },
                meta: None,
            };

            Ok(HttpResponse::Created().json(response))
        }
        Err(err) => {
            let error_response = ErrorResponse::from_error(
                &anyhow!("Failed to create app directory: {}", err),
                "Failed to create app directory",
            );
            Ok(HttpResponse::InternalServerError().json(error_response))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use actix_web::{test, App};
    use tempfile::tempdir;

    use super::*;
    use crate::{config::TmanConfig, output::TmanOutputCli};

    #[actix_web::test]
    async fn test_create_app_success() {
        // Create a temporary directory for testing
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_string_lossy().to_string();

        let designer_state = DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
        };
        let designer_state = Arc::new(RwLock::new(designer_state));

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(designer_state.clone()))
                .route(
                    "/api/designer/v1/apps/create",
                    web::post().to(create_app_endpoint),
                ),
        )
        .await;

        let create_app_request = CreateAppRequestPayload {
            base_dir: temp_path,
            app_name: "test_app".to_string(),
        };

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/apps/create")
            .set_json(&create_app_request)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_create_app_invalid_dir() {
        let designer_state = DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
        };
        let designer_state = Arc::new(RwLock::new(designer_state));

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(designer_state.clone()))
                .route(
                    "/api/designer/v1/apps/create",
                    web::post().to(create_app_endpoint),
                ),
        )
        .await;

        let create_app_request = CreateAppRequestPayload {
            base_dir: "/non/existent/directory".to_string(),
            app_name: "test_app".to_string(),
        };

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/apps/create")
            .set_json(&create_app_request)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_client_error());
    }
}
