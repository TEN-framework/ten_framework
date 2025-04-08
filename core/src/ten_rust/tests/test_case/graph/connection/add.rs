//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#[cfg(test)]
mod tests {
    use std::{collections::HashMap, str::FromStr};

    use ten_rust::{
        base_dir_pkg_info::{PkgsInfoInApp, PkgsInfoInAppWithBaseDir},
        graph::{node::GraphNode, Graph},
        pkg_info::{
            manifest::Manifest, message::MsgType, pkg_type::PkgType,
            pkg_type_and_name::PkgTypeAndName,
            property::parse_property_from_str, PkgInfo,
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

    fn create_test_pkg_info_map(
    ) -> HashMap<Option<String>, PkgsInfoInAppWithBaseDir> {
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
        let app_property =
            parse_property_from_str(prop_str, &mut HashMap::new()).unwrap();

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

        // Create a PkgsInfoInAppWithBaseDir and add all packages
        let base_dir_pkg_info = PkgsInfoInAppWithBaseDir {
            pkgs_info_in_app: PkgsInfoInApp {
                app_pkg_info: Some(app_pkg_info),
                extension_pkg_info: Some(vec![
                    ext1_pkg_info,
                    ext2_pkg_info,
                    ext3_pkg_info,
                ]),
                protocol_pkg_info: None,
                addon_loader_pkg_info: None,
                system_pkg_info: None,
            },
            base_dir: "app1".to_string(),
        };

        // Add to map with app URI as key
        map.insert(Some("app1".to_string()), base_dir_pkg_info);

        map
    }

    #[test]
    fn test_add_connection() {
        // Create a graph with two nodes.
        let mut graph = Graph {
            nodes: vec![
                create_test_node("ext1", "addon1", Some("app1")),
                create_test_node("ext2", "addon2", Some("app1")),
            ],
            connections: None,
        };

        // Test adding a connection.
        let result = graph.add_connection(
            Some("app1".to_string()),
            "ext1".to_string(),
            MsgType::Cmd,
            "test_cmd".to_string(),
            Some("app1".to_string()),
            "ext2".to_string(),
            &create_test_pkg_info_map(),
            None,
        );

        assert!(result.is_ok());
        assert!(graph.connections.is_some());

        let connections = graph.connections.as_ref().unwrap();
        assert_eq!(connections.len(), 1);

        let connection = &connections[0];
        assert_eq!(connection.app, Some("app1".to_string()));
        assert_eq!(connection.extension, "ext1");

        let cmd_flows = connection.cmd.as_ref().unwrap();
        assert_eq!(cmd_flows.len(), 1);

        let flow = &cmd_flows[0];
        assert_eq!(flow.name, "test_cmd");
        assert_eq!(flow.dest.len(), 1);

        let dest = &flow.dest[0];
        assert_eq!(dest.app, Some("app1".to_string()));
        assert_eq!(dest.extension, "ext2");
    }

    #[test]
    fn test_add_connection_nonexistent_source() {
        // Create a graph with only one node.
        let mut graph = Graph {
            nodes: vec![create_test_node("ext2", "addon2", Some("app1"))],
            connections: None,
        };

        // Test adding a connection with nonexistent source.
        let result = graph.add_connection(
            Some("app1".to_string()),
            "ext1".to_string(), // This node doesn't exist.
            MsgType::Cmd,
            "test_cmd".to_string(),
            Some("app1".to_string()),
            "ext2".to_string(),
            &create_test_pkg_info_map(),
            None,
        );

        assert!(result.is_err());
        assert!(graph.connections.is_none()); // Graph should remain unchanged.
    }

    #[test]
    fn test_add_connection_nonexistent_destination() {
        // Create a graph with only one node.
        let mut graph = Graph {
            nodes: vec![create_test_node("ext1", "addon1", Some("app1"))],
            connections: None,
        };

        // Test adding a connection with nonexistent destination.
        let result = graph.add_connection(
            Some("app1".to_string()),
            "ext1".to_string(),
            MsgType::Cmd,
            "test_cmd".to_string(),
            Some("app1".to_string()),
            "ext2".to_string(), // This node doesn't exist.
            &create_test_pkg_info_map(),
            None,
        );

        assert!(result.is_err());
        assert!(graph.connections.is_none()); // Graph should remain unchanged.
    }

    #[test]
    fn test_add_connection_to_existing_flow() {
        // Create a graph with three nodes.
        let mut graph = Graph {
            nodes: vec![
                create_test_node("ext1", "addon1", Some("app1")),
                create_test_node("ext2", "addon2", Some("app1")),
                create_test_node("ext3", "addon3", Some("app1")),
            ],
            connections: None,
        };

        let pkg_info_map = create_test_pkg_info_map();

        // Add first connection.
        let result = graph.add_connection(
            Some("app1".to_string()),
            "ext1".to_string(),
            MsgType::Cmd,
            "test_cmd".to_string(),
            Some("app1".to_string()),
            "ext2".to_string(),
            &pkg_info_map,
            None,
        );
        assert!(result.is_ok());

        // Add second connection with same source and message name but different
        // destination.
        let result = graph.add_connection(
            Some("app1".to_string()),
            "ext1".to_string(),
            MsgType::Cmd,
            "test_cmd".to_string(),
            Some("app1".to_string()),
            "ext3".to_string(),
            &pkg_info_map,
            None,
        );
        assert!(result.is_ok());

        // Verify that we have one connection with one message flow that has two
        // destinations.
        let connections = graph.connections.as_ref().unwrap();
        assert_eq!(connections.len(), 1);

        let connection = &connections[0];
        let cmd_flows = connection.cmd.as_ref().unwrap();
        assert_eq!(cmd_flows.len(), 1);

        let flow = &cmd_flows[0];
        assert_eq!(flow.name, "test_cmd");
        assert_eq!(flow.dest.len(), 2);

        // Verify destinations.
        assert_eq!(flow.dest[0].extension, "ext2");
        assert_eq!(flow.dest[1].extension, "ext3");
    }

    #[test]
    fn test_add_different_message_types() {
        // Create a graph with two nodes.
        let mut graph = Graph {
            nodes: vec![
                create_test_node("ext1", "addon1", Some("app1")),
                create_test_node("ext2", "addon2", Some("app1")),
            ],
            connections: None,
        };

        let pkg_info_map = create_test_pkg_info_map();

        // Add different message types.
        let result = graph.add_connection(
            Some("app1".to_string()),
            "ext1".to_string(),
            MsgType::Cmd,
            "cmd1".to_string(),
            Some("app1".to_string()),
            "ext2".to_string(),
            &pkg_info_map,
            None,
        );
        assert!(result.is_ok());

        let result = graph.add_connection(
            Some("app1".to_string()),
            "ext1".to_string(),
            MsgType::Data,
            "data1".to_string(),
            Some("app1".to_string()),
            "ext2".to_string(),
            &pkg_info_map,
            None,
        );
        assert!(result.is_ok());

        let result = graph.add_connection(
            Some("app1".to_string()),
            "ext1".to_string(),
            MsgType::AudioFrame,
            "audio1".to_string(),
            Some("app1".to_string()),
            "ext2".to_string(),
            &pkg_info_map,
            None,
        );
        assert!(result.is_ok());

        let result = graph.add_connection(
            Some("app1".to_string()),
            "ext1".to_string(),
            MsgType::VideoFrame,
            "video1".to_string(),
            Some("app1".to_string()),
            "ext2".to_string(),
            &pkg_info_map,
            None,
        );
        assert!(result.is_ok());

        // Verify that we have one connection with different message flows.
        let connection = &graph.connections.as_ref().unwrap()[0];

        assert!(connection.cmd.is_some());
        assert!(connection.data.is_some());
        assert!(connection.audio_frame.is_some());
        assert!(connection.video_frame.is_some());

        assert_eq!(connection.cmd.as_ref().unwrap()[0].name, "cmd1");
        assert_eq!(connection.data.as_ref().unwrap()[0].name, "data1");
        assert_eq!(connection.audio_frame.as_ref().unwrap()[0].name, "audio1");
        assert_eq!(connection.video_frame.as_ref().unwrap()[0].name, "video1");
    }

    #[test]
    fn test_add_duplicate_connection() {
        // Create a graph with two nodes.
        let mut graph = Graph {
            nodes: vec![
                create_test_node("ext1", "addon1", Some("app1")),
                create_test_node("ext2", "addon2", Some("app1")),
            ],
            connections: None,
        };

        let pkg_info_map = create_test_pkg_info_map();

        // Add a connection.
        let result = graph.add_connection(
            Some("app1".to_string()),
            "ext1".to_string(),
            MsgType::Cmd,
            "test_cmd".to_string(),
            Some("app1".to_string()),
            "ext2".to_string(),
            &pkg_info_map,
            None,
        );
        assert!(result.is_ok());

        // Try to add the same connection again.
        let result = graph.add_connection(
            Some("app1".to_string()),
            "ext1".to_string(),
            MsgType::Cmd,
            "test_cmd".to_string(),
            Some("app1".to_string()),
            "ext2".to_string(),
            &pkg_info_map,
            None,
        );

        // This should fail because the connection already exists.
        assert!(result.is_err());

        // The error message should indicate that the connection already exists.
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Connection already exists"));

        // Verify that the graph wasn't changed by the second add attempt.
        let connections = graph.connections.as_ref().unwrap();
        assert_eq!(connections.len(), 1);

        let connection = &connections[0];
        let cmd_flows = connection.cmd.as_ref().unwrap();
        assert_eq!(cmd_flows.len(), 1);

        let flow = &cmd_flows[0];
        assert_eq!(flow.dest.len(), 1);
    }

    #[test]
    fn test_schema_compatibility_check() {
        // Create a graph with three nodes.
        let mut graph = Graph {
            nodes: vec![
                create_test_node("ext1", "addon1", Some("app1")),
                create_test_node("ext2", "addon2", Some("app1")),
                create_test_node("ext3", "addon3", Some("app1")),
            ],
            connections: None,
        };

        let pkg_info_map = create_test_pkg_info_map();

        // Test connecting ext1 to ext2 with compatible schema - should succeed.
        let result = graph.add_connection(
            Some("app1".to_string()),
            "ext1".to_string(),
            MsgType::Cmd,
            "cmd1".to_string(),
            Some("app1".to_string()),
            "ext2".to_string(),
            &pkg_info_map,
            None,
        );
        assert!(result.is_ok());

        // Test connecting ext1 to ext3 with compatible schema - should succeed.
        let result = graph.add_connection(
            Some("app1".to_string()),
            "ext1".to_string(),
            MsgType::Data,
            "data1".to_string(),
            Some("app1".to_string()),
            "ext3".to_string(),
            &pkg_info_map,
            None,
        );
        assert!(result.is_ok());

        // Test connecting ext1 to ext3 with incompatible schema - should fail.
        let result = graph.add_connection(
            Some("app1".to_string()),
            "ext1".to_string(),
            MsgType::Cmd,
            "cmd_incompatible".to_string(),
            Some("app1".to_string()),
            "ext3".to_string(),
            &pkg_info_map,
            None,
        );
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("schema incompatibility"));

        // Test connecting ext1 to ext3 with incompatible schema for data -
        // should fail.
        let result = graph.add_connection(
            Some("app1".to_string()),
            "ext1".to_string(),
            MsgType::Data,
            "data_incompatible".to_string(),
            Some("app1".to_string()),
            "ext3".to_string(),
            &pkg_info_map,
            None,
        );
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("schema incompatibility"));
    }
}
