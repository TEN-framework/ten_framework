//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
mod predefined_graph;

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        fs::{self},
    };

    use serde_json::Map;
    use tempfile::tempdir;
    use ten_rust::{
        graph::msg_conversion::{MsgConversionMode, MsgConversionType},
        pkg_info::{
            constants::TEN_FIELD_IN_PROPERTY,
            localhost,
            property::{Property, TenInProperty},
        },
    };

    #[test]
    fn test_parse_property_with_known_fields() {
        let json_data = r#"
        {
            "_ten": {
                "predefined_graphs": [],
                "uri": "http://example.com"
            }
        }
        "#;

        let property: Property = json_data.parse().unwrap();

        assert!(property._ten.is_some());
        let ten_in_property = property._ten.unwrap();

        assert_eq!(ten_in_property.uri.unwrap(), "http://example.com");
        assert!(ten_in_property.predefined_graphs.is_some());
        assert!(ten_in_property.additional_fields.is_empty());
        assert_eq!(property.all_fields.len(), 1); // Should contain _ten field.
        assert!(property.all_fields.contains_key("_ten"));
    }

    #[test]
    fn test_parse_property_with_additional_fields() {
        let json_data = r#"
        {
            "_ten": {
                "predefined_graphs": [],
                "uri": "http://example.com",
                "extra_field_1": "value1"
            },
            "global_field_1": "global_value1"
        }
        "#;

        let property: Property = json_data.parse().unwrap();

        assert!(property._ten.is_some());
        let ten_in_property = property._ten.unwrap();
        assert_eq!(ten_in_property.uri.unwrap(), "http://example.com");
        assert!(ten_in_property.predefined_graphs.is_some());
        assert_eq!(
            ten_in_property
                .additional_fields
                .get("extra_field_1")
                .unwrap(),
            "value1"
        );

        // Should contain _ten and global_field_1.
        assert_eq!(property.all_fields.len(), 2);
        assert_eq!(
            property.all_fields.get("global_field_1").unwrap(),
            "global_value1"
        );
    }

    #[test]
    fn test_dump_property_to_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("property.json");

        let ten_in_property = TenInProperty {
            predefined_graphs: Some(vec![]),
            uri: Some(String::from("http://example.com")),
            additional_fields: HashMap::new(),
        };

        // Create a property with all_fields containing _ten data.
        let mut all_fields = Map::new();
        all_fields.insert(
            TEN_FIELD_IN_PROPERTY.to_string(),
            serde_json::to_value(&ten_in_property).unwrap(),
        );

        let property = Property {
            _ten: Some(ten_in_property),
            all_fields,
        };

        property.dump_property_to_file(&file_path).unwrap();

        let saved_content = fs::read_to_string(file_path).unwrap();
        let expected_content = r#"
        {
            "_ten": {
                "predefined_graphs": [],
                "uri": "http://example.com"
            }
        }
        "#
        .trim();

        let saved_json: serde_json::Value =
            serde_json::from_str(&saved_content).unwrap();
        let expected_json: serde_json::Value =
            serde_json::from_str(expected_content).unwrap();

        assert_eq!(saved_json, expected_json);
    }

    #[test]
    fn test_parse_and_dump_property_with_additional_fields() {
        let json_data = r#"
        {
            "_ten": {
                "predefined_graphs": [],
                "uri": "http://example.com",
                "extra_field_1": "value1"
            },
            "global_field_1": "global_value1"
        }
        "#;

        let property: Property = json_data.parse().unwrap();

        let dir = tempdir().unwrap();
        let file_path = dir.path().join("property.json");

        property.dump_property_to_file(&file_path).unwrap();

        let saved_content = fs::read_to_string(file_path).unwrap();
        let saved_json: serde_json::Value =
            serde_json::from_str(&saved_content).unwrap();
        let original_json: serde_json::Value =
            serde_json::from_str(json_data).unwrap();

        assert_eq!(saved_json, original_json);
    }

    #[test]
    fn test_dump_property_without_localhost_app_in_graph() {
        let json_data = include_str!("test_data_embed/property.json");
        let property: Property = json_data.parse().unwrap();
        assert!(property._ten.is_some());
        let ten = property._ten.as_ref().unwrap();
        let predefined_graphs = ten.predefined_graphs.as_ref().unwrap();
        let nodes = &predefined_graphs.first().as_ref().unwrap().graph.nodes;
        let node = nodes.first().unwrap();
        assert_eq!(node.get_app_uri(), None);

        let dir = tempdir().unwrap();
        let file_path = dir.path().join("property.json");
        property.dump_property_to_file(&file_path).unwrap();

        let saved_content = fs::read_to_string(file_path).unwrap();
        eprintln!("{}", saved_content);
        assert_eq!(saved_content.find(localhost()), None);
    }

    #[test]
    fn test_dump_property_with_msg_conversion() {
        let prop_str = include_str!(
            "test_data_embed/dump_property_with_msg_conversion.json"
        );
        let property: Property = prop_str.parse().unwrap();
        assert!(property._ten.is_some());

        let ten = property._ten.as_ref().unwrap();
        let predefined_graphs = ten.predefined_graphs.as_ref().unwrap();
        let connections = &predefined_graphs
            .first()
            .as_ref()
            .unwrap()
            .graph
            .connections
            .as_ref()
            .unwrap();
        let connection = connections.first().unwrap();
        let cmd = connection.cmd.as_ref().unwrap();
        assert_eq!(cmd.len(), 1);

        let cmd_dest = &cmd.first().unwrap().dest;
        assert_eq!(cmd_dest.len(), 1);

        let msg_conversion =
            cmd_dest.first().unwrap().msg_conversion.as_ref().unwrap();
        assert_eq!(
            msg_conversion.msg.conversion_type,
            MsgConversionType::PerProperty
        );

        let rules = &msg_conversion.msg.rules;
        assert!(rules.keep_original.is_none());
        assert_eq!(rules.rules.len(), 2);

        let rule = rules.rules.first().unwrap();
        assert_eq!(rule.conversion_mode, MsgConversionMode::FixedValue);
        assert!(rule.original_path.is_none());
        assert_eq!(rule.value.as_ref().unwrap(), "hello_mapping");

        let dir = tempdir().unwrap();
        let file_path = dir.path().join("property.json");

        property.dump_property_to_file(&file_path).unwrap();

        let saved_content = fs::read_to_string(file_path).unwrap();
        eprintln!("{}", saved_content);
        assert!(saved_content.contains("msg_conversion"));
    }
}
