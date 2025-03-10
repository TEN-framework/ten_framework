//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::sync::{Arc, RwLock};

use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

use ten_rust::pkg_info::pkg_type::PkgType;

use crate::designer::{
    app::base_dir::get_base_dir_from_pkgs_cache,
    response::{ApiResponse, ErrorResponse, Status},
    DesignerState,
};

#[derive(Serialize, Deserialize)]
pub struct CheckPropertyRequestPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub base_dir: Option<String>,

    #[serde(rename = "type")]
    check_type: String,
}

#[derive(Serialize, Deserialize)]
struct CheckPropertyResponseData {
    is_dirty: bool,
}

pub async fn check_property(
    request_payload: web::Json<CheckPropertyRequestPayload>,
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

    let all_pkgs = state_read.pkgs_cache.get(&base_dir).unwrap();

    if let Some(app_pkg) = all_pkgs
        .iter()
        .find(|pkg| pkg.basic_info.type_and_name.pkg_type == PkgType::App)
    {
        match request_payload.check_type.as_str() {
            "dirty" => match app_pkg.is_property_equal_to_fs() {
                Ok(is_dirty) => {
                    let response = ApiResponse {
                        status: Status::Ok,
                        data: CheckPropertyResponseData { is_dirty },
                        meta: None,
                    };

                    Ok(HttpResponse::Ok().json(response))
                }
                Err(err) => {
                    let error_response = ErrorResponse::from_error(
                        &err,
                        "Failed to check property:",
                    );
                    Ok(HttpResponse::NotFound().json(error_response))
                }
            },
            _ => Ok(HttpResponse::BadRequest().body("Invalid check type")),
        }
    } else {
        let error_response = ErrorResponse {
            status: Status::Fail,
            message: "Failed to find app package.".to_string(),
            error: None,
        };
        Ok(HttpResponse::NotFound().json(error_response))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::vec;

    use actix_web::{test, App};

    use super::*;
    use crate::{
        config::TmanConfig, constants::TEST_DIR,
        designer::mock::inject_all_pkgs_for_mock, output::TmanOutputCli,
    };

    #[actix_web::test]
    async fn test_check_property_is_not_dirty() {
        let mut designer_state = DesignerState {
            tman_config: TmanConfig::default(),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
        };

        // The first item is 'manifest.json', and the second item is
        // 'property.json'.
        let all_pkgs_json = vec![(
            include_str!("test_data_embed/app_manifest.json").to_string(),
            include_str!("test_data_embed/app_property.json").to_string(),
        )];

        let inject_ret = inject_all_pkgs_for_mock(
            TEST_DIR,
            &mut designer_state.pkgs_cache,
            all_pkgs_json,
        );
        assert!(inject_ret.is_ok());

        let designer_state = Arc::new(RwLock::new(designer_state));
        let app = test::init_service(
            App::new().app_data(web::Data::new(designer_state)).route(
                "/api/designer/v1/property/check",
                web::post().to(check_property),
            ),
        )
        .await;

        let request_payload = CheckPropertyRequestPayload {
            base_dir: Some(TEST_DIR.to_string()),
            check_type: "dirty".to_string(),
        };

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/property/check")
            .set_json(request_payload)
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();

        let check_response: ApiResponse<CheckPropertyResponseData> =
            serde_json::from_str(body_str).unwrap();

        assert!(!check_response.data.is_dirty);
    }
}
