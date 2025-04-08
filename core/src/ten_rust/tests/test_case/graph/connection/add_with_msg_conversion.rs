//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#[cfg(test)]
mod msg_conversion_tests {
    use std::collections::HashMap;
    use std::str::FromStr;

    use ten_rust::{
        base_dir_pkg_info::PkgsInfoInApp,
        graph::msg_conversion::{
            MsgAndResultConversion, MsgConversion, MsgConversionMode,
            MsgConversionRule, MsgConversionRules, MsgConversionType,
        },
        graph::{node::GraphNode, Graph},
        pkg_info::{
            manifest::Manifest, message::MsgType, pkg_type::PkgType,
            pkg_type_and_name::PkgTypeAndName, property::Property, PkgInfo,
        },
        schema::store::SchemaStore,
    };

    fn create_test_node(
        name: &str,
        addon: &str,
        app: Option<&str>,
    ) -> GraphNode {
        GraphNode {
            type_and_name: PkgTypeAndName {
                pkg_type: PkgType::Extension,
                name: name.to_string(),
            },
            addon: addon.to_string(),
            extension_group: None,
            app: app.map(|s| s.to_string()),
            property: None,
        }
    }

    fn create_test_pkg_info_map() -> HashMap<String, PkgsInfoInApp> {
        let mut map = HashMap::new();

        // Create app PkgInfo.
        let app_manifest_str = r#"
        {
            "type": "app",
            "name": "app1",
            "version": "1.0.0"
        }
        "#;
        let app_manifest = Manifest::from_str(app_manifest_str).unwrap();

        // Create property with URI for the app.
        let prop_str = r#"
        {
            "_ten": {
                "uri": "app1"
            }
        }
        "#;
        let app_property = Property::from_str(prop_str).unwrap();

        // Create ext1 PkgInfo with valid API schema for message communication.
        let ext1_manifest_json_str =
            include_str!("test_data_embed/ext_1_manifest.json");
        let ext1_manifest = Manifest::from_str(ext1_manifest_json_str).unwrap();

        // Create ext2 PkgInfo with compatible API schemas.
        let ext2_manifest_json_str =
            include_str!("test_data_embed/ext_2_manifest.json");
        let ext2_manifest = Manifest::from_str(ext2_manifest_json_str).unwrap();

        // Create ext3 PkgInfo with incompatible API schemas.
        let ext3_manifest_json_str =
            include_str!("test_data_embed/ext_3_manifest.json");
        let ext3_manifest = Manifest::from_str(ext3_manifest_json_str).unwrap();

        // Create app PkgInfo.
        let app_pkg_info = PkgInfo {
            manifest: Some(app_manifest),
            property: Some(app_property),
            compatible_score: 0,
            is_installed: true,
            url: String::new(),
            hash: String::new(),
            schema_store: None,
            is_local_dependency: false,
            local_dependency_path: None,
            local_dependency_base_dir: None,
        };

        // Create schema stores for extensions.
        let ext1_schema_store =
            SchemaStore::from_manifest(&ext1_manifest).unwrap().unwrap();
        let ext2_schema_store =
            SchemaStore::from_manifest(&ext2_manifest).unwrap().unwrap();
        let ext3_schema_store =
            SchemaStore::from_manifest(&ext3_manifest).unwrap().unwrap();

        // Create extension PkgInfos.
        let ext1_pkg_info = PkgInfo {
            manifest: Some(ext1_manifest),
            property: None,
            compatible_score: 0,
            is_installed: true,
            url: String::new(),
            hash: String::new(),
            schema_store: Some(ext1_schema_store),
            is_local_dependency: false,
            local_dependency_path: None,
            local_dependency_base_dir: None,
        };

        let ext2_pkg_info = PkgInfo {
            manifest: Some(ext2_manifest),
            property: None,
            compatible_score: 0,
            is_installed: true,
            url: String::new(),
            hash: String::new(),
            schema_store: Some(ext2_schema_store),
            is_local_dependency: false,
            local_dependency_path: None,
            local_dependency_base_dir: None,
        };

        let ext3_pkg_info = PkgInfo {
            manifest: Some(ext3_manifest),
            property: None,
            compatible_score: 0,
            is_installed: true,
            url: String::new(),
            hash: String::new(),
            schema_store: Some(ext3_schema_store),
            is_local_dependency: false,
            local_dependency_path: None,
            local_dependency_base_dir: None,
        };

        // Create a PkgsInfoInApp and add all packages
        let base_dir_pkg_info = PkgsInfoInApp {
            app_pkg_info: Some(app_pkg_info),
            extension_pkg_info: Some(vec![
                ext1_pkg_info,
                ext2_pkg_info,
                ext3_pkg_info,
            ]),
            protocol_pkg_info: None,
            addon_loader_pkg_info: None,
            system_pkg_info: None,
        };

        // Add to map with app URI as key
        map.insert("app1".to_string(), base_dir_pkg_info);

        map
    }

