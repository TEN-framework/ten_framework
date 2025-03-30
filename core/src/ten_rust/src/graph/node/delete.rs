//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use anyhow::Result;

use crate::graph::Graph;
use crate::pkg_info::pkg_type::PkgType;

impl Graph {
    pub fn delete_extension_node(
        &mut self,
        pkg_name: String,
        addon: String,
        app: Option<String>,
        extension_group: Option<String>,
    ) -> Result<()> {
        // Find and remove the matching node.
        let original_nodes_len = self.nodes.len();
        self.nodes.retain(|node| {
            !(node.type_and_name.pkg_type == PkgType::Extension
                && node.type_and_name.name == pkg_name
                && node.addon == addon
                && node.app == app
                && node.extension_group == extension_group)
        });

        // If no node was removed, return early.
        if self.nodes.len() == original_nodes_len {
            return Ok(());
        }

        // The node was removed, now clean up any connections.
        if let Some(connections) = &mut self.connections {
            // 1. Remove entire connections with matching app and extension.
            connections.retain(|conn| {
                !(conn.extension == pkg_name && conn.app == app)
            });

            // 2. Remove destinations from message flows in all connections.
            for connection in connections.iter_mut() {
                // Process cmd flows.
                if let Some(cmd_flows) = &mut connection.cmd {
                    for flow in cmd_flows.iter_mut() {
                        flow.dest.retain(|dest| {
                            !(dest.extension == pkg_name && dest.app == app)
                        });
                    }
                    // Remove empty cmd flows.
                    cmd_flows.retain(|flow| !flow.dest.is_empty());
                }

                // Process data flows.
                if let Some(data_flows) = &mut connection.data {
                    for flow in data_flows.iter_mut() {
                        flow.dest.retain(|dest| {
                            !(dest.extension == pkg_name && dest.app == app)
                        });
                    }
                    // Remove empty data flows.
                    data_flows.retain(|flow| !flow.dest.is_empty());
                }

                // Process audio_frame flows.
                if let Some(audio_flows) = &mut connection.audio_frame {
                    for flow in audio_flows.iter_mut() {
                        flow.dest.retain(|dest| {
                            !(dest.extension == pkg_name && dest.app == app)
                        });
                    }
                    // Remove empty audio_frame flows.
                    audio_flows.retain(|flow| !flow.dest.is_empty());
                }

                // Process video_frame flows.
                if let Some(video_flows) = &mut connection.video_frame {
                    for flow in video_flows.iter_mut() {
                        flow.dest.retain(|dest| {
                            !(dest.extension == pkg_name && dest.app == app)
                        });
                    }
                    // Remove empty video_frame flows.
                    video_flows.retain(|flow| !flow.dest.is_empty());
                }
            }

            // Remove connections that have no message flows left.
            connections.retain(|conn| {
                let has_cmd = conn.cmd.as_ref().is_some_and(|c| !c.is_empty());
                let has_data =
                    conn.data.as_ref().is_some_and(|d| !d.is_empty());
                let has_audio =
                    conn.audio_frame.as_ref().is_some_and(|a| !a.is_empty());
                let has_video =
                    conn.video_frame.as_ref().is_some_and(|v| !v.is_empty());
                has_cmd || has_data || has_audio || has_video
            });

            // If no connections left, set connections to None.
            if connections.is_empty() {
                self.connections = None;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::connection::{
        GraphConnection, GraphDestination, GraphMessageFlow,
    };
    use crate::graph::node::GraphNode;
    use crate::pkg_info::pkg_type::PkgType;
    use crate::pkg_info::pkg_type_and_name::PkgTypeAndName;

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
        let result = graph.delete_extension_node(
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
        let result = graph.delete_extension_node(
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
        let result = graph.delete_extension_node(
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
        let result = graph.delete_extension_node(
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
        let result = graph.delete_extension_node(
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
