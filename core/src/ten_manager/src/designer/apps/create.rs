//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::{
    path::Path,
    sync::{Arc, RwLock},
};

use actix_web::{web, HttpResponse, Responder};
use anyhow::{anyhow, Result};
use semver::VersionReq;
use serde::{Deserialize, Serialize};

use crate::{
    create::create_pkg_in_path,
    designer::{
        response::{ApiResponse, ErrorResponse, Status},
        DesignerState,
    },
    fs::check_is_valid_dir,
    pkg_info::get_all_pkgs::get_all_pkgs,
};

use ten_rust::pkg_info::pkg_type::PkgType;

#[derive(Deserialize, Serialize, Debug)]
pub struct CreateAppRequestPayload {
    pub base_dir: String,
    pub template_name: String,
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
    let CreateAppRequestPayload {
        base_dir,
        app_name,
        template_name,
    } = request_payload.into_inner();

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

    // Clone necessary values from state before the async call.
    let tman_config_clone;
    let out_clone;

    {
        let state_write = state.write().unwrap();
        tman_config_clone = state_write.tman_config.clone();
        out_clone = state_write.out.clone();
    }

    // Create app using create_pkg_in_path.
    match create_pkg_in_path(
        tman_config_clone,
        Path::new(&base_dir),
        &PkgType::App,
        &app_name,
        &template_name,
        &VersionReq::default(),
        None,
        out_clone,
    )
    .await
    {
        Ok(_) => {
            let app_path_str = app_path.to_string_lossy().to_string();

            // Re-acquire the lock for updating the cache.
            let mut state_write = state.write().unwrap();
            let DesignerState {
                tman_config,
                pkgs_cache,
                out,
            } = &mut *state_write;

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
                &anyhow!("Failed to create app: {}", err),
                "Failed to create app",
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
    use crate::{
        config::TmanConfig, constants::DEFAULT_APP_CPP, output::TmanOutputCli,
    };

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
            template_name: DEFAULT_APP_CPP.to_string(),
        };

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/apps/create")
            .set_json(&create_app_request)
            .to_request();

        let resp = test::call_service(&app, req).await;
        if !resp.status().is_success() {
            println!("resp: {:?}", resp);

            let body = test::read_body(resp).await;
            let body_str = std::str::from_utf8(&body).unwrap();

            println!("body: {:?}", body_str);

            panic!("Failed to create app");
        }
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
            template_name: "default".to_string(),
        };

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/apps/create")
            .set_json(&create_app_request)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_client_error());
    }
}
