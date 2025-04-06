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

    use actix_web::{test, web, App};
    use ten_manager::{
        config::TmanConfig,
        designer::{
            graphs::connections::add::{
                add_graph_connection_endpoint,
                AddGraphConnectionRequestPayload,
                AddGraphConnectionResponsePayload,
            },
            mock::inject_all_pkgs_for_mock,
            response::ApiResponse,
            DesignerState,
        },
        output::TmanOutputCli,
    };
    use ten_rust::{
        graph::msg_conversion::{
            MsgAndResultConversion, MsgConversion, MsgConversionMode,
            MsgConversionRule, MsgConversionRules, MsgConversionType,
        },
        pkg_info::{constants::PROPERTY_JSON_FILENAME, message::MsgType},
    };

    #[actix_web::test]
    async fn test_add_graph_connection_with_msg_conversion() {
        let mut designer_state = DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
        };

        // Create a temporary directory for our test to store the generated
        // property.json.
        let temp_dir = tempfile::tempdir().unwrap();
        let test_dir = temp_dir.path().to_str().unwrap().to_string();

        // Load both the app package JSON and extension addon package JSONs.
        let app_manifest_json_str =
            include_str!("../test_data_embed/app_manifest.json").to_string();
        let app_property_json_str =
            include_str!("../test_data_embed/app_property.json").to_string();

        // Create the property.json file in the temporary directory.
        let property_path =
            std::path::Path::new(&test_dir).join(PROPERTY_JSON_FILENAME);
        std::fs::write(&property_path, &app_property_json_str).unwrap();

        // Create extension addon manifest strings.
        let ext1_manifest = r#"{
            "type": "extension",
            "name": "extension_1",
            "version": "0.1.0"
        }"#
        .to_string();

        let ext2_manifest = r#"{
            "type": "extension",
            "name": "extension_2",
            "version": "0.1.0"
        }"#
        .to_string();

        // The empty property for addons.
        let empty_property = r#"{"_ten":{}}"#.to_string();

        let all_pkgs_json = vec![
            (app_manifest_json_str, app_property_json_str),
            (ext1_manifest, empty_property.clone()),
            (ext2_manifest, empty_property.clone()),
        ];

        let inject_ret = inject_all_pkgs_for_mock(
            &test_dir,
            &mut designer_state.pkgs_cache,
            all_pkgs_json,
        );
        assert!(inject_ret.is_ok());

        let designer_state = Arc::new(RwLock::new(designer_state));

        let app = test::init_service(
            App::new().app_data(web::Data::new(designer_state)).route(
                "/api/designer/v1/graphs/connections/add",
                web::post().to(add_graph_connection_endpoint),
            ),
        )
        .await;

        // Create msg_conversion rules.
        let msg_conversion = MsgAndResultConversion {
            msg: MsgConversion {
                conversion_type: MsgConversionType::PerProperty,
                rules: MsgConversionRules {
                    rules: vec![
                        // Fixed value rule.
                        MsgConversionRule {
                            path: "target_property".to_string(),
                            conversion_mode: MsgConversionMode::FixedValue,
                            original_path: None,
                            value: Some(serde_json::json!(
                                "fixed_value_string"
                            )),
                        },
                        // From original rule.
                        MsgConversionRule {
                            path: "copied_property".to_string(),
                            conversion_mode: MsgConversionMode::FromOriginal,
                            original_path: Some("source_property".to_string()),
                            value: None,
                        },
                    ],
                    keep_original: Some(false),
                },
            },
            result: None,
        };

        // Add a connection between existing nodes in the default graph.
        // Use "http://example.com:8000" for both src_app and dest_app to match the test data.
        let request_payload = AddGraphConnectionRequestPayload {
            base_dir: test_dir.clone(),
            graph_name: "default_with_app_uri".to_string(),
            src_app: Some("http://example.com:8000".to_string()),
            src_extension: "extension_1".to_string(),
            msg_type: MsgType::Cmd,
            msg_name: "test_cmd".to_string(),
            dest_app: Some("http://example.com:8000".to_string()),
            dest_extension: "extension_2".to_string(),
            msg_conversion: Some(msg_conversion),
        };

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/graphs/connections/add")
            .set_json(request_payload)
            .to_request();
        let resp = test::call_service(&app, req).await;

        // Print the status and body for debugging.
        let status = resp.status();
        println!("Response status: {:?}", status);
        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();
        println!("Response body: {}", body_str);

        assert!(status.is_success());

        let response: ApiResponse<AddGraphConnectionResponsePayload> =
            serde_json::from_str(body_str).unwrap();

        assert!(response.data.success);

        // Define expected property.json content after adding all three
        // connections.
        let expected_property_json_str = include_str!("test_data_embed/expected_json__connection_with_msg_conversion.json");

        // Read the actual property.json file generated during the test.
        let property_path =
            std::path::Path::new(&test_dir).join(PROPERTY_JSON_FILENAME);
        let actual_property = std::fs::read_to_string(property_path).unwrap();

        // Normalize both JSON strings to handle formatting differences.
        let expected_value: serde_json::Value =
            serde_json::from_str(expected_property_json_str).unwrap();
        let actual_value: serde_json::Value =
            serde_json::from_str(&actual_property).unwrap();

        // Compare the normalized JSON values.
        assert_eq!(
            expected_value, actual_value,
            "Property file doesn't match expected content.\nExpected:\n{}\nActual:\n{}",
            serde_json::to_string_pretty(&expected_value).unwrap(),
            serde_json::to_string_pretty(&actual_value).unwrap()
        );
    }

    #[actix_web::test]
    async fn test_add_graph_connection_with_result_conversion() {
        let mut designer_state = DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
        };

        // Create a temporary directory for our test to store the generated
        // property.json.
        let temp_dir = tempfile::tempdir().unwrap();
        let test_dir = temp_dir.path().to_str().unwrap().to_string();

        // Load both the app package JSON and extension addon package JSONs.
        let app_manifest_json_str =
            include_str!("../test_data_embed/app_manifest.json").to_string();
        let app_property_json_str =
            include_str!("../test_data_embed/app_property.json").to_string();

        // Create the property.json file in the temporary directory.
        let property_path =
            std::path::Path::new(&test_dir).join(PROPERTY_JSON_FILENAME);
        std::fs::write(&property_path, &app_property_json_str).unwrap();

        // Create extension addon manifest strings.
        let ext1_manifest = r#"{
            "type": "extension",
            "name": "extension_1",
            "version": "0.1.0"
        }"#
        .to_string();

        let ext2_manifest = r#"{
            "type": "extension",
            "name": "extension_2",
            "version": "0.1.0"
        }"#
        .to_string();

        // The empty property for addons
        let empty_property = r#"{"_ten":{}}"#.to_string();

        let all_pkgs_json = vec![
            (app_manifest_json_str, app_property_json_str),
            (ext1_manifest, empty_property.clone()),
            (ext2_manifest, empty_property.clone()),
        ];

        let inject_ret = inject_all_pkgs_for_mock(
            &test_dir,
            &mut designer_state.pkgs_cache,
            all_pkgs_json,
        );
        assert!(inject_ret.is_ok());

        let designer_state = Arc::new(RwLock::new(designer_state));

        let app = test::init_service(
            App::new().app_data(web::Data::new(designer_state)).route(
                "/api/designer/v1/graphs/connections/add",
                web::post().to(add_graph_connection_endpoint),
            ),
        )
        .await;

        // Create msg_conversion rules with result conversion
        let msg_conversion = MsgAndResultConversion {
            msg: MsgConversion {
                conversion_type: MsgConversionType::PerProperty,
                rules: MsgConversionRules {
                    rules: vec![MsgConversionRule {
                        path: "request_property".to_string(),
                        conversion_mode: MsgConversionMode::FixedValue,
                        original_path: None,
                        value: Some(serde_json::json!(123)),
                    }],
                    keep_original: Some(true),
                },
            },
            result: Some(MsgConversion {
                conversion_type: MsgConversionType::PerProperty,
                rules: MsgConversionRules {
                    rules: vec![MsgConversionRule {
                        path: "response_property".to_string(),
                        conversion_mode: MsgConversionMode::FromOriginal,
                        original_path: Some("original_response".to_string()),
                        value: None,
                    }],
                    keep_original: Some(false),
                },
            }),
        };

        // Add a connection between existing nodes in the default graph.
        let request_payload = AddGraphConnectionRequestPayload {
            base_dir: test_dir.clone(),
            graph_name: "default_with_app_uri".to_string(),
            src_app: Some("http://example.com:8000".to_string()),
            src_extension: "extension_1".to_string(),
            msg_type: MsgType::Cmd,
            msg_name: "test_cmd_with_result".to_string(),
            dest_app: Some("http://example.com:8000".to_string()),
            dest_extension: "extension_2".to_string(),
            msg_conversion: Some(msg_conversion),
        };

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/graphs/connections/add")
            .set_json(request_payload)
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();
        let response: ApiResponse<AddGraphConnectionResponsePayload> =
            serde_json::from_str(body_str).unwrap();

        assert!(response.data.success);

        // Read the actual property.json file generated during the test.
        let property_path =
            std::path::Path::new(&test_dir).join(PROPERTY_JSON_FILENAME);
        let actual_property_str =
            std::fs::read_to_string(property_path).unwrap();
        let actual_property: serde_json::Value =
            serde_json::from_str(&actual_property_str).unwrap();

        // Validate the property.json contains our connection with both
        // msg_conversion and result
        if let Some(ten) = actual_property.get("_ten") {
            if let Some(predefined_graphs) = ten.get("predefined_graphs") {
                if let Some(default_graph) =
                    predefined_graphs.get("default_with_app_uri")
                {
                    if let Some(connections) = default_graph.get("connections")
                    {
                        let mut found_msg_conversion = false;
                        let mut found_result_conversion = false;

                        // Check all connections in the array
                        if let Some(connections_array) = connections.as_array()
                        {
                            for connection in connections_array {
                                if let Some(extension) =
                                    connection.get("extension")
                                {
                                    if extension == "extension_1" {
                                        if let Some(cmd) = connection.get("cmd")
                                        {
                                            if let Some(cmd_array) =
                                                cmd.as_array()
                                            {
                                                for cmd_item in cmd_array {
                                                    if let Some(name) =
                                                        cmd_item.get("name")
                                                    {
                                                        if name == "test_cmd_with_result" {
                                                            if let Some(dest_array) = cmd_item.get("dest").and_then(|d| d.as_array()) {
                                                                for dest in dest_array {
                                                                    if dest.get("extension") == Some(&serde_json::json!("extension_2")) {
                                                                        // Check for msg_conversion
                                                                        if let Some(msg_conversion) = dest.get("msg_conversion") {
                                                                            // Verify request conversion
                                                                            if msg_conversion.get("type") == Some(&serde_json::json!("per_property")) {
                                                                                if let Some(rules) = msg_conversion.get("rules") {
                                                                                    if let Some(rule) = rules.as_array().unwrap().first() {
                                                                                        if rule.get("path") == Some(&serde_json::json!("request_property")) {
                                                                                            found_msg_conversion = true;
                                                                                        }
                                                                                    }
                                                                                }

                                                                                // Verify keep_original
                                                                                assert_eq!(msg_conversion.get("keep_original"), Some(&serde_json::json!(true)));
                                                                            }

                                                                            // Verify result conversion
                                                                            if let Some(result) = msg_conversion.get("result") {
                                                                                if result.get("type") == Some(&serde_json::json!("per_property")) {
                                                                                    if let Some(rules) = result.get("rules") {
                                                                                        if let Some(rule) = rules.as_array().unwrap().first() {
                                                                                            if rule.get("path") == Some(&serde_json::json!("response_property")) {
                                                                                                found_result_conversion = true;
                                                                                            }
                                                                                        }
                                                                                    }

                                                                                    // Verify keep_original for result
                                                                                    assert_eq!(result.get("keep_original"), Some(&serde_json::json!(false)));
                                                                                }
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        assert!(
                            found_msg_conversion,
                            "Message conversion not found in property.json"
                        );
                        assert!(
                            found_result_conversion,
                            "Result conversion not found in property.json"
                        );
                    }
                }
            }
        }
    }
}
