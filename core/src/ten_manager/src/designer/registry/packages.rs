//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::sync::{Arc, RwLock};

use actix_web::{web, HttpResponse, Responder};
use semver::VersionReq;
use serde::{Deserialize, Serialize};
use ten_rust::pkg_info::pkg_type::PkgType;

use crate::designer::response::{ApiResponse, ErrorResponse, Status};
use crate::designer::DesignerState;
use crate::registry;
use crate::registry::found_result::PkgRegistryInfo;

#[derive(Deserialize, Serialize, Debug)]
pub struct GetPackagesRequestPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub pkg_type: Option<PkgType>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub version_req: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct GetPackagesResponseData {
    pub packages: Vec<PkgRegistryInfo>,
}

pub async fn get_packages_endpoint(
    request_payload: web::Json<GetPackagesRequestPayload>,
    state: web::Data<Arc<RwLock<DesignerState>>>,
) -> Result<impl Responder, actix_web::Error> {
    // Extract what we need from state before await.
    let tman_config;
    let out;
    {
        let state_read = state.read().unwrap();
        tman_config = state_read.tman_config.clone();
        out = state_read.out.clone();
    } // Lock is dropped here.

    // Parse version requirement if provided.
    let version_req = if let Some(version_req_str) =
        &request_payload.version_req
    {
        match VersionReq::parse(version_req_str) {
            Ok(req) => Some(req),
            Err(e) => {
                return Ok(HttpResponse::BadRequest().json(ErrorResponse {
                    status: Status::Fail,
                    message: format!("Invalid version requirement: {}", e),
                    error: None,
                }));
            }
        }
    } else {
        None
    };

    // Call the registry function to get package list with optional parameters.
    match registry::get_package_list(
        tman_config,
        request_payload.pkg_type,
        request_payload.name.clone(),
        version_req,
        out,
    )
    .await
    {
        Ok(packages) => {
            let response_data = GetPackagesResponseData { packages };

            Ok(HttpResponse::Ok().json(ApiResponse {
                status: Status::Ok,
                data: response_data,
                meta: None,
            }))
        }
        Err(e) => Ok(HttpResponse::InternalServerError().json(ErrorResponse {
            status: Status::Fail,
            message: format!("Failed to get packages: {}", e),
            error: None,
        })),
    }
}
