//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::sync::{Arc, RwLock};

use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

use super::{
    response::{ApiResponse, Status},
    DesignerState,
};
use crate::{
    constants::GITHUB_RELEASE_PAGE, version::VERSION,
    version_utils::check_update,
};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct GetVersionResponseData {
    version: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct CheckUpdateResponseData {
    update_available: bool,
    latest_version: Option<String>,
    release_page: Option<String>,
    message: Option<String>,
}

pub async fn get_version(
    _state: web::Data<Arc<RwLock<DesignerState>>>,
) -> Result<impl Responder, actix_web::Error> {
    let version_info = GetVersionResponseData {
        version: VERSION.to_string(),
    };

    let response = ApiResponse {
        status: Status::Ok,
        data: version_info,
        meta: None,
    };

    Ok(HttpResponse::Ok().json(response))
}

pub async fn check_update_endpoint() -> Result<impl Responder, actix_web::Error>
{
    match check_update().await {
        Ok((true, latest)) => {
            let update_info = CheckUpdateResponseData {
                update_available: true,
                latest_version: Some(latest),
                release_page: Some(GITHUB_RELEASE_PAGE.to_string()),
                message: None,
            };
            let response = ApiResponse {
                status: Status::Ok,
                data: update_info,
                meta: None,
            };
            Ok(HttpResponse::Ok().json(response))
        }
        Ok((false, _)) => {
            let update_info = CheckUpdateResponseData {
                update_available: false,
                latest_version: None,
                release_page: None,
                message: None,
            };
            let response = ApiResponse {
                status: Status::Ok,
                data: update_info,
                meta: None,
            };
            Ok(HttpResponse::Ok().json(response))
        }
        Err(err_msg) => {
            let update_info = CheckUpdateResponseData {
                update_available: false,
                latest_version: None,
                release_page: None,
                message: Some(err_msg),
            };
            let response = ApiResponse {
                status: Status::Fail,
                data: update_info,
                meta: None,
            };
            Ok(HttpResponse::Ok().json(response))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use actix_web::{http::StatusCode, test, App};

    use super::*;
    use crate::{config::TmanConfig, output::TmanOutputCli};

    #[actix_web::test]
    async fn test_get_version() {
        // Initialize the DesignerState.
        let state = web::Data::new(Arc::new(RwLock::new(DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
        })));

        // Create the App with the routes configured.
        let app = test::init_service(
            App::new()
                .app_data(state.clone())
                .route("/api/designer/v1/version", web::get().to(get_version)),
        )
        .await;

        // Send a request to the version endpoint.
        let req = test::TestRequest::get()
            .uri("/api/designer/v1/version")
            .to_request();
        let resp = test::call_service(&app, req).await;

        // Check that the response status is 200 OK.
        assert_eq!(resp.status(), StatusCode::OK);

        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();

        let version: ApiResponse<GetVersionResponseData> =
            serde_json::from_str(body_str).unwrap();

        // Create the expected Version struct
        let expected_version = GetVersionResponseData {
            version: VERSION.to_string(),
        };

        // Compare the actual Version struct with the expected one
        assert_eq!(version.data, expected_version);

        let json: ApiResponse<GetVersionResponseData> =
            serde_json::from_str(body_str).unwrap();
        let pretty_json = serde_json::to_string_pretty(&json).unwrap();
        println!("Response body: {}", pretty_json);
    }
}
