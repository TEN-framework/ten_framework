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
                        // BaseDirPkgInfo.
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use actix_web::{test, App};
    use ten_rust::base_dir_pkg_info::BaseDirPkgInfo;

    use super::*;
    use crate::{
        config::TmanConfig, constants::TEST_DIR, output::TmanOutputCli,
    };

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

        // Create an empty BaseDirPkgInfo.
        let empty_pkg_info = BaseDirPkgInfo::default();

        designer_state
            .write()
            .unwrap()
            .pkgs_cache
            .insert(TEST_DIR.to_string(), empty_pkg_info);

        let req = test::TestRequest::get()
            .uri("/api/designer/v1/apps")
            .to_request();

        let resp: ApiResponse<GetAppsResponseData> =
            test::call_and_read_body_json(&app, req).await;

        assert_eq!(resp.status, Status::Ok);
        assert_eq!(
            resp.data,
            GetAppsResponseData {
                app_info: vec![AppInfo {
                    base_dir: TEST_DIR.to_string(),
                    app_uri: None,
                }]
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
        assert_eq!(resp.data, GetAppsResponseData { app_info: vec![] });
    }
}
