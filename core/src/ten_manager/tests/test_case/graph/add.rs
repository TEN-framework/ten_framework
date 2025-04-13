//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use ten_manager::{
        constants::TEST_DIR, graph::connections::add::graph_add_connection,
        pkg_info::create_uri_to_pkg_info_map,
    };
    use ten_rust::{
        graph::{node::GraphNode, Graph},
        pkg_info::{
            message::MsgType, pkg_type::PkgType,
            pkg_type_and_name::PkgTypeAndName,
        },
    };

    use crate::test_case::mock::inject_all_pkgs_for_mock;

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

    #[test]
    fn test_add_connection() {
        let all_pkgs_json_str = vec![
            (
                TEST_DIR.to_string(),
                include_str!("test_data_embed/app_manifest.json").to_string(),
                include_str!("test_data_embed/app_property.json").to_string(),
            ),
            (
                format!(
                    "{}{}",
                    TEST_DIR, "/ten_packages/extension/extension_1"
                ),
                include_str!("test_data_embed/ext_1_manifest.json").to_string(),
                "{}".to_string(),
            ),
            (
                format!(
                    "{}{}",
                    TEST_DIR, "/ten_packages/extension/extension_2"
                ),
                include_str!("test_data_embed/ext_2_manifest.json").to_string(),
                "{}".to_string(),
            ),
            (
                format!(
                    "{}{}",
                    TEST_DIR, "/ten_packages/extension/extension_3"
                ),
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

        // Test adding a connection.
        let result = graph_add_connection(
            &mut graph,
            Some("http://localhost:8000".to_string()),
            "ext1".to_string(),
            MsgType::Cmd,
            "test_cmd".to_string(),
            Some("http://localhost:8000".to_string()),
            "ext2".to_string(),
            &uri_to_pkg_info,
            None,
        );

        assert!(result.is_ok());
        assert!(graph.connections.is_some());

        let connections = graph.connections.as_ref().unwrap();
        assert_eq!(connections.len(), 1);

        let connection = &connections[0];
        assert_eq!(connection.app, Some("http://localhost:8000".to_string()));
        assert_eq!(connection.extension, "ext1");

        let cmd_flows = connection.cmd.as_ref().unwrap();
        assert_eq!(cmd_flows.len(), 1);

        let flow = &cmd_flows[0];
        assert_eq!(flow.name, "test_cmd");
        assert_eq!(flow.dest.len(), 1);

        let dest = &flow.dest[0];
        assert_eq!(dest.app, Some("http://localhost:8000".to_string()));
        assert_eq!(dest.extension, "ext2");
    }

    #[test]
    fn test_add_connection_nonexistent_source() {
        let all_pkgs_json_str = vec![
            (
                TEST_DIR.to_string(),
                include_str!("test_data_embed/app_manifest.json").to_string(),
                include_str!("test_data_embed/app_property.json").to_string(),
            ),
            (
                format!(
                    "{}{}",
                    TEST_DIR, "/ten_packages/extension/extension_1"
                ),
                include_str!("test_data_embed/ext_1_manifest.json").to_string(),
                "{}".to_string(),
            ),
            (
                format!(
                    "{}{}",
                    TEST_DIR, "/ten_packages/extension/extension_2"
                ),
                include_str!("test_data_embed/ext_2_manifest.json").to_string(),
                "{}".to_string(),
            ),
            (
                format!(
                    "{}{}",
                    TEST_DIR, "/ten_packages/extension/extension_3"
                ),
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

        // Create a graph with only one node.
        let mut graph = Graph {
            nodes: vec![create_test_node("ext2", "addon2", Some("app1"))],
            connections: None,
        };

        // Test adding a connection with nonexistent source.
        let result = graph_add_connection(
            &mut graph,
            Some("app1".to_string()),
            "ext1".to_string(), // This node doesn't exist.
            MsgType::Cmd,
            "test_cmd".to_string(),
            Some("app1".to_string()),
            "ext2".to_string(),
            &uri_to_pkg_info,
            None,
        );

        assert!(result.is_err());
        assert!(graph.connections.is_none()); // Graph should remain unchanged.
    }

    #[test]
    fn test_add_connection_nonexistent_destination() {
        let all_pkgs_json_str = vec![
            (
                TEST_DIR.to_string(),
                include_str!("test_data_embed/app_manifest.json").to_string(),
                include_str!("test_data_embed/app_property.json").to_string(),
            ),
            (
                format!(
                    "{}{}",
                    TEST_DIR, "/ten_packages/extension/extension_1"
                ),
                include_str!("test_data_embed/ext_1_manifest.json").to_string(),
                "{}".to_string(),
            ),
            (
                format!(
                    "{}{}",
                    TEST_DIR, "/ten_packages/extension/extension_2"
                ),
                include_str!("test_data_embed/ext_2_manifest.json").to_string(),
                "{}".to_string(),
            ),
            (
                format!(
                    "{}{}",
                    TEST_DIR, "/ten_packages/extension/extension_3"
                ),
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

        // Create a graph with only one node.
        let mut graph = Graph {
            nodes: vec![create_test_node("ext1", "addon1", Some("app1"))],
            connections: None,
        };

        // Test adding a connection with nonexistent destination.
        let result = graph_add_connection(
            &mut graph,
            Some("app1".to_string()),
            "ext1".to_string(),
            MsgType::Cmd,
            "test_cmd".to_string(),
            Some("app1".to_string()),
            "ext2".to_string(), // This node doesn't exist.
            &uri_to_pkg_info,
            None,
        );

        assert!(result.is_err());
        assert!(graph.connections.is_none()); // Graph should remain unchanged.
    }

    #[test]
    fn test_add_connection_to_existing_flow() {
        let all_pkgs_json_str = vec![
            (
                TEST_DIR.to_string(),
                include_str!("test_data_embed/app_manifest.json").to_string(),
                include_str!("test_data_embed/app_property.json").to_string(),
            ),
            (
                format!(
                    "{}{}",
                    TEST_DIR, "/ten_packages/extension/extension_1"
                ),
                include_str!("test_data_embed/ext_1_manifest.json").to_string(),
                "{}".to_string(),
            ),
            (
                format!(
                    "{}{}",
                    TEST_DIR, "/ten_packages/extension/extension_2"
                ),
                include_str!("test_data_embed/ext_2_manifest.json").to_string(),
                "{}".to_string(),
            ),
            (
                format!(
                    "{}{}",
                    TEST_DIR, "/ten_packages/extension/extension_3"
                ),
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

        // Create a graph with three nodes.
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
            ],
            connections: None,
        };

        // Add first connection.
        let result = graph_add_connection(
            &mut graph,
            Some("http://localhost:8000".to_string()),
            "ext1".to_string(),
            MsgType::Cmd,
            "test_cmd".to_string(),
            Some("http://localhost:8000".to_string()),
            "ext2".to_string(),
            &uri_to_pkg_info,
            None,
        );
        assert!(result.is_ok());

        // Add second connection with same source and message name but different
        // destination.
        let result = graph_add_connection(
            &mut graph,
            Some("http://localhost:8000".to_string()),
            "ext1".to_string(),
            MsgType::Cmd,
            "test_cmd".to_string(),
            Some("http://localhost:8000".to_string()),
            "ext3".to_string(),
            &uri_to_pkg_info,
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
        let all_pkgs_json_str = vec![
            (
                TEST_DIR.to_string(),
                include_str!("test_data_embed/app_manifest.json").to_string(),
                include_str!("test_data_embed/app_property.json").to_string(),
            ),
            (
                format!(
                    "{}{}",
                    TEST_DIR, "/ten_packages/extension/extension_1"
                ),
                include_str!("test_data_embed/ext_1_manifest.json").to_string(),
                "{}".to_string(),
            ),
            (
                format!(
                    "{}{}",
                    TEST_DIR, "/ten_packages/extension/extension_2"
                ),
                include_str!("test_data_embed/ext_2_manifest.json").to_string(),
                "{}".to_string(),
            ),
            (
                format!(
                    "{}{}",
                    TEST_DIR, "/ten_packages/extension/extension_3"
                ),
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

        // Add different message types.
        let result = graph_add_connection(
            &mut graph,
            Some("http://localhost:8000".to_string()),
            "ext1".to_string(),
            MsgType::Cmd,
            "cmd1".to_string(),
            Some("http://localhost:8000".to_string()),
            "ext2".to_string(),
            &uri_to_pkg_info,
            None,
        );
        assert!(result.is_ok());

        let result = graph_add_connection(
            &mut graph,
            Some("http://localhost:8000".to_string()),
            "ext1".to_string(),
            MsgType::Data,
            "data1".to_string(),
            Some("http://localhost:8000".to_string()),
            "ext2".to_string(),
            &uri_to_pkg_info,
            None,
        );
        assert!(result.is_ok());

        let result = graph_add_connection(
            &mut graph,
            Some("http://localhost:8000".to_string()),
            "ext1".to_string(),
            MsgType::AudioFrame,
            "audio1".to_string(),
            Some("http://localhost:8000".to_string()),
            "ext2".to_string(),
            &uri_to_pkg_info,
            None,
        );
        assert!(result.is_ok());

        let result = graph_add_connection(
            &mut graph,
            Some("http://localhost:8000".to_string()),
            "ext1".to_string(),
            MsgType::VideoFrame,
            "video1".to_string(),
            Some("http://localhost:8000".to_string()),
            "ext2".to_string(),
            &uri_to_pkg_info,
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
        let all_pkgs_json_str = vec![
            (
                TEST_DIR.to_string(),
                include_str!("test_data_embed/app_manifest.json").to_string(),
                include_str!("test_data_embed/app_property.json").to_string(),
            ),
            (
                format!(
                    "{}{}",
                    TEST_DIR, "/ten_packages/extension/extension_1"
                ),
                include_str!("test_data_embed/ext_1_manifest.json").to_string(),
                "{}".to_string(),
            ),
            (
                format!(
                    "{}{}",
                    TEST_DIR, "/ten_packages/extension/extension_2"
                ),
                include_str!("test_data_embed/ext_2_manifest.json").to_string(),
                "{}".to_string(),
            ),
            (
                format!(
                    "{}{}",
                    TEST_DIR, "/ten_packages/extension/extension_3"
                ),
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

        // Add a connection.
        let result = graph_add_connection(
            &mut graph,
            Some("http://localhost:8000".to_string()),
            "ext1".to_string(),
            MsgType::Cmd,
            "test_cmd".to_string(),
            Some("http://localhost:8000".to_string()),
            "ext2".to_string(),
            &uri_to_pkg_info,
            None,
        );
        assert!(result.is_ok());

        // Try to add the same connection again.
        let result = graph_add_connection(
            &mut graph,
            Some("http://localhost:8000".to_string()),
            "ext1".to_string(),
            MsgType::Cmd,
            "test_cmd".to_string(),
            Some("http://localhost:8000".to_string()),
            "ext2".to_string(),
            &uri_to_pkg_info,
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
        let all_pkgs_json_str = vec![
            (
                TEST_DIR.to_string(),
                include_str!("test_data_embed/app_manifest.json").to_string(),
                include_str!("test_data_embed/app_property.json").to_string(),
            ),
            (
                format!(
                    "{}{}",
                    TEST_DIR, "/ten_packages/extension/extension_1"
                ),
                include_str!("test_data_embed/ext_1_manifest.json").to_string(),
                "{}".to_string(),
            ),
            (
                format!(
                    "{}{}",
                    TEST_DIR, "/ten_packages/extension/extension_2"
                ),
                include_str!("test_data_embed/ext_2_manifest.json").to_string(),
                "{}".to_string(),
            ),
            (
                format!(
                    "{}{}",
                    TEST_DIR, "/ten_packages/extension/extension_3"
                ),
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

        // Create a graph with three nodes.
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
            ],
            connections: None,
        };

        // Test connecting ext1 to ext2 with compatible schema - should succeed.
        let result = graph_add_connection(
            &mut graph,
            Some("http://localhost:8000".to_string()),
            "ext1".to_string(),
            MsgType::Cmd,
            "cmd1".to_string(),
            Some("http://localhost:8000".to_string()),
            "ext2".to_string(),
            &uri_to_pkg_info,
            None,
        );
        assert!(result.is_ok());

        // Test connecting ext1 to ext3 with compatible schema - should succeed.
        let result = graph_add_connection(
            &mut graph,
            Some("http://localhost:8000".to_string()),
            "ext1".to_string(),
            MsgType::Data,
            "data1".to_string(),
            Some("http://localhost:8000".to_string()),
            "ext3".to_string(),
            &uri_to_pkg_info,
            None,
        );
        assert!(result.is_ok());

        // Test connecting ext1 to ext3 with incompatible schema - should fail.
        let result = graph_add_connection(
            &mut graph,
            Some("http://localhost:8000".to_string()),
            "ext1".to_string(),
            MsgType::Cmd,
            "cmd_incompatible".to_string(),
            Some("http://localhost:8000".to_string()),
            "ext3".to_string(),
            &uri_to_pkg_info,
            None,
        );
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("schema incompatibility"));

        // Test connecting ext1 to ext3 with incompatible schema for data -
        // should fail.
        let result = graph_add_connection(
            &mut graph,
            Some("http://localhost:8000".to_string()),
            "ext1".to_string(),
            MsgType::Data,
            "data_incompatible".to_string(),
            Some("http://localhost:8000".to_string()),
            "ext3".to_string(),
            &uri_to_pkg_info,
            None,
        );
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("schema incompatibility"));
    }
}
