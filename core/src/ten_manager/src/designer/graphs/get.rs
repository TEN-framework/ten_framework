//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::sync::{Arc, RwLock};

use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

use crate::designer::{
    response::{ApiResponse, ErrorResponse, Status},
    DesignerState,
};

#[derive(Serialize, Deserialize)]
pub struct GetGraphsRequestPayload {
    pub base_dir: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct GetGraphsResponseData {
    pub name: String,
    pub auto_start: bool,
}

pub async fn get_graphs_endpoint(
    request_payload: web::Json<GetGraphsRequestPayload>,
    state: web::Data<Arc<RwLock<DesignerState>>>,
) -> Result<impl Responder, actix_web::Error> {
    let state_read = state.read().unwrap();

    if let Some(base_dir_pkg_info) =
        &state_read.pkgs_cache.get(&request_payload.base_dir)
    {
        if let Some(app_pkg) = &base_dir_pkg_info.app_pkg_info {
            let graphs: Vec<GetGraphsResponseData> = app_pkg
                .get_predefined_graphs()
                .unwrap_or(&vec![])
                .iter()
                .map(|graph| GetGraphsResponseData {
                    name: graph.name.clone(),
                    auto_start: graph.auto_start.unwrap_or(false),
                })
                .collect();

            let response = ApiResponse {
                status: Status::Ok,
                data: graphs,
                meta: None,
            };

            Ok(HttpResponse::Ok().json(response))
        } else {
            let error_response = ErrorResponse {
                status: Status::Fail,
                message: "Failed to find any app packages".to_string(),
                error: None,
            };
            Ok(HttpResponse::NotFound().json(error_response))
        }
    } else {
        let error_response = ErrorResponse {
            status: Status::Fail,
            message: "All packages not available".to_string(),
            error: None,
        };
        Ok(HttpResponse::NotFound().json(error_response))
    }
}
