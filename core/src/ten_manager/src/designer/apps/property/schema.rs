//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use actix_web::{web, HttpResponse, Responder};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use ten_rust::pkg_info::manifest::api::ManifestPropertyAttributes;

use crate::designer::{
    response::{ApiResponse, ErrorResponse, Status},
    DesignerState,
};

#[derive(Deserialize, Serialize, Debug)]
pub struct GetAppPropertySchemaRequestPayload {
    pub app_base_dir: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetAppPropertySchemaResponseData {
    pub property_schema: Option<HashMap<String, ManifestPropertyAttributes>>,
}

pub async fn get_app_property_schema_endpoint(
    request_payload: web::Json<GetAppPropertySchemaRequestPayload>,
    state: web::Data<Arc<RwLock<DesignerState>>>,
) -> Result<impl Responder, actix_web::Error> {
    let state_read = state.read().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!(
            "Failed to acquire read lock: {}",
            e
        ))
    })?;

    let pkgs_cache = &state_read.pkgs_cache;

    if let Some(pkgs_info_in_app) =
        pkgs_cache.get(&request_payload.app_base_dir)
    {
        if let Some(app_pkg_info) = &pkgs_info_in_app.app_pkg_info {
            let response = ApiResponse {
                status: Status::Ok,
                data: GetAppPropertySchemaResponseData {
                    property_schema: app_pkg_info
                        .manifest
                        .api
                        .as_ref()
                        .and_then(|api| api.property.clone()),
                },
                meta: None,
            };

            Ok(HttpResponse::Ok().json(response))
        } else {
            let error_response = ErrorResponse::from_error(
                &anyhow!("App not found"),
                "App not found",
            );
            Ok(HttpResponse::NotFound().json(error_response))
        }
    } else {
        let error_response = ErrorResponse::from_error(
            &anyhow!("App not found"),
            "App not found",
        );
        Ok(HttpResponse::NotFound().json(error_response))
    }
}
