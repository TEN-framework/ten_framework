//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use anyhow::Result;
    use serde_json::Value;
    use tempfile::TempDir;

    use ten_manager::constants::TEST_DIR;
    use ten_manager::graph::connections::add::graph_add_connection;
    use ten_manager::graph::update_graph_connections_all_fields;
    use ten_rust::graph::connection::{
        GraphConnection, GraphDestination, GraphMessageFlow,
    };
    use ten_rust::graph::msg_conversion::{
        MsgAndResultConversion, MsgConversion, MsgConversionMode,
        MsgConversionRule, MsgConversionRules, MsgConversionType,
    };
    use ten_rust::graph::Graph;
    use ten_rust::pkg_info::constants::PROPERTY_JSON_FILENAME;
    use ten_rust::pkg_info::create_uri_to_pkg_info_map;
    use ten_rust::pkg_info::message::MsgType;

    use crate::test_case::graph::connection::create_test_node;
    use crate::test_case::mock::inject_all_pkgs_for_mock;

    #[test]
    fn test_add_connection_with_msg_conversion() -> Result<()> {
        // Create a temporary directory for our test.
        let temp_dir = TempDir::new()?;
        let test_dir = temp_dir.path().to_str().unwrap().to_string();

        // First, create the initial property.json with a connection.
        let initial_json =
            include_str!("../../../test_data/initial_property.json");

        // Expected JSON after adding the connection with message conversion.
        let expected_json = include_str!(
            "../../../test_data/expected_property_with_msg_conversion.json"
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

    #[test]
    fn test_add_connection_with_fixed_value_msg_conversion() {
        let all_pkgs_json_str = vec![
            (
                TEST_DIR.to_string(),
                include_str!("../../../test_data/app_manifest.json")
                    .to_string(),
                include_str!("test_data_embed/app_property.json").to_string(),
            ),
            (
                format!("{}{}", TEST_DIR, "/ten_packages/extension/addon1"),
                include_str!("test_data_embed/ext_1_manifest.json").to_string(),
                "{}".to_string(),
            ),
            (
                format!("{}{}", TEST_DIR, "/ten_packages/extension/addon2"),
                include_str!("test_data_embed/ext_2_manifest.json").to_string(),
                "{}".to_string(),
            ),
            (
                format!("{}{}", TEST_DIR, "/ten_packages/extension/addon3"),
                include_str!("test_data_embed/ext_3_manifest.json").to_string(),
                "{}".to_string(),
            ),
        ];

        let mut pkgs_cache = HashMap::new();
        let mut graphs_cache = HashMap::new();

        let inject_ret = inject_all_pkgs_for_mock(
            &mut pkgs_cache,
            &mut graphs_cache,
            all_pkgs_json_str,
        );
        assert!(inject_ret.is_ok());

        let uri_to_pkg_info = create_uri_to_pkg_info_map(&pkgs_cache).unwrap();

        // Create a graph with two nodes.
        let mut graph = Graph {
            nodes: vec![
                create_test_node(
                    "ext1",
                    "addon1",
                    Some("http://localhost:8000"),
                ),
                create_test_node(
                    "ext2",
                    "addon2",
                    Some("http://localhost:8000"),
                ),
            ],
            connections: None,
        };

        // Create a message conversion with fixed value.
        let msg_conversion = MsgAndResultConversion {
            msg: MsgConversion {
                conversion_type: MsgConversionType::PerProperty,
                rules: MsgConversionRules {
                    rules: vec![
                        MsgConversionRule {
                            path: "_ten.name".to_string(),
                            conversion_mode: MsgConversionMode::FixedValue,
                            original_path: None,
                            value: Some(serde_json::Value::String(
                                "test_value".to_string(),
                            )),
                        },
                        MsgConversionRule {
                            path: "param1".to_string(),
                            conversion_mode: MsgConversionMode::FixedValue,
                            original_path: None,
                            value: Some(serde_json::Value::Number(
                                serde_json::Number::from(42),
                            )),
                        },
                    ],
                    keep_original: Some(true),
                },
            },
            result: None,
        };

        // Test adding a connection with msg_conversion.
        let result = graph_add_connection(
            &mut graph,
            &Some(TEST_DIR.to_string()),
            Some("http://localhost:8000".to_string()),
            "ext1".to_string(),
            MsgType::Cmd,
            "cmd1".to_string(),
            Some("http://localhost:8000".to_string()),
            "ext2".to_string(),
            &uri_to_pkg_info,
            &pkgs_cache,
            Some(msg_conversion.clone()),
        );

        assert!(result.is_ok());
        assert!(graph.connections.is_some());

        // Verify connection and msg_conversion were added correctly
        let connections = graph.connections.as_ref().unwrap();
        assert_eq!(connections.len(), 1);

        let connection = &connections[0];
        assert_eq!(connection.app, Some("http://localhost:8000".to_string()));
        assert_eq!(connection.extension, "ext1");

        let cmd_flows = connection.cmd.as_ref().unwrap();
        assert_eq!(cmd_flows.len(), 1);

        let flow = &cmd_flows[0];
        assert_eq!(flow.name, "cmd1");
        assert_eq!(flow.dest.len(), 1);

        let dest = &flow.dest[0];
        assert_eq!(dest.app, Some("http://localhost:8000".to_string()));
        assert_eq!(dest.extension, "ext2");

        // Verify the msg_conversion was properly set
        assert!(dest.msg_conversion.is_some());
        let dest_msg_conversion = dest.msg_conversion.as_ref().unwrap();

        // Check conversion type.
        assert_eq!(
            dest_msg_conversion.msg.conversion_type,
            MsgConversionType::PerProperty
        );

        // Check rules.
        let rules = &dest_msg_conversion.msg.rules;
        assert_eq!(rules.keep_original, Some(true));
        assert_eq!(rules.rules.len(), 2);

        // Check first rule.
        let rule1 = &rules.rules[0];
        assert_eq!(rule1.path, "_ten.name");
        assert_eq!(rule1.conversion_mode, MsgConversionMode::FixedValue);
        assert_eq!(
            rule1.value.as_ref().unwrap().as_str().unwrap(),
            "test_value"
        );

        // Check second rule
        let rule2 = &rules.rules[1];
        assert_eq!(rule2.path, "param1");
        assert_eq!(rule2.conversion_mode, MsgConversionMode::FixedValue);
        assert_eq!(rule2.value.as_ref().unwrap().as_i64().unwrap(), 42);
    }

    #[test]
    fn test_add_connection_with_from_original_msg_conversion() {
        let all_pkgs_json_str = vec![
            (
                TEST_DIR.to_string(),
                include_str!("../../../test_data/app_manifest.json")
                    .to_string(),
                include_str!("test_data_embed/app_property.json").to_string(),
            ),
            (
                format!("{}{}", TEST_DIR, "/ten_packages/extension/addon1"),
                include_str!("test_data_embed/ext_1_manifest.json").to_string(),
                "{}".to_string(),
            ),
            (
                format!("{}{}", TEST_DIR, "/ten_packages/extension/addon2"),
                include_str!("test_data_embed/ext_2_manifest.json").to_string(),
                "{}".to_string(),
            ),
            (
                format!("{}{}", TEST_DIR, "/ten_packages/extension/addon3"),
                include_str!("test_data_embed/ext_3_manifest.json").to_string(),
                "{}".to_string(),
            ),
        ];

        let mut pkgs_cache = HashMap::new();
        let mut graphs_cache = HashMap::new();

        let inject_ret = inject_all_pkgs_for_mock(
            &mut pkgs_cache,
            &mut graphs_cache,
            all_pkgs_json_str,
        );
        assert!(inject_ret.is_ok());

        let uri_to_pkg_info = create_uri_to_pkg_info_map(&pkgs_cache).unwrap();

        // Create a graph with two nodes.
        let mut graph = Graph {
            nodes: vec![
                create_test_node(
                    "ext1",
                    "addon1",
                    Some("http://localhost:8000"),
                ),
                create_test_node(
                    "ext2",
                    "addon2",
                    Some("http://localhost:8000"),
                ),
            ],
            connections: None,
        };

        // Create a message conversion with from_original mode.
        let msg_conversion = MsgAndResultConversion {
            msg: MsgConversion {
                conversion_type: MsgConversionType::PerProperty,
                rules: MsgConversionRules {
                    rules: vec![MsgConversionRule {
                        path: "new_param".to_string(),
                        conversion_mode: MsgConversionMode::FromOriginal,
                        original_path: Some("param1".to_string()),
                        value: None,
                    }],
                    keep_original: Some(false),
                },
            },
            result: None,
        };

        // Test adding a connection with msg_conversion.
        let result = graph_add_connection(
            &mut graph,
            &Some(TEST_DIR.to_string()),
            Some("http://localhost:8000".to_string()),
            "ext1".to_string(),
            MsgType::Cmd,
            "cmd1".to_string(),
            Some("http://localhost:8000".to_string()),
            "ext2".to_string(),
            &uri_to_pkg_info,
            &pkgs_cache,
            Some(msg_conversion),
        );

        println!("result: {:?}", result);

        assert!(result.is_ok());
        assert!(graph.connections.is_some());

        // Verify connection and msg_conversion were added correctly.
        let connections = graph.connections.as_ref().unwrap();
        let connection = &connections[0];
        let cmd_flows = connection.cmd.as_ref().unwrap();
        let flow = &cmd_flows[0];
        let dest = &flow.dest[0];

        // Verify the msg_conversion was properly set.
        assert!(dest.msg_conversion.is_some());
        let dest_msg_conversion = dest.msg_conversion.as_ref().unwrap();

        // Check rules.
        let rules = &dest_msg_conversion.msg.rules;
        assert_eq!(rules.keep_original, Some(false));
        assert_eq!(rules.rules.len(), 1);

        // Check the rule.
        let rule = &rules.rules[0];
        assert_eq!(rule.path, "new_param");
        assert_eq!(rule.conversion_mode, MsgConversionMode::FromOriginal);
        assert_eq!(rule.original_path.as_ref().unwrap(), "param1");
        assert!(rule.value.is_none());
    }

    #[test]
    fn test_add_connection_with_msg_and_result_conversion() {
        let all_pkgs_json_str = vec![
            (
                TEST_DIR.to_string(),
                include_str!("../../../test_data/app_manifest.json")
                    .to_string(),
                include_str!("test_data_embed/app_property.json").to_string(),
            ),
            (
                format!("{}{}", TEST_DIR, "/ten_packages/extension/addon1"),
                include_str!("test_data_embed/ext_1_manifest.json").to_string(),
                "{}".to_string(),
            ),
            (
                format!("{}{}", TEST_DIR, "/ten_packages/extension/addon2"),
                include_str!("test_data_embed/ext_2_manifest.json").to_string(),
                "{}".to_string(),
            ),
            (
                format!("{}{}", TEST_DIR, "/ten_packages/extension/addon3"),
                include_str!("test_data_embed/ext_3_manifest.json").to_string(),
                "{}".to_string(),
            ),
        ];

        let mut pkgs_cache = HashMap::new();
        let mut graphs_cache = HashMap::new();

        let inject_ret = inject_all_pkgs_for_mock(
            &mut pkgs_cache,
            &mut graphs_cache,
            all_pkgs_json_str,
        );
        assert!(inject_ret.is_ok());

        let uri_to_pkg_info = create_uri_to_pkg_info_map(&pkgs_cache).unwrap();

        // Create a graph with two nodes.
        let mut graph = Graph {
            nodes: vec![
                create_test_node(
                    "ext1",
                    "addon1",
                    Some("http://localhost:8000"),
                ),
                create_test_node(
                    "ext2",
                    "addon2",
                    Some("http://localhost:8000"),
                ),
            ],
            connections: None,
        };

        // Create a message conversion with both message and result conversion.
        let msg_conversion = MsgAndResultConversion {
            msg: MsgConversion {
                conversion_type: MsgConversionType::PerProperty,
                rules: MsgConversionRules {
                    rules: vec![MsgConversionRule {
                        path: "mapped_param".to_string(),
                        conversion_mode: MsgConversionMode::FromOriginal,
                        original_path: Some("param1".to_string()),
                        value: None,
                    }],
                    keep_original: Some(true),
                },
            },
            result: Some(MsgConversion {
                conversion_type: MsgConversionType::PerProperty,
                rules: MsgConversionRules {
                    rules: vec![MsgConversionRule {
                        path: "mapped_detail".to_string(),
                        conversion_mode: MsgConversionMode::FromOriginal,
                        original_path: Some("detail".to_string()),
                        value: None,
                    }],
                    keep_original: Some(false),
                },
            }),
        };

        // Test adding a connection with msg_conversion.
        let result = graph_add_connection(
            &mut graph,
            &Some(TEST_DIR.to_string()),
            Some("http://localhost:8000".to_string()),
            "ext1".to_string(),
            MsgType::Cmd,
            "cmd1".to_string(),
            Some("http://localhost:8000".to_string()),
            "ext2".to_string(),
            &uri_to_pkg_info,
            &pkgs_cache,
            Some(msg_conversion),
        );
        eprintln!("result: {:?}", result);

        assert!(result.is_ok());

        // Verify connection and msg_conversion were added correctly.
        let dest =
            &graph.connections.as_ref().unwrap()[0].cmd.as_ref().unwrap()[0]
                .dest[0];

        // Verify the msg_conversion was properly set.
        let dest_msg_conversion = dest.msg_conversion.as_ref().unwrap();

        // Verify both message and result conversion exist.
        assert!(dest_msg_conversion.result.is_some());

        // Check message conversion.
        assert_eq!(dest_msg_conversion.msg.rules.rules[0].path, "mapped_param");

        // Check result conversion.
        let result_conversion = dest_msg_conversion.result.as_ref().unwrap();
        assert_eq!(
            result_conversion.conversion_type,
            MsgConversionType::PerProperty
        );
        assert_eq!(result_conversion.rules.rules[0].path, "mapped_detail");
        assert_eq!(
            result_conversion.rules.rules[0]
                .original_path
                .as_ref()
                .unwrap(),
            "detail"
        );
    }

    #[test]
    fn test_add_connection_with_invalid_msg_conversion() {
        let all_pkgs_json_str = vec![
            (
                TEST_DIR.to_string(),
                include_str!("../../../test_data/app_manifest.json")
                    .to_string(),
                include_str!("test_data_embed/app_property.json").to_string(),
            ),
            (
                format!("{}{}", TEST_DIR, "/ten_packages/extension/addon1"),
                include_str!("test_data_embed/ext_1_manifest.json").to_string(),
                "{}".to_string(),
            ),
            (
                format!("{}{}", TEST_DIR, "/ten_packages/extension/addon2"),
                include_str!("test_data_embed/ext_2_manifest.json").to_string(),
                "{}".to_string(),
            ),
            (
                format!("{}{}", TEST_DIR, "/ten_packages/extension/addon3"),
                include_str!("test_data_embed/ext_3_manifest.json").to_string(),
                "{}".to_string(),
            ),
        ];

        let mut pkgs_cache = HashMap::new();
        let mut graphs_cache = HashMap::new();

        let inject_ret = inject_all_pkgs_for_mock(
            &mut pkgs_cache,
            &mut graphs_cache,
            all_pkgs_json_str,
        );
        assert!(inject_ret.is_ok());

        let uri_to_pkg_info = create_uri_to_pkg_info_map(&pkgs_cache).unwrap();

        // Create a graph with two nodes.
        let mut graph = Graph {
            nodes: vec![
                create_test_node(
                    "ext1",
                    "addon1",
                    Some("http://localhost:8000"),
                ),
                create_test_node(
                    "ext2",
                    "addon2",
                    Some("http://localhost:8000"),
                ),
            ],
            connections: None,
        };

        // Create an invalid message conversion with empty rules.
        let msg_conversion = MsgAndResultConversion {
            msg: MsgConversion {
                conversion_type: MsgConversionType::PerProperty,
                rules: MsgConversionRules {
                    rules: vec![],
                    keep_original: None,
                },
            },
            result: None,
        };

        // Test adding a connection with invalid msg_conversion.
        let result = graph_add_connection(
            &mut graph,
            &Some(TEST_DIR.to_string()),
            Some("http://localhost:8000".to_string()),
            "ext1".to_string(),
            MsgType::Cmd,
            "cmd1".to_string(),
            Some("http://localhost:8000".to_string()),
            "ext2".to_string(),
            &uri_to_pkg_info,
            &pkgs_cache,
            Some(msg_conversion),
        );

        // Should fail validation due to empty rules.
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("conversion rules are empty"));
    }

    #[test]
    fn test_add_connection_with_invalid_forward_conversion_1() {
        let all_pkgs_json_str = vec![
            (
                TEST_DIR.to_string(),
                include_str!("../../../test_data/app_manifest.json")
                    .to_string(),
                include_str!("test_data_embed/app_property.json").to_string(),
            ),
            (
                format!("{}{}", TEST_DIR, "/ten_packages/extension/addon1"),
                include_str!("test_data_embed/ext_1_manifest.json").to_string(),
                "{}".to_string(),
            ),
            (
                format!("{}{}", TEST_DIR, "/ten_packages/extension/addon2"),
                include_str!("test_data_embed/ext_2_manifest.json").to_string(),
                "{}".to_string(),
            ),
            (
                format!("{}{}", TEST_DIR, "/ten_packages/extension/addon3"),
                include_str!("test_data_embed/ext_3_manifest.json").to_string(),
                "{}".to_string(),
            ),
            (
                format!("{}{}", TEST_DIR, "/ten_packages/extension/addon4"),
                include_str!("test_data_embed/ext_4_manifest.json").to_string(),
                "{}".to_string(),
            ),
        ];

        let mut pkgs_cache = HashMap::new();
        let mut graphs_cache = HashMap::new();

        let inject_ret = inject_all_pkgs_for_mock(
            &mut pkgs_cache,
            &mut graphs_cache,
            all_pkgs_json_str,
        );
        assert!(inject_ret.is_ok());

        let uri_to_pkg_info = create_uri_to_pkg_info_map(&pkgs_cache).unwrap();

        // Create a graph with two nodes.
        let mut graph = Graph {
            nodes: vec![
                create_test_node(
                    "ext1",
                    "addon1",
                    Some("http://localhost:8000"),
                ),
                create_test_node(
                    "ext2",
                    "addon2",
                    Some("http://localhost:8000"),
                ),
                create_test_node(
                    "ext3",
                    "addon3",
                    Some("http://localhost:8000"),
                ),
                create_test_node(
                    "ext4",
                    "addon4",
                    Some("http://localhost:8000"),
                ),
            ],
            connections: None,
        };

        let msg_conversion = MsgAndResultConversion {
            msg: MsgConversion {
                conversion_type: MsgConversionType::PerProperty,
                rules: MsgConversionRules {
                    rules: vec![MsgConversionRule {
                        path: "param1".to_string(),
                        conversion_mode: MsgConversionMode::FromOriginal,
                        original_path: Some("param2".to_string()),
                        value: None,
                    }],
                    keep_original: None,
                },
            },
            result: Some(MsgConversion {
                conversion_type: MsgConversionType::PerProperty,
                rules: MsgConversionRules {
                    rules: vec![MsgConversionRule {
                        path: "mapped_detail".to_string(),
                        conversion_mode: MsgConversionMode::FromOriginal,
                        original_path: Some("detail".to_string()),
                        value: None,
                    }],
                    keep_original: Some(false),
                },
            }),
        };

        // Test adding a connection with invalid msg_conversion.
        let result = graph_add_connection(
            &mut graph,
            &Some(TEST_DIR.to_string()),
            Some("http://localhost:8000".to_string()),
            "ext1".to_string(),
            MsgType::Cmd,
            "cmd1".to_string(),
            Some("http://localhost:8000".to_string()),
            "ext2".to_string(),
            &uri_to_pkg_info,
            &pkgs_cache,
            Some(msg_conversion),
        );

        println!("result: {:?}", result);

        // Should fail validation due to empty rules.
        assert!(result.is_err());
    }

    #[test]
    fn test_add_connection_with_result_conversion_success() {
        let all_pkgs_json_str = vec![
            (
                TEST_DIR.to_string(),
                include_str!("../../../test_data/app_manifest.json")
                    .to_string(),
                include_str!("test_data_embed/app_property.json").to_string(),
            ),
            (
                format!("{}{}", TEST_DIR, "/ten_packages/extension/addon1"),
                include_str!("test_data_embed/ext_1_manifest.json").to_string(),
                "{}".to_string(),
            ),
            (
                format!("{}{}", TEST_DIR, "/ten_packages/extension/addon2"),
                include_str!("test_data_embed/ext_2_manifest.json").to_string(),
                "{}".to_string(),
            ),
            (
                format!("{}{}", TEST_DIR, "/ten_packages/extension/addon3"),
                include_str!("test_data_embed/ext_3_manifest.json").to_string(),
                "{}".to_string(),
            ),
            (
                format!("{}{}", TEST_DIR, "/ten_packages/extension/addon4"),
                include_str!("test_data_embed/ext_4_manifest.json").to_string(),
                "{}".to_string(),
            ),
        ];

        let mut pkgs_cache = HashMap::new();
        let mut graphs_cache = HashMap::new();

        let inject_ret = inject_all_pkgs_for_mock(
            &mut pkgs_cache,
            &mut graphs_cache,
            all_pkgs_json_str,
        );
        assert!(inject_ret.is_ok());

        let uri_to_pkg_info = create_uri_to_pkg_info_map(&pkgs_cache).unwrap();

        // Create a graph with two nodes.
        let mut graph = Graph {
            nodes: vec![
                create_test_node(
                    "ext1",
                    "addon1",
                    Some("http://localhost:8000"),
                ),
                create_test_node(
                    "ext2",
                    "addon2",
                    Some("http://localhost:8000"),
                ),
                create_test_node(
                    "ext3",
                    "addon3",
                    Some("http://localhost:8000"),
                ),
                create_test_node(
                    "ext4",
                    "addon4",
                    Some("http://localhost:8000"),
                ),
            ],
            connections: None,
        };

        let msg_conversion = MsgAndResultConversion {
            msg: MsgConversion {
                conversion_type: MsgConversionType::PerProperty,
                rules: MsgConversionRules {
                    rules: vec![MsgConversionRule {
                        path: "param1".to_string(),
                        conversion_mode: MsgConversionMode::FromOriginal,
                        original_path: Some("param1".to_string()),
                        value: None,
                    }],
                    keep_original: None,
                },
            },
            result: Some(MsgConversion {
                conversion_type: MsgConversionType::PerProperty,
                rules: MsgConversionRules {
                    rules: vec![MsgConversionRule {
                        path: "mapped_detail".to_string(),
                        conversion_mode: MsgConversionMode::FromOriginal,
                        original_path: Some("detail".to_string()),
                        value: None,
                    }],
                    keep_original: Some(false),
                },
            }),
        };

        // Test adding a connection with invalid msg_conversion.
        let result = graph_add_connection(
            &mut graph,
            &Some(TEST_DIR.to_string()),
            Some("http://localhost:8000".to_string()),
            "ext1".to_string(),
            MsgType::Cmd,
            "cmd1".to_string(),
            Some("http://localhost:8000".to_string()),
            "ext2".to_string(),
            &uri_to_pkg_info,
            &pkgs_cache,
            Some(msg_conversion),
        );

        println!("result: {:?}", result);

        assert!(result.is_ok());
    }

    #[test]
    fn test_add_connection_with_result_conversion_failure() {
        let all_pkgs_json_str = vec![
            (
                TEST_DIR.to_string(),
                include_str!("../../../test_data/app_manifest.json")
                    .to_string(),
                include_str!("test_data_embed/app_property.json").to_string(),
            ),
            (
                format!("{}{}", TEST_DIR, "/ten_packages/extension/addon1"),
                include_str!("test_data_embed/ext_1_manifest.json").to_string(),
                "{}".to_string(),
            ),
            (
                format!("{}{}", TEST_DIR, "/ten_packages/extension/addon2"),
                include_str!("test_data_embed/ext_2_manifest.json").to_string(),
                "{}".to_string(),
            ),
            (
                format!("{}{}", TEST_DIR, "/ten_packages/extension/addon3"),
                include_str!("test_data_embed/ext_3_manifest.json").to_string(),
                "{}".to_string(),
            ),
            (
                format!("{}{}", TEST_DIR, "/ten_packages/extension/addon4"),
                include_str!("test_data_embed/ext_4_manifest.json").to_string(),
                "{}".to_string(),
            ),
        ];

        let mut pkgs_cache = HashMap::new();
        let mut graphs_cache = HashMap::new();

        let inject_ret = inject_all_pkgs_for_mock(
            &mut pkgs_cache,
            &mut graphs_cache,
            all_pkgs_json_str,
        );
        assert!(inject_ret.is_ok());

        let uri_to_pkg_info = create_uri_to_pkg_info_map(&pkgs_cache).unwrap();

        // Create a graph with two nodes.
        let mut graph = Graph {
            nodes: vec![
                create_test_node(
                    "ext1",
                    "addon1",
                    Some("http://localhost:8000"),
                ),
                create_test_node(
                    "ext2",
                    "addon2",
                    Some("http://localhost:8000"),
                ),
                create_test_node(
                    "ext3",
                    "addon3",
                    Some("http://localhost:8000"),
                ),
                create_test_node(
                    "ext4",
                    "addon4",
                    Some("http://localhost:8000"),
                ),
            ],
            connections: None,
        };

        let msg_conversion = MsgAndResultConversion {
            msg: MsgConversion {
                conversion_type: MsgConversionType::PerProperty,
                rules: MsgConversionRules {
                    rules: vec![MsgConversionRule {
                        path: "param1".to_string(),
                        conversion_mode: MsgConversionMode::FromOriginal,
                        original_path: Some("param1".to_string()),
                        value: None,
                    }],
                    keep_original: None,
                },
            },
            result: Some(MsgConversion {
                conversion_type: MsgConversionType::PerProperty,
                rules: MsgConversionRules {
                    rules: vec![MsgConversionRule {
                        path: "mapped_detail".to_string(),
                        conversion_mode: MsgConversionMode::FromOriginal,
                        original_path: Some("string_detail".to_string()),
                        value: None,
                    }],
                    keep_original: Some(false),
                },
            }),
        };

        // Test adding a connection with invalid msg_conversion.
        let result = graph_add_connection(
            &mut graph,
            &Some(TEST_DIR.to_string()),
            Some("http://localhost:8000".to_string()),
            "ext1".to_string(),
            MsgType::Cmd,
            "cmd1".to_string(),
            Some("http://localhost:8000".to_string()),
            "ext2".to_string(),
            &uri_to_pkg_info,
            &pkgs_cache,
            Some(msg_conversion),
        );

        println!("result: {:?}", result);

        assert!(result.is_err());
    }

    #[test]
    fn test_add_connection_with_msg_conversion_without_result_conversion_failure(
    ) {
        let all_pkgs_json_str = vec![
            (
                TEST_DIR.to_string(),
                include_str!("../../../test_data/app_manifest.json")
                    .to_string(),
                include_str!("test_data_embed/app_property.json").to_string(),
            ),
            (
                format!("{}{}", TEST_DIR, "/ten_packages/extension/addon1"),
                include_str!("test_data_embed/ext_1_manifest.json").to_string(),
                "{}".to_string(),
            ),
            (
                format!("{}{}", TEST_DIR, "/ten_packages/extension/addon2"),
                include_str!("test_data_embed/ext_2_manifest.json").to_string(),
                "{}".to_string(),
            ),
            (
                format!("{}{}", TEST_DIR, "/ten_packages/extension/addon3"),
                include_str!("test_data_embed/ext_3_manifest.json").to_string(),
                "{}".to_string(),
            ),
            (
                format!("{}{}", TEST_DIR, "/ten_packages/extension/addon4"),
                include_str!("test_data_embed/ext_4_manifest.json").to_string(),
                "{}".to_string(),
            ),
        ];

        let mut pkgs_cache = HashMap::new();
        let mut graphs_cache = HashMap::new();

        let inject_ret = inject_all_pkgs_for_mock(
            &mut pkgs_cache,
            &mut graphs_cache,
            all_pkgs_json_str,
        );
        assert!(inject_ret.is_ok());

        let uri_to_pkg_info = create_uri_to_pkg_info_map(&pkgs_cache).unwrap();

        // Create a graph with two nodes.
        let mut graph = Graph {
            nodes: vec![
                create_test_node(
                    "ext1",
                    "addon1",
                    Some("http://localhost:8000"),
                ),
                create_test_node(
                    "ext2",
                    "addon2",
                    Some("http://localhost:8000"),
                ),
                create_test_node(
                    "ext3",
                    "addon3",
                    Some("http://localhost:8000"),
                ),
                create_test_node(
                    "ext4",
                    "addon4",
                    Some("http://localhost:8000"),
                ),
            ],
            connections: None,
        };

        let msg_conversion = MsgAndResultConversion {
            msg: MsgConversion {
                conversion_type: MsgConversionType::PerProperty,
                rules: MsgConversionRules {
                    rules: vec![MsgConversionRule {
                        path: "param1".to_string(),
                        conversion_mode: MsgConversionMode::FromOriginal,
                        original_path: Some("param1".to_string()),
                        value: None,
                    }],
                    keep_original: None,
                },
            },
            result: None,
        };

        // Test adding a connection with invalid msg_conversion.
        let result = graph_add_connection(
            &mut graph,
            &Some(TEST_DIR.to_string()),
            Some("http://localhost:8000".to_string()),
            "ext1".to_string(),
            MsgType::Cmd,
            "cmd2".to_string(),
            Some("http://localhost:8000".to_string()),
            "ext2".to_string(),
            &uri_to_pkg_info,
            &pkgs_cache,
            Some(msg_conversion),
        );

        println!("result: {:?}", result);

        assert!(result.is_err());
    }
}
