//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::{
    collections::HashMap,
    path::Path,
    sync::{Arc, RwLock},
};

use actix_web::{web, HttpResponse, Responder};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use ten_rust::pkg_info::PkgInfo;

use crate::{
    designer::{
        response::{ApiResponse, ErrorResponse, Status},
        DesignerState,
    },
    fs::check_is_app_folder,
    package_info::get_all_pkgs::get_all_pkgs,
};

#[derive(Deserialize, Serialize, Debug)]
pub struct AddBaseDirRequestPayload {
    pub base_dir: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AddBaseDirResponseData {
    pub success: bool,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct DeleteBaseDirRequestPayload {
    pub base_dir: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DeleteBaseDirResponseData {
    pub success: bool,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct GetBaseDirResponseData {
    pub base_dirs: Option<Vec<String>>,
}

pub async fn add_base_dir(
    request_payload: web::Json<AddBaseDirRequestPayload>,
    state: web::Data<Arc<RwLock<DesignerState>>>,
) -> Result<impl Responder, actix_web::Error> {
    let mut state = state.write().unwrap();

    if state.pkgs_cache.contains_key(&request_payload.base_dir) {
        let error_response = ErrorResponse::from_error(
            &anyhow::anyhow!("Base dir already exists"),
            "Base dir already exists",
        );
        return Ok(HttpResponse::NotFound().json(error_response));
    }

    match check_is_app_folder(Path::new(&request_payload.base_dir)) {
        Ok(_) => {
            let DesignerState {
                tman_config,
                pkgs_cache,
                out,
            } = &mut *state;

            if let Err(err) = get_all_pkgs(
                tman_config.clone(),
                pkgs_cache,
                &request_payload.base_dir,
                out,
            ) {
                let error_response =
                    ErrorResponse::from_error(&err, "Error fetching packages:");
                return Ok(HttpResponse::NotFound().json(error_response));
            }

            let response = ApiResponse {
                status: Status::Ok,
                data: serde_json::json!({ "success": true }),
                meta: None,
            };

            Ok(HttpResponse::Ok().json(response))
        }
        Err(err) => {
            let error_response = ErrorResponse::from_error(
                &err,
                format!("{} is not an app folder: ", &request_payload.base_dir)
                    .as_str(),
            );
            Ok(HttpResponse::NotFound().json(error_response))
        }
    }
}

pub async fn get_base_dir(
    state: web::Data<Arc<RwLock<DesignerState>>>,
) -> Result<impl Responder, actix_web::Error> {
    let state = state.read().unwrap();
    let response = ApiResponse {
        status: Status::Ok,
        data: GetBaseDirResponseData {
            base_dirs: (!state.pkgs_cache.is_empty())
                .then(|| state.pkgs_cache.keys().cloned().collect()),
        },
        meta: None,
    };
    Ok(HttpResponse::Ok().json(response))
}

pub async fn delete_base_dir(
    request_payload: web::Json<DeleteBaseDirRequestPayload>,
    state: web::Data<Arc<RwLock<DesignerState>>>,
) -> Result<impl Responder, actix_web::Error> {
    let mut state = state.write().unwrap();
    state.pkgs_cache.remove(&request_payload.base_dir);

    let response = ApiResponse {
        status: Status::Ok,
        data: serde_json::json!({ "success": true }),
        meta: None,
    };

    Ok(HttpResponse::Ok().json(response))
}

pub fn get_base_dir_from_pkgs_cache(
    base_dir: Option<String>,
    pkgs_cache: &HashMap<String, Vec<PkgInfo>>,
) -> Result<String> {
    // Determine base_dir based on the requirements.
    let base_dir = match base_dir {
        // Use provided base_dir if available.
        Some(dir) => dir.clone(),
        // If not provided, check pkgs_cache.
        None => {
            if pkgs_cache.len() == 1 {
                // If only one item in pkgs_cache, use it as base_dir.
                pkgs_cache.keys().next().unwrap().clone()
            } else {
                // If multiple items in pkgs_cache, return error.
                return Err(anyhow::anyhow!(
                    "Multiple apps available. Please specify base_dir."
                ));
            }
        }
    };

    Ok(base_dir)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use actix_web::{test, App};

    use super::*;
    use crate::{
        config::TmanConfig, constants::TEST_DIR, output::TmanOutputCli,
    };

    #[actix_web::test]
    async fn test_add_base_dir_fail() {
        let designer_state = DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
        };
        let designer_state = Arc::new(RwLock::new(designer_state));

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(designer_state.clone()))
                .route(
                    "/api/designer/v1/app/base-dir",
                    web::post().to(add_base_dir),
                ),
        )
        .await;

        let new_base_dir = AddBaseDirRequestPayload {
            base_dir: "/not/a/correct/app/folder/path".to_string(),
        };

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/app/base-dir")
            .set_json(&new_base_dir)
            .to_request();
        let resp: Result<
            ApiResponse<AddBaseDirResponseData>,
            std::boxed::Box<dyn std::error::Error>,
        > = test::try_call_and_read_body_json(&app, req).await;

        assert!(resp.is_err());
    }

    #[actix_web::test]
    async fn test_get_base_dir_some() {
        let designer_state = DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
        };

        let designer_state = Arc::new(RwLock::new(designer_state));

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(designer_state.clone()))
                .route(
                    "/api/designer/v1/app/base-dir",
                    web::get().to(get_base_dir),
                ),
        )
        .await;

        designer_state
            .write()
            .unwrap()
            .pkgs_cache
            .insert(TEST_DIR.to_string(), vec![]);

        let req = test::TestRequest::get()
            .uri("/api/designer/v1/app/base-dir")
            .to_request();

        let resp: ApiResponse<GetBaseDirResponseData> =
            test::call_and_read_body_json(&app, req).await;

        assert_eq!(resp.status, Status::Ok);
        assert_eq!(
            resp.data,
            GetBaseDirResponseData {
                base_dirs: Some(vec![TEST_DIR.to_string()])
            }
        );
    }

    #[actix_web::test]
    async fn test_get_base_dir_none() {
        let designer_state = DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
        };
        let designer_state = Arc::new(RwLock::new(designer_state));

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(designer_state.clone()))
                .route(
                    "/api/designer/v1/app/base-dir",
                    web::get().to(get_base_dir),
                ),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/api/designer/v1/app/base-dir")
            .to_request();

        let resp: ApiResponse<GetBaseDirResponseData> =
            test::call_and_read_body_json(&app, req).await;

        assert_eq!(resp.status, Status::Ok);
        assert_eq!(resp.data, GetBaseDirResponseData { base_dirs: None });
    }
}
