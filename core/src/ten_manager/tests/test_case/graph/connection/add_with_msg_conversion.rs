//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#[cfg(test)]
mod tests {
    use anyhow::Result;
    use serde_json::Value;
    use tempfile::TempDir;

    use ten_manager::graph::update_graph_connections_all_fields;
    use ten_rust::graph::connection::{
        GraphConnection, GraphDestination, GraphMessageFlow,
    };
    use ten_rust::graph::msg_conversion::{
        MsgAndResultConversion, MsgConversion, MsgConversionMode,
        MsgConversionRule, MsgConversionRules, MsgConversionType,
    };
    use ten_rust::pkg_info::constants::PROPERTY_JSON_FILENAME;

    #[test]
    fn test_add_connection_with_msg_conversion() -> Result<()> {
        // Create a temporary directory for our test.
        let temp_dir = TempDir::new()?;
        let test_dir = temp_dir.path().to_str().unwrap().to_string();

        // First, create the initial property.json with a connection.
        let initial_json =
            include_str!("test_data_embed/initial_property.json");

        // Expected JSON after adding the connection with message conversion.
        let expected_json = include_str!(
            "test_data_embed/expected_property_with_msg_conversion.json"
        );

        // Write the initial JSON to property.json.
        let property_path =
            std::path::Path::new(&test_dir).join(PROPERTY_JSON_FILENAME);
        std::fs::write(&property_path, initial_json)?;

        // Parse the initial JSON to all_fields.
        let mut all_fields: serde_json::Map<String, Value> =
            serde_json::from_str(initial_json)?;

        // Create a connection with message conversion
        let connection = GraphConnection {
            app: Some("http://example.com:8000".to_string()),
            extension: "extension_1".to_string(),
            cmd: Some(vec![GraphMessageFlow {
                name: "cmd_with_conversion".to_string(),
                dest: vec![GraphDestination {
                    app: Some("http://example.com:8000".to_string()),
                    extension: "extension_2".to_string(),
                    msg_conversion: Some(MsgAndResultConversion {
                        msg: MsgConversion {
                            conversion_type: MsgConversionType::PerProperty,
                            rules: MsgConversionRules {
                                rules: vec![
                                    MsgConversionRule {
                                        path: "mapped_param".to_string(),
                                        conversion_mode:
                                            MsgConversionMode::FromOriginal,
                                        original_path: Some(
                                            "original_param".to_string(),
                                        ),
                                        value: None,
                                    },
                                    MsgConversionRule {
                                        path: "fixed_param".to_string(),
                                        conversion_mode:
                                            MsgConversionMode::FixedValue,
                                        original_path: None,
                                        value: Some(serde_json::json!(42)),
                                    },
                                ],
                                keep_original: Some(true),
                            },
                        },
                        result: Some(MsgConversion {
                            conversion_type: MsgConversionType::PerProperty,
                            rules: MsgConversionRules {
                                rules: vec![MsgConversionRule {
                                    path: "mapped_result".to_string(),
                                    conversion_mode:
                                        MsgConversionMode::FromOriginal,
                                    original_path: Some(
                                        "original_result".to_string(),
                                    ),
                                    value: None,
                                }],
                                keep_original: Some(false),
                            },
                        }),
                    }),
                }],
            }]),
            data: None,
            audio_frame: None,
            video_frame: None,
        };

        let connections_to_add = vec![connection];

        // Update the connections in memory and in the file.
        update_graph_connections_all_fields(
            &test_dir,
            &mut all_fields,
            "test_graph",
            Some(&connections_to_add),
            None,
            None,
        )?;

        // Read the updated property.json.
        let actual_json = std::fs::read_to_string(&property_path)?;

        // Normalize both JSON strings (parse and reformat to remove whitespace
        // differences).
        let expected_value: serde_json::Value =
            serde_json::from_str(expected_json)?;
        let actual_value: serde_json::Value =
            serde_json::from_str(&actual_json)?;

        assert_eq!(
            expected_value, actual_value,
            "Updated property does not match expected property"
        );

        Ok(())
    }
}
