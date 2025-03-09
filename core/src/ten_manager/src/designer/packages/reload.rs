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
    package_info::get_all_pkgs::get_all_pkgs,
};
use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ReloadPkgsRequestPayload {
    base_dir: String,
}

pub async fn clear_and_reload_pkgs(
    request_payload: web::Json<ReloadPkgsRequestPayload>,
    state: web::Data<Arc<RwLock<DesignerState>>>,
) -> impl Responder {
    let mut state = state.write().unwrap();

    // Clear the existing packages.
    state.pkgs_cache.remove(&request_payload.base_dir);

    let DesignerState {
        tman_config,
        pkgs_cache,
        out,
    } = &mut *state;

    if let Err(err) =
        get_all_pkgs(tman_config, pkgs_cache, &request_payload.base_dir, out)
    {
        return HttpResponse::InternalServerError().json(
            ErrorResponse::from_error(&err, "Failed to reload packages:"),
        );
    }

    HttpResponse::Ok().json(ApiResponse {
        status: Status::Ok,
        data: "Packages reloaded successfully",
        meta: None,
    })
}
