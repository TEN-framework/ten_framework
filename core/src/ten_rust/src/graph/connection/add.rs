//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::graph::{
    connection::{GraphConnection, GraphDestination, GraphMessageFlow},
    Graph,
};

/// Message type enum representing the different types of message flows in a
/// connection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageType {
    Command,
    Data,
    AudioFrame,
    VideoFrame,
}

impl Graph {
    /// Adds a new connection between two extension nodes in the graph.
    pub fn add_connection(
        &mut self,
        src_app: Option<String>,
        src_extension: String,
        msg_type: MessageType,
        msg_name: String,
        dest_app: Option<String>,
        dest_extension: String,
    ) -> Result<()> {
        // Store the original state in case validation fails.
        let original_graph = self.clone();

        // Validate that source node exists.
        let src_node_exists = self.nodes.iter().any(|node| {
            node.type_and_name.name == src_extension && node.app == src_app
        });

        if !src_node_exists {
            return Err(anyhow::anyhow!(
                "Source node with extension '{}' and app '{:?}' not found in the graph",
                src_extension,
                src_app
            ));
        }

        // Validate that destination node exists.
        let dest_node_exists = self.nodes.iter().any(|node| {
            node.type_and_name.name == dest_extension && node.app == dest_app
        });

        if !dest_node_exists {
            return Err(anyhow::anyhow!(
                "Destination node with extension '{}' and app '{:?}' not found in the graph",
                dest_extension,
                dest_app
            ));
        }

        // Check if this connection already exists.
        if let Some(connections) = &self.connections {
            for conn in connections.iter() {
                // Check if source matches
                if conn.extension == src_extension && conn.app == src_app {
                    // Check for duplicate message flows based on message type.
                    let msg_flows = match msg_type {
                        MessageType::Command => conn.cmd.as_ref(),
                        MessageType::Data => conn.data.as_ref(),
                        MessageType::AudioFrame => conn.audio_frame.as_ref(),
                        MessageType::VideoFrame => conn.video_frame.as_ref(),
                    };

                    if let Some(flows) = msg_flows {
                        for flow in flows {
                            // Check if message name matches.
                            if flow.name == msg_name {
                                // Check if destination already exists.
                                for dest in &flow.dest {
                                    if dest.extension == dest_extension
                                        && dest.app == dest_app
                                    {
                                        return Err(anyhow::anyhow!(
                                            "Connection already exists: src:({:?}, {}), msg_type:{:?}, msg_name:{}, dest:({:?}, {})",
                                            src_app, src_extension, msg_type, msg_name, dest_app, dest_extension
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Create destination object.
        let destination = GraphDestination {
            app: dest_app,
            extension: dest_extension,
            msg_conversion: None,
        };

        // Initialize connections if None.
        if self.connections.is_none() {
            self.connections = Some(Vec::new());
        }

        // Get or create a connection for the source node.
        let connections = self.connections.as_mut().unwrap();
        let connection = match connections
            .iter_mut()
            .find(|conn| conn.extension == src_extension && conn.app == src_app)
        {
            Some(conn) => conn,
            None => {
                // Create a new connection for the source node.
                let new_connection = GraphConnection {
                    app: src_app.clone(),
                    extension: src_extension.clone(),
                    cmd: None,
                    data: None,
                    audio_frame: None,
                    video_frame: None,
                };
                connections.push(new_connection);
                connections.last_mut().unwrap()
            }
        };

        // Create a message flow.
        let message_flow = GraphMessageFlow {
            name: msg_name,
            dest: vec![destination],
        };

        // Add the message flow to the appropriate vector based on message type.
        match msg_type {
            MessageType::Command => {
                if connection.cmd.is_none() {
                    connection.cmd = Some(Vec::new());
                }

                // Check if a message flow with the same name already exists.
                let cmd_flows = connection.cmd.as_mut().unwrap();
                if let Some(existing_flow) = cmd_flows
                    .iter_mut()
                    .find(|flow| flow.name == message_flow.name)
                {
                    // Add the destination to the existing flow if it doesn't
                    // already exist.
                    if !existing_flow.dest.iter().any(|dest| {
                        dest.extension == message_flow.dest[0].extension
                            && dest.app == message_flow.dest[0].app
                    }) {
                        existing_flow.dest.push(message_flow.dest[0].clone());
                    }
                } else {
                    // Add the new message flow.
                    cmd_flows.push(message_flow);
                }
            }
            MessageType::Data => {
                if connection.data.is_none() {
                    connection.data = Some(Vec::new());
                }

                // Check if a message flow with the same name already exists.
                let data_flows = connection.data.as_mut().unwrap();
                if let Some(existing_flow) = data_flows
                    .iter_mut()
                    .find(|flow| flow.name == message_flow.name)
                {
                    // Add the destination to the existing flow if it doesn't
                    // already exist.
                    if !existing_flow.dest.iter().any(|dest| {
                        dest.extension == message_flow.dest[0].extension
                            && dest.app == message_flow.dest[0].app
                    }) {
                        existing_flow.dest.push(message_flow.dest[0].clone());
                    }
                } else {
                    // Add the new message flow.
                    data_flows.push(message_flow);
                }
            }
            MessageType::AudioFrame => {
                if connection.audio_frame.is_none() {
                    connection.audio_frame = Some(Vec::new());
                }

                // Check if a message flow with the same name already exists.
                let audio_flows = connection.audio_frame.as_mut().unwrap();
                if let Some(existing_flow) = audio_flows
                    .iter_mut()
                    .find(|flow| flow.name == message_flow.name)
                {
                    // Add the destination to the existing flow if it doesn't
                    // already exist.
                    if !existing_flow.dest.iter().any(|dest| {
                        dest.extension == message_flow.dest[0].extension
                            && dest.app == message_flow.dest[0].app
                    }) {
                        existing_flow.dest.push(message_flow.dest[0].clone());
                    }
                } else {
                    // Add the new message flow.
                    audio_flows.push(message_flow);
                }
            }
            MessageType::VideoFrame => {
                if connection.video_frame.is_none() {
                    connection.video_frame = Some(Vec::new());
                }

                // Check if a message flow with the same name already exists.
                let video_flows = connection.video_frame.as_mut().unwrap();
                if let Some(existing_flow) = video_flows
                    .iter_mut()
                    .find(|flow| flow.name == message_flow.name)
                {
                    // Add the destination to the existing flow if it doesn't
                    // already exist.
                    if !existing_flow.dest.iter().any(|dest| {
                        dest.extension == message_flow.dest[0].extension
                            && dest.app == message_flow.dest[0].app
                    }) {
                        existing_flow.dest.push(message_flow.dest[0].clone());
                    }
                } else {
                    // Add the new message flow.
                    video_flows.push(message_flow);
                }
            }
        }

        // Validate the updated graph.
        match self.validate_and_complete() {
            Ok(_) => Ok(()),
            Err(e) => {
                // Restore the original graph if validation fails.
                *self = original_graph;
                Err(e)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::node::GraphNode;
    use crate::pkg_info::pkg_type::PkgType;
    use crate::pkg_info::pkg_type_and_name::PkgTypeAndName;

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
            MessageType::Command,
            "test_cmd".to_string(),
            Some("app1".to_string()),
            "ext2".to_string(),
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
            MessageType::Command,
            "test_cmd".to_string(),
            Some("app1".to_string()),
            "ext2".to_string(),
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
            MessageType::Command,
            "test_cmd".to_string(),
            Some("app1".to_string()),
            "ext2".to_string(), // This node doesn't exist.
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

        // Add first connection.
        let result = graph.add_connection(
            Some("app1".to_string()),
            "ext1".to_string(),
            MessageType::Command,
            "test_cmd".to_string(),
            Some("app1".to_string()),
            "ext2".to_string(),
        );
        assert!(result.is_ok());

        // Add second connection with same source and message name but different
        // destination.
        let result = graph.add_connection(
            Some("app1".to_string()),
            "ext1".to_string(),
            MessageType::Command,
            "test_cmd".to_string(),
            Some("app1".to_string()),
            "ext3".to_string(),
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

        // Add different message types.
        let result = graph.add_connection(
            Some("app1".to_string()),
            "ext1".to_string(),
            MessageType::Command,
            "cmd1".to_string(),
            Some("app1".to_string()),
            "ext2".to_string(),
        );
        assert!(result.is_ok());

        let result = graph.add_connection(
            Some("app1".to_string()),
            "ext1".to_string(),
            MessageType::Data,
            "data1".to_string(),
            Some("app1".to_string()),
            "ext2".to_string(),
        );
        assert!(result.is_ok());

        let result = graph.add_connection(
            Some("app1".to_string()),
            "ext1".to_string(),
            MessageType::AudioFrame,
            "audio1".to_string(),
            Some("app1".to_string()),
            "ext2".to_string(),
        );
        assert!(result.is_ok());

        let result = graph.add_connection(
            Some("app1".to_string()),
            "ext1".to_string(),
            MessageType::VideoFrame,
            "video1".to_string(),
            Some("app1".to_string()),
            "ext2".to_string(),
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

        // Add a connection.
        let result = graph.add_connection(
            Some("app1".to_string()),
            "ext1".to_string(),
            MessageType::Command,
            "test_cmd".to_string(),
            Some("app1".to_string()),
            "ext2".to_string(),
        );
        assert!(result.is_ok());

        // Try to add the same connection again.
        let result = graph.add_connection(
            Some("app1".to_string()),
            "ext1".to_string(),
            MessageType::Command,
            "test_cmd".to_string(),
            Some("app1".to_string()),
            "ext2".to_string(),
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
}
