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
use ten_rust::pkg_info::pkg_type::PkgType;

use crate::designer::{
    response::{ApiResponse, ErrorResponse, Status},
    DesignerState,
};

#[derive(Deserialize, Serialize, Debug)]
pub struct GetPackagesScriptsRequestPayload {
    pub base_dir: String,
    pub pkg_type: PkgType,
    pub pkg_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetPackagesScriptsResponseData {
    pub scripts: Option<Vec<String>>,
}

pub async fn get_packages_scripts_endpoint(
    request_payload: web::Json<GetPackagesScriptsRequestPayload>,
    state: web::Data<Arc<RwLock<DesignerState>>>,
) -> Result<impl Responder, actix_web::Error> {
    let state_read = state.read().unwrap();

    if let Some(pkgs) = &state_read.pkgs_cache.get(&request_payload.base_dir) {
        if let Some(pkg) = pkgs.iter().find(|pkg| {
            pkg.manifest.as_ref().is_some_and(|m| {
                m.type_and_name.pkg_type == request_payload.pkg_type
                    && m.type_and_name.name == request_payload.pkg_name
            })
        }) {
            let scripts = pkg
                .manifest
                .as_ref()
                .and_then(|m| m.scripts.as_ref())
                .map(|scripts| scripts.keys().cloned().collect());

            let response = ApiResponse {
                status: Status::Ok,
                data: GetPackagesScriptsResponseData { scripts },
                meta: None,
            };

            Ok(HttpResponse::Ok().json(response))
        } else {
            let error_response = ErrorResponse {
                status: Status::Fail,
                message: format!(
                    "Package {} of type {} not found",
                    request_payload.pkg_name, request_payload.pkg_type
                ),
                error: None,
            };
            Ok(HttpResponse::NotFound().json(error_response))
        }
    } else {
        let error_response = ErrorResponse {
            status: Status::Fail,
            message: "Package information is missing".to_string(),
            error: None,
        };
        Ok(HttpResponse::NotFound().json(error_response))
    }
}
