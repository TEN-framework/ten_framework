//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
pub mod addons;
pub mod reload;
pub mod unload;

use std::{
    path::Path,
    sync::{Arc, RwLock},
};

use actix_web::{web, HttpResponse, Responder};
use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{
    designer::{
        response::{ApiResponse, ErrorResponse, Status},
        DesignerState,
    },
    fs::check_is_app_folder,
    package_info::get_all_pkgs::get_all_pkgs,
};

#[derive(Deserialize, Serialize, Debug)]
pub struct LoadAppRequestPayload {
    pub base_dir: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LoadAppResponseData {
    pub success: bool,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct GetAppsResponseData {
    pub base_dirs: Vec<String>,
}

pub async fn load_app_endpoint(
    request_payload: web::Json<LoadAppRequestPayload>,
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

pub async fn get_apps_endpoint(
    state: web::Data<Arc<RwLock<DesignerState>>>,
) -> Result<impl Responder, actix_web::Error> {
    let state = state.read().unwrap();
    let response = ApiResponse {
        status: Status::Ok,
        data: GetAppsResponseData {
            base_dirs: if state.pkgs_cache.is_empty() {
                vec![]
            } else {
                state.pkgs_cache.keys().cloned().collect()
            },
        },
        meta: None,
    };
    Ok(HttpResponse::Ok().json(response))
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
    async fn test_load_app_fail() {
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
                    "/api/designer/v1/apps",
                    web::post().to(load_app_endpoint),
                ),
        )
        .await;

        let new_base_dir = LoadAppRequestPayload {
            base_dir: "/not/a/correct/app/folder/path".to_string(),
        };

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/apps")
            .set_json(&new_base_dir)
            .to_request();
        let resp: Result<
            ApiResponse<LoadAppResponseData>,
            std::boxed::Box<dyn std::error::Error>,
        > = test::try_call_and_read_body_json(&app, req).await;

        assert!(resp.is_err());
    }

    #[actix_web::test]
    async fn test_get_apps_some() {
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
                    "/api/designer/v1/apps",
                    web::get().to(get_apps_endpoint),
                ),
        )
        .await;

        designer_state
            .write()
            .unwrap()
            .pkgs_cache
            .insert(TEST_DIR.to_string(), vec![]);

        let req = test::TestRequest::get()
            .uri("/api/designer/v1/apps")
            .to_request();

        let resp: ApiResponse<GetAppsResponseData> =
            test::call_and_read_body_json(&app, req).await;

        assert_eq!(resp.status, Status::Ok);
        assert_eq!(
            resp.data,
            GetAppsResponseData {
                base_dirs: vec![TEST_DIR.to_string()]
            }
        );
    }

    #[actix_web::test]
    async fn test_get_apps_none() {
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
                    "/api/designer/v1/apps",
                    web::get().to(get_apps_endpoint),
                ),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/api/designer/v1/apps")
            .to_request();

        let resp: ApiResponse<GetAppsResponseData> =
            test::call_and_read_body_json(&app, req).await;

        assert_eq!(resp.status, Status::Ok);
        assert_eq!(resp.data, GetAppsResponseData { base_dirs: vec![] });
    }
}
