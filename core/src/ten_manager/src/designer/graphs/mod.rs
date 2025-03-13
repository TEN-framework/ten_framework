//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
pub mod connections;
pub mod nodes;
pub mod update;

use std::sync::{Arc, RwLock};

use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

use ten_rust::pkg_info::pkg_type::PkgType;

use super::{
    app::base_dir::get_base_dir_from_pkgs_cache,
    response::{ApiResponse, ErrorResponse, Status},
    DesignerState,
};

#[derive(Serialize, Deserialize)]
pub struct GetGraphsRequestPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub base_dir: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct GetGraphsResponseData {
    name: String,
    auto_start: bool,
}

pub async fn get_graphs(
    request_payload: web::Json<GetGraphsRequestPayload>,
    state: web::Data<Arc<RwLock<DesignerState>>>,
) -> Result<impl Responder, actix_web::Error> {
    let state_read = state.read().unwrap();

    let base_dir = match get_base_dir_from_pkgs_cache(
        request_payload.base_dir.clone(),
        &state_read.pkgs_cache,
    ) {
        Ok(base_dir) => base_dir,
        Err(e) => {
            let error_response = ErrorResponse {
                status: Status::Fail,
                message: e.to_string(),
                error: None,
            };
            return Ok(HttpResponse::BadRequest().json(error_response));
        }
    };

    if let Some(pkgs) = &state_read.pkgs_cache.get(&base_dir) {
        if let Some(app_pkg) = pkgs
            .iter()
            .find(|pkg| pkg.basic_info.type_and_name.pkg_type == PkgType::App)
        {
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{
        config::TmanConfig, constants::TEST_DIR,
        designer::mock::inject_all_pkgs_for_mock, output::TmanOutputCli,
    };

    use super::*;
    use actix_web::{test, App};

    #[actix_web::test]
    async fn test_get_graphs_success() {
        let mut designer_state = DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
        };

        let all_pkgs_json = vec![
            (
                include_str!("test_data_embed/app_manifest.json").to_string(),
                include_str!("test_data_embed/app_property.json").to_string(),
            ),
            (
                include_str!("test_data_embed/extension_addon_1_manifest.json")
                    .to_string(),
                "{}".to_string(),
            ),
            (
                include_str!("test_data_embed/extension_addon_2_manifest.json")
                    .to_string(),
                "{}".to_string(),
            ),
            (
                include_str!("test_data_embed/extension_addon_3_manifest.json")
                    .to_string(),
                "{}".to_string(),
            ),
        ];

        let inject_ret = inject_all_pkgs_for_mock(
            TEST_DIR,
            &mut designer_state.pkgs_cache,
            all_pkgs_json,
        );
        assert!(inject_ret.is_ok());

        let designer_state = Arc::new(RwLock::new(designer_state));

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(designer_state))
                .route("/api/designer/v1/graphs", web::post().to(get_graphs)),
        )
        .await;

        let request_payload = GetGraphsRequestPayload {
            base_dir: Some(TEST_DIR.to_string()),
        };

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/graphs")
            .set_json(request_payload)
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();

        let graphs: ApiResponse<Vec<GetGraphsResponseData>> =
            serde_json::from_str(body_str).unwrap();

        let expected_graphs = vec![
            GetGraphsResponseData {
                name: "default".to_string(),
                auto_start: true,
            },
            GetGraphsResponseData {
                name: "addon_not_found".to_string(),
                auto_start: false,
            },
        ];

        assert_eq!(graphs.data, expected_graphs);

        let json: ApiResponse<Vec<GetGraphsResponseData>> =
            serde_json::from_str(body_str).unwrap();
        let pretty_json = serde_json::to_string_pretty(&json).unwrap();
        println!("Response body: {}", pretty_json);
    }

    #[actix_web::test]
    async fn test_get_graphs_no_app_package() {
        let designer_state = Arc::new(RwLock::new(DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
        }));

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(designer_state))
                .route("/api/designer/v1/graphs", web::post().to(get_graphs)),
        )
        .await;

        let request_payload = GetGraphsRequestPayload {
            base_dir: Some(TEST_DIR.to_string()),
        };

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/graphs")
            .set_json(request_payload)
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_client_error());
        println!("Response body: {}", resp.status());

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();

        let error: ErrorResponse = serde_json::from_str(body_str).unwrap();

        assert_eq!(error.message, "All packages not available");
    }
}
