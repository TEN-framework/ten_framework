//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::sync::{Arc, RwLock};

use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

use ten_rust::pkg_info::pkg_type::PkgType;

use crate::designer::{
    response::{ApiResponse, ErrorResponse, Status},
    DesignerState,
};

#[derive(Deserialize)]
pub struct CheckManifestRequestPayload {
    base_dir: String,

    #[serde(rename = "type")]
    check_type: String,
}

#[derive(Serialize)]
struct CheckManifestResponseData {
    is_dirty: bool,
}

pub async fn check_manifest(
    request_payload: web::Json<CheckManifestRequestPayload>,
    state: web::Data<Arc<RwLock<DesignerState>>>,
) -> impl Responder {
    let state = state.read().unwrap();
    let all_pkgs = state.pkgs_cache.get(&request_payload.base_dir).unwrap();

    if let Some(app_pkg) = all_pkgs
        .iter()
        .find(|pkg| pkg.basic_info.type_and_name.pkg_type == PkgType::App)
    {
        match request_payload.check_type.as_str() {
            "dirty" => match app_pkg.is_manifest_equal_to_fs() {
                Ok(is_dirty) => {
                    let response = ApiResponse {
                        status: Status::Ok,
                        data: CheckManifestResponseData { is_dirty },
                        meta: None,
                    };

                    HttpResponse::Ok().json(response)
                }
                Err(err) => {
                    let error_response = ErrorResponse::from_error(
                        &err,
                        "Failed to check manifest:",
                    );
                    HttpResponse::NotFound().json(error_response)
                }
            },
            _ => HttpResponse::BadRequest().body("Invalid check type"),
        }
    } else {
        let error_response = ErrorResponse {
            status: Status::Fail,
            message: "Failed to find app package.".to_string(),
            error: None,
        };
        HttpResponse::NotFound().json(error_response)
    }
}
