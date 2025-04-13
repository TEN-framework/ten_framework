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
    pkg_info::get_all_pkgs::get_all_pkgs_in_app,
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

    let tman_config_clone;
    let out_clone;
    {
        let state_read = state.read().map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!(
                "Failed to acquire read lock: {}",
                e
            ))
        })?;

        // Destructure to avoid multiple mutable borrows.
        let DesignerState {
            tman_config, out, ..
        } = &*state_read;

        // Clone the values we need
        tman_config_clone = tman_config.clone();
        out_clone = out.clone();
    }

    // Create app using create_pkg_in_path.
    match create_pkg_in_path(
        &tman_config_clone,
        Path::new(&base_dir),
        &PkgType::App,
        &app_name,
        &template_name,
        &VersionReq::default(),
        None,
        &out_clone,
    )
    .await
    {
        Ok(_) => {
            let app_path_str = app_path.to_string_lossy().to_string();

            let mut state_write = state.write().map_err(|e| {
                actix_web::error::ErrorInternalServerError(format!(
                    "Failed to acquire write lock: {}",
                    e
                ))
            })?;

            // Destructure to avoid multiple mutable borrows.
            let DesignerState {
                pkgs_cache,
                graphs_cache,
                ..
            } = &mut *state_write;

            // Try to load the newly created app into the cache.
            if let Err(err) =
                get_all_pkgs_in_app(pkgs_cache, graphs_cache, &app_path_str)
            {
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
