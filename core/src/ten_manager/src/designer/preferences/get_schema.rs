//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use actix_web::{HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::designer::response::{ApiResponse, Status};
use crate::schema::get_designer_schema;

#[derive(Debug, Serialize, Deserialize)]
pub struct GetPreferencesSchemaResponseData {
    pub schema: Value,
}

/// Get the JSON schema for designer frontend preferences.
pub async fn get_preferences_schema_endpoint(
) -> Result<impl Responder, actix_web::Error> {
    let schema: Value = serde_json::from_str(get_designer_schema())
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let response_data = GetPreferencesSchemaResponseData { schema };
    let response = ApiResponse {
        status: Status::Ok,
        data: response_data,
        meta: None,
    };

    Ok(HttpResponse::Ok().json(response))
}

#[cfg(test)]
mod tests {
    use actix_web::{test, web, App};

    use super::*;

    #[actix_web::test]
    async fn test_get_preferences_schema_success() {
        // Create test app.
        let app = test::init_service(App::new().service(
            web::scope("/api/designer/v1").route(
                "/preferences/schema",
                web::get().to(get_preferences_schema_endpoint),
            ),
        ))
        .await;

        // Create test request.
        let req = test::TestRequest::get()
            .uri("/api/designer/v1/preferences/schema")
            .to_request();
        let resp = test::call_service(&app, req).await;

        // Assert response status is 200 OK.
        assert!(resp.status().is_success());

        // Parse response body.
        let body = test::read_body(resp).await;
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        // Assert response structure.
        assert_eq!(json["status"], "ok");
        assert!(json["data"].is_object());
        assert!(json["data"]["schema"].is_object());

        // Verify schema contains the expected properties.
        let schema = &json["data"]["schema"];
        assert_eq!(schema["title"], "Designer");
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"].is_object());
        assert!(schema["properties"]["logviewer_line_size"].is_object());
    }

    #[actix_web::test]
    async fn test_get_preferences_schema_invalid_path() {
        // Create test app.
        let app = test::init_service(App::new().service(
            web::scope("/api/designer/v1").route(
                "/preferences/schema",
                web::get().to(get_preferences_schema_endpoint),
            ),
        ))
        .await;

        // Create test request with invalid path.
        let req = test::TestRequest::get()
            .uri("/api/designer/v1/preferences/invalid")
            .to_request();
        let resp = test::call_service(&app, req).await;

        // Assert response status is 404 Not Found.
        assert_eq!(resp.status(), 404);
    }
}