    #[test]
    fn test_add_connection_with_fixed_value_msg_conversion() {
        // Create a graph with two nodes.
        let mut graph = Graph {
            nodes: vec![
                create_test_node("ext1", "addon1", Some("app1")),
                create_test_node("ext2", "addon2", Some("app1")),
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
        let result = graph.add_connection(
            Some("app1".to_string()),
            "ext1".to_string(),
            MsgType::Cmd,
            "cmd1".to_string(),
            Some("app1".to_string()),
            "ext2".to_string(),
            &create_test_pkg_info_map(),
            Some(msg_conversion.clone()),
        );

        assert!(result.is_ok());
        assert!(graph.connections.is_some());

        // Verify connection and msg_conversion were added correctly
        let connections = graph.connections.as_ref().unwrap();
        assert_eq!(connections.len(), 1);

        let connection = &connections[0];
        assert_eq!(connection.app, Some("app1".to_string()));
        assert_eq!(connection.extension, "ext1");

        let cmd_flows = connection.cmd.as_ref().unwrap();
        assert_eq!(cmd_flows.len(), 1);

        let flow = &cmd_flows[0];
        assert_eq!(flow.name, "cmd1");
        assert_eq!(flow.dest.len(), 1);

        let dest = &flow.dest[0];
        assert_eq!(dest.app, Some("app1".to_string()));
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
        // Create a graph with two nodes.
        let mut graph = Graph {
            nodes: vec![
                create_test_node("ext1", "addon1", Some("app1")),
                create_test_node("ext2", "addon2", Some("app1")),
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
        let result = graph.add_connection(
            Some("app1".to_string()),
            "ext1".to_string(),
            MsgType::Cmd,
            "cmd1".to_string(),
            Some("app1".to_string()),
            "ext2".to_string(),
            &create_test_pkg_info_map(),
            Some(msg_conversion),
        );

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
        // Create a graph with two nodes.
        let mut graph = Graph {
            nodes: vec![
                create_test_node("ext1", "addon1", Some("app1")),
                create_test_node("ext2", "addon2", Some("app1")),
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
        let result = graph.add_connection(
            Some("app1".to_string()),
            "ext1".to_string(),
            MsgType::Cmd,
            "cmd1".to_string(),
            Some("app1".to_string()),
            "ext2".to_string(),
            &create_test_pkg_info_map(),
            Some(msg_conversion),
        );

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
        // Create a graph with two nodes.
        let mut graph = Graph {
            nodes: vec![
                create_test_node("ext1", "addon1", Some("app1")),
                create_test_node("ext2", "addon2", Some("app1")),
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
        let result = graph.add_connection(
            Some("app1".to_string()),
            "ext1".to_string(),
            MsgType::Cmd,
            "cmd1".to_string(),
            Some("app1".to_string()),
            "ext2".to_string(),
            &create_test_pkg_info_map(),
            Some(msg_conversion),
        );

        // Should fail validation due to empty rules.
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("conversion rules are empty"));
    }
}
