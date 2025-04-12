//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#[cfg(test)]
mod tests {
    use ten_manager::designer::graphs::nodes::delete::graph_delete_extension_node;
    use ten_rust::{
        graph::{
            connection::{GraphConnection, GraphDestination, GraphMessageFlow},
            node::GraphNode,
            Graph,
        },
        pkg_info::{pkg_type::PkgType, pkg_type_and_name::PkgTypeAndName},
    };

    fn create_test_node(
        name: &str,
        addon: &str,
        app: Option<&str>,
        extension_group: Option<&str>,
    ) -> GraphNode {
        GraphNode {
            type_and_name: PkgTypeAndName {
                pkg_type: PkgType::Extension,
                name: name.to_string(),
            },
            addon: addon.to_string(),
            extension_group: extension_group.map(|s| s.to_string()),
            app: app.map(|s| s.to_string()),
            property: None,
        }
    }

    fn create_test_connection(
        extension: &str,
        app: Option<&str>,
        cmd_name: &str,
        dest_extension: &str,
        dest_app: Option<&str>,
    ) -> GraphConnection {
        let dest = GraphDestination {
            app: dest_app.map(|s| s.to_string()),
            extension: dest_extension.to_string(),
            msg_conversion: None,
        };

        let message_flow = GraphMessageFlow {
            name: cmd_name.to_string(),
            dest: vec![dest],
        };

        GraphConnection {
            app: app.map(|s| s.to_string()),
            extension: extension.to_string(),
            cmd: Some(vec![message_flow]),
            data: None,
            audio_frame: None,
            video_frame: None,
        }
    }

    #[test]
    fn test_delete_extension_node() {
        // Create a graph with multiple nodes and connections.
        let mut graph = Graph {
            nodes: vec![
                create_test_node(
                    "ext1",
                    "addon1",
                    Some("app1"),
                    Some("group1"),
                ),
                create_test_node(
                    "ext2",
                    "addon2",
                    Some("app1"),
                    Some("group2"),
                ),
                create_test_node(
                    "ext3",
                    "addon3",
                    Some("app2"),
                    Some("group3"),
                ),
            ],
            connections: Some(vec![
                create_test_connection(
                    "ext1",
                    Some("app1"),
                    "cmd1",
                    "ext2",
                    Some("app1"),
                ),
                create_test_connection(
                    "ext2",
                    Some("app1"),
                    "cmd2",
                    "ext3",
                    Some("app2"),
                ),
                create_test_connection(
                    "ext3",
                    Some("app2"),
                    "cmd3",
                    "ext1",
                    Some("app1"),
                ),
            ]),
        };

        // Test case 1: Delete a node that doesn't exist.
        let result = graph_delete_extension_node(
            &mut graph,
            "non_existent".to_string(),
            "addon1".to_string(),
            Some("app1".to_string()),
            None,
        );
        assert!(result.is_ok());
        assert_eq!(graph.nodes.len(), 3);
        assert_eq!(graph.connections.as_ref().unwrap().len(), 3);

        // Test case 2: Delete ext1 - should remove node and connections to/from
        // it.
        let result = graph_delete_extension_node(
            &mut graph,
            "ext1".to_string(),
            "addon1".to_string(),
            Some("app1".to_string()),
            Some("group1".to_string()),
        );
        assert!(result.is_ok());
        assert_eq!(graph.nodes.len(), 2);

        // Check connections.
        // After deleting ext1:
        // - ext1's connection is removed directly.
        // - ext3's connection had only ext1 as destination, so it's removed.
        // - ext2's connection remains since it points to ext3.
        assert!(graph.connections.is_some());
        let connections = graph.connections.as_ref().unwrap();
        assert_eq!(connections.len(), 1);

        // The remaining connection should be for ext2.
        assert_eq!(connections[0].extension, "ext2");

        // Test case 3: Delete ext3 - should remove node and connections to/from
        // it.
        let result = graph_delete_extension_node(
            &mut graph,
            "ext3".to_string(),
            "addon3".to_string(),
            Some("app2".to_string()),
            Some("group3".to_string()),
        );
        assert!(result.is_ok());
        assert_eq!(graph.nodes.len(), 1);

        // After deleting ext3:
        // - ext2's connection is empty because its only destination (ext3) is
        //   gone.
        // So all connections are removed.
        assert!(graph.connections.is_none());

        // Test case 4: Delete the last node - should have no effect on
        // connections (already None).
        let result = graph_delete_extension_node(
            &mut graph,
            "ext2".to_string(),
            "addon2".to_string(),
            Some("app1".to_string()),
            Some("group2".to_string()),
        );
        assert!(result.is_ok());
        assert_eq!(graph.nodes.len(), 0);
        assert!(graph.connections.is_none());
    }

    #[test]
    fn test_delete_extension_node_multiple_message_types() {
        // Create a graph with multiple message types in connections.
        let mut graph = Graph {
            nodes: vec![
                create_test_node("ext1", "addon1", Some("app1"), None),
                create_test_node("ext2", "addon2", Some("app1"), None),
            ],
            connections: Some(vec![]),
        };

        // Add a connection with multiple message types.
        let connection = GraphConnection {
            app: Some("app1".to_string()),
            extension: "ext1".to_string(),
            cmd: Some(vec![GraphMessageFlow {
                name: "cmd1".to_string(),
                dest: vec![GraphDestination {
                    app: Some("app1".to_string()),
                    extension: "ext2".to_string(),
                    msg_conversion: None,
                }],
            }]),
            data: Some(vec![GraphMessageFlow {
                name: "data1".to_string(),
                dest: vec![GraphDestination {
                    app: Some("app1".to_string()),
                    extension: "ext2".to_string(),
                    msg_conversion: None,
                }],
            }]),
            audio_frame: Some(vec![GraphMessageFlow {
                name: "audio1".to_string(),
                dest: vec![GraphDestination {
                    app: Some("app1".to_string()),
                    extension: "ext2".to_string(),
                    msg_conversion: None,
                }],
            }]),
            video_frame: Some(vec![GraphMessageFlow {
                name: "video1".to_string(),
                dest: vec![GraphDestination {
                    app: Some("app1".to_string()),
                    extension: "ext2".to_string(),
                    msg_conversion: None,
                }],
            }]),
        };

        graph.connections.as_mut().unwrap().push(connection);

        // Delete ext2 - should remove all destinations referring to it.
        // This will result in all message flows becoming empty, and since all
        // message flows are empty, the connection will be removed, and since no
        // connections are left, connections will be set to None.
        let result = graph_delete_extension_node(
            &mut graph,
            "ext2".to_string(),
            "addon2".to_string(),
            Some("app1".to_string()),
            None,
        );

        assert!(result.is_ok());
        assert_eq!(graph.nodes.len(), 1);

        // All connections should be removed since all had only ext2 as
        // destinations.
        assert!(graph.connections.is_none());
    }
}
