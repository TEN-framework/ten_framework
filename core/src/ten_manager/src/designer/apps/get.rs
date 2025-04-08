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

use crate::designer::{
    response::{ApiResponse, Status},
    DesignerState,
};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct AppInfo {
    pub base_dir: String,
    pub app_uri: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct GetAppsResponseData {
    pub app_info: Vec<AppInfo>,
}

pub async fn get_apps_endpoint(
    state: web::Data<Arc<RwLock<DesignerState>>>,
) -> Result<impl Responder, actix_web::Error> {
    let state_read = state.read().unwrap();

    let response = ApiResponse {
        status: Status::Ok,
        data: GetAppsResponseData {
            app_info: if state_read.pkgs_cache.is_empty() {
                vec![]
            } else {
                state_read
                    .pkgs_cache
                    .iter()
                    .map(|(base_dir, base_dir_pkg_info)| {
                        // Get the App package info directly from
                        // PkgsInfoInApp.
                        let app_uri = base_dir_pkg_info
                            .app_pkg_info
                            .as_ref()
                            .and_then(|app_pkg_info| {
                                app_pkg_info.property.as_ref()
                            })
                            .and_then(|property| property._ten.as_ref())
                            .and_then(|ten| ten.uri.as_ref())
                            .map(|uri| uri.to_string());

                        AppInfo {
                            base_dir: base_dir.clone(),
                            app_uri,
                        }
                    })
                    .collect()
            },
        },
        meta: None,
    };
    Ok(HttpResponse::Ok().json(response))
}
