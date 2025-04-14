//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        sync::{Arc, RwLock},
    };

    use actix_web::{http::StatusCode, test, web, App};
    use ten_manager::{
        config::TmanConfig,
        designer::{
            property::validate::{
                validate_property_endpoint, ValidatePropertyRequestPayload,
            },
            DesignerState,
        },
        output::TmanOutputCli,
    };

    #[actix_web::test]
    async fn test_validate_property_valid_empty() {
        // Setup.
        let designer_state = DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            tman_internal_config: Arc::new(TmanInternalConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
            graphs_cache: HashMap::new(),
        };
        let state = web::Data::new(Arc::new(RwLock::new(designer_state)));

        let app = test::init_service(
            App::new().app_data(state.clone()).service(
                web::resource("/")
                    .route(web::post().to(validate_property_endpoint)),
            ),
        )
        .await;

        // Valid empty property JSON.
        let req = test::TestRequest::post()
            .uri("/")
            .set_json(&ValidatePropertyRequestPayload {
                property_json_str: "{}".to_string(),
            })
            .to_request();

        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::OK);
        let response_body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(response_body["status"], "ok");
        assert_eq!(response_body["data"]["is_valid"], true);
        assert_eq!(
            response_body["data"]["error_message"],
            serde_json::json!(null)
        );
    }

    #[actix_web::test]
    async fn test_validate_property_valid_with_basic_fields() {
        // Setup
        let designer_state = DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            tman_internal_config: Arc::new(TmanInternalConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
            graphs_cache: HashMap::new(),
        };
        let state = web::Data::new(Arc::new(RwLock::new(designer_state)));

        let app = test::init_service(
            App::new().app_data(state.clone()).service(
                web::resource("/")
                    .route(web::post().to(validate_property_endpoint)),
            ),
        )
        .await;

        // Valid property JSON with basic fields
        let req = test::TestRequest::post()
            .uri("/")
            .set_json(&ValidatePropertyRequestPayload {
                property_json_str: r#"
                {
                  "a": 1,
                  "b": "2",
                  "c": [1, 2, 3],
                  "d": {
                    "e": 1.0
                  }
                }
                "#
                .to_string(),
            })
            .to_request();

        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::OK);
        let response_body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(response_body["status"], "ok");
        assert_eq!(response_body["data"]["is_valid"], true);
        assert_eq!(
            response_body["data"]["error_message"],
            serde_json::json!(null)
        );
    }

    #[actix_web::test]
    async fn test_validate_property_valid_with_ten_field() {
        // Setup
        let designer_state = DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            tman_internal_config: Arc::new(TmanInternalConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
            graphs_cache: HashMap::new(),
        };
        let state = web::Data::new(Arc::new(RwLock::new(designer_state)));

        let app = test::init_service(
            App::new().app_data(state.clone()).service(
                web::resource("/")
                    .route(web::post().to(validate_property_endpoint)),
            ),
        )
        .await;

        // Valid property JSON with _ten field
        let req = test::TestRequest::post()
            .uri("/")
            .set_json(&ValidatePropertyRequestPayload {
                property_json_str: r#"
                {
                  "_ten": {
                    "log_level": 2,
                    "log": {
                      "level": 2,
                      "file": "api.log"
                    }
                  },
                  "a": 1,
                  "b": "2"
                }
                "#
                .to_string(),
            })
            .to_request();

        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::OK);
        let response_body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(response_body["status"], "ok");
        assert_eq!(response_body["data"]["is_valid"], true);
        assert_eq!(
            response_body["data"]["error_message"],
            serde_json::json!(null)
        );
    }

    #[actix_web::test]
    async fn test_validate_property_valid_with_predefined_graphs() {
        // Setup
        let designer_state = DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            tman_internal_config: Arc::new(TmanInternalConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
            graphs_cache: HashMap::new(),
        };
        let state = web::Data::new(Arc::new(RwLock::new(designer_state)));

        let app = test::init_service(
            App::new().app_data(state.clone()).service(
                web::resource("/")
                    .route(web::post().to(validate_property_endpoint)),
            ),
        )
        .await;

        // Valid property JSON with predefined_graphs
        let req = test::TestRequest::post()
            .uri("/")
            .set_json(&ValidatePropertyRequestPayload {
                property_json_str: r#"
                {
                  "_ten": {
                    "predefined_graphs": [
                      {
                        "name": "default",
                        "nodes": [
                          {
                            "type": "extension",
                            "name": "default_extension_cpp",
                            "addon": "default_extension_cpp",
                            "extension_group": "default_extension_group"
                          }
                        ]
                      }
                    ]
                  }
                }
                "#
                .to_string(),
            })
            .to_request();

        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::OK);
        let response_body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(response_body["status"], "ok");
        assert_eq!(response_body["data"]["is_valid"], true);
        assert_eq!(
            response_body["data"]["error_message"],
            serde_json::json!(null)
        );
    }

    #[actix_web::test]
    async fn test_validate_property_invalid_json_syntax() {
        // Setup
        let designer_state = DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            tman_internal_config: Arc::new(TmanInternalConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
            graphs_cache: HashMap::new(),
        };
        let state = web::Data::new(Arc::new(RwLock::new(designer_state)));

        let app = test::init_service(
            App::new().app_data(state.clone()).service(
                web::resource("/")
                    .route(web::post().to(validate_property_endpoint)),
            ),
        )
        .await;

        // Invalid JSON syntax
        let req = test::TestRequest::post()
            .uri("/")
            .set_json(&ValidatePropertyRequestPayload {
                property_json_str: r#"
                {
                  "a": 1,
                  "b": "2",
                  "c": [1, 2, 3,
                }
                "#
                .to_string(),
            })
            .to_request();

        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::OK);
        let response_body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(response_body["status"], "ok");
        assert_eq!(response_body["data"]["is_valid"], false);
        assert!(response_body["data"]["error_message"]
            .as_str()
            .unwrap()
            .contains("expected value"));
    }

    #[actix_web::test]
    async fn test_validate_property_invalid_additional_properties() {
        // Setup
        let designer_state = DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            tman_internal_config: Arc::new(TmanInternalConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
            graphs_cache: HashMap::new(),
        };
        let state = web::Data::new(Arc::new(RwLock::new(designer_state)));

        let app = test::init_service(
            App::new().app_data(state.clone()).service(
                web::resource("/")
                    .route(web::post().to(validate_property_endpoint)),
            ),
        )
        .await;

        // Invalid property with additional properties in a restricted object
        let req = test::TestRequest::post()
            .uri("/")
            .set_json(&ValidatePropertyRequestPayload {
                property_json_str: r#"
                {
                  "_ten": {
                    "predefined_graphs": [
                      {
                        "name": "default",
                        "nodes": [
                          {
                            "type": "extension",
                            "name": "default_extension_cpp",
                            "addon": "default_extension_cpp",
                            "extension_group": "default_extension_group",
                            "should_not_present": "aa"
                          }
                        ]
                      }
                    ]
                  }
                }
                "#
                .to_string(),
            })
            .to_request();

        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::OK);
        let response_body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(response_body["status"], "ok");
        assert_eq!(response_body["data"]["is_valid"], false);
        assert!(response_body["data"]["error_message"]
            .as_str()
            .unwrap()
            .contains("Additional properties are not allowed"));
    }

    #[actix_web::test]
    async fn test_validate_property_invalid_non_alphanumeric_field() {
        // Setup
        let designer_state = DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            tman_internal_config: Arc::new(TmanInternalConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
            graphs_cache: HashMap::new(),
        };
        let state = web::Data::new(Arc::new(RwLock::new(designer_state)));

        let app = test::init_service(
            App::new().app_data(state.clone()).service(
                web::resource("/")
                    .route(web::post().to(validate_property_endpoint)),
            ),
        )
        .await;

        // Invalid property with non-alphanumeric field.
        let req = test::TestRequest::post()
            .uri("/")
            .set_json(&ValidatePropertyRequestPayload {
                property_json_str: r#"
                {
                  "_ten": {
                    "predefined_graphs": [
                      {
                        "name": "default",
                        "nodes": [{
                          "type": "extension",
                          "name": "default_extension_cpp",
                          "addon": "default_extension_cpp",
                          "extension_group": "default_extension_group"
                        }],
                        "connections": [
                          {
                            "extension": "default_extension_cpp",
                            "cmd": [
                              {
                                "name": "invalid command",
                                "dest": [
                                  {
                                    "extension": "default_extension_cpp"
                                  }
                                ]
                              }
                            ]
                          }
                        ]
                      }
                    ]
                  }
                }
                "#
                .to_string(),
            })
            .to_request();

        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::OK);
        let response_body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(response_body["status"], "ok");
        assert_eq!(response_body["data"]["is_valid"], false);
        assert!(response_body["data"]["error_message"]
            .as_str()
            .unwrap()
            .contains("does not match"));
    }

    #[actix_web::test]
    async fn test_validate_property_invalid_missing_required_fields() {
        // Setup
        let designer_state = DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            tman_internal_config: Arc::new(TmanInternalConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
            graphs_cache: HashMap::new(),
        };
        let state = web::Data::new(Arc::new(RwLock::new(designer_state)));

        let app = test::init_service(
            App::new().app_data(state.clone()).service(
                web::resource("/")
                    .route(web::post().to(validate_property_endpoint)),
            ),
        )
        .await;

        // Invalid property with missing required fields.
        let req = test::TestRequest::post()
            .uri("/")
            .set_json(&ValidatePropertyRequestPayload {
                property_json_str: r#"
                {
                  "_ten": {
                    "predefined_graphs": [
                      {
                        "nodes": [
                          {
                            "type": "extension",
                            "addon": "default_extension_cpp",
                            "extension_group": "default_extension_group"
                          }
                        ]
                      }
                    ]
                  }
                }
                "#
                .to_string(),
            })
            .to_request();

        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::OK);
        let response_body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(response_body["status"], "ok");
        assert_eq!(response_body["data"]["is_valid"], false);
        assert!(response_body["data"]["error_message"]
            .as_str()
            .unwrap()
            .contains("required"));

        println!("error_message: {}", response_body["data"]["error_message"]);
    }
}
