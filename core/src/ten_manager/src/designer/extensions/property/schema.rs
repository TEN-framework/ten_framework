//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::sync::{Arc, RwLock};

use actix_web::{web, HttpResponse, Responder};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use crate::designer::{
    response::{ApiResponse, ErrorResponse, Status},
    DesignerState,
};

#[derive(Deserialize, Serialize, Debug)]
pub struct GetExtensionPropertySchemaRequestPayload {
    pub app_base_dir: String,
    pub addon_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetExtensionPropertySchemaResponseData {
    pub property_schema_json_str: Option<String>,
}

pub async fn get_extension_property_schema_endpoint(
    request_payload: web::Json<GetExtensionPropertySchemaRequestPayload>,
    state: web::Data<Arc<RwLock<DesignerState>>>,
) -> Result<impl Responder, actix_web::Error> {
    let state_read = state.read().unwrap();

    let pkgs_cache = &state_read.pkgs_cache;

    if let Some(pkgs_info_in_app) =
        pkgs_cache.get(&request_payload.app_base_dir)
    {
        if let Some(extension_pkgs_info) = &pkgs_info_in_app.extension_pkgs_info
        {
            let extension_pkg_info =
                extension_pkgs_info.iter().find(|pkg_info| {
                    pkg_info.manifest.type_and_name.name
                        == request_payload.addon_name
                });

            if let Some(extension_pkg_info) = extension_pkg_info {
                let response = ApiResponse {
                    status: Status::Ok,
                    data: GetExtensionPropertySchemaResponseData {
                        property_schema_json_str: extension_pkg_info
                            .schema_store
                            .as_ref()
                            .and_then(|schema_store| {
                                schema_store.property_str.clone()
                            }),
                    },
                    meta: None,
                };

                Ok(HttpResponse::Ok().json(response))
            } else {
                let error_response = ErrorResponse::from_error(
                    &anyhow!("Extension not found"),
                    "Extension not found",
                );
                Ok(HttpResponse::NotFound().json(error_response))
            }
        } else {
            let error_response = ErrorResponse::from_error(
                &anyhow!("Extension not found"),
                "Extension not found",
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
