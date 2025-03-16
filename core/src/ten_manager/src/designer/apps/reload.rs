//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::sync::{Arc, RwLock};

use crate::{
    designer::{
        apps::get_base_dir_from_pkgs_cache,
        response::{ApiResponse, ErrorResponse, Status},
        DesignerState,
    },
    package_info::get_all_pkgs::get_all_pkgs,
};
use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;

#[derive(Deserialize)]
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

    // Clear the existing packages.
    state_write.pkgs_cache.remove(&base_dir);

    let DesignerState {
        tman_config,
        pkgs_cache,
        out,
    } = &mut *state_write;

    if let Err(err) =
        get_all_pkgs(tman_config.clone(), pkgs_cache, &base_dir, out)
    {
        return Ok(HttpResponse::InternalServerError().json(
            ErrorResponse::from_error(&err, "Failed to reload packages:"),
        ));
    }

    Ok(HttpResponse::Ok().json(ApiResponse {
        status: Status::Ok,
        data: "Packages reloaded successfully",
        meta: None,
    }))
}
