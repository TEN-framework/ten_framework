//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use anyhow::{anyhow, Result};

use crate::graph::Graph;
use crate::pkg_info::message::MsgType;

impl Graph {
    /// Delete a connection from the graph.
    pub fn delete_connection(
        &mut self,
        src_app: Option<String>,
        src_extension: String,
        msg_type: MsgType,
        msg_name: String,
        dest_app: Option<String>,
        dest_extension: String,
    ) -> Result<()> {
        // If no connections exist, return an error.
        if self.connections.is_none() {
            return Err(anyhow!("No connections found in the graph"));
        }

        let connections = self.connections.as_mut().unwrap();

        // Find the source node's connection in the connections list.
        let connection_idx = connections.iter().position(|conn| {
            conn.app == src_app && conn.extension == src_extension
        });

        if let Some(idx) = connection_idx {
            let connection = &mut connections[idx];

            // Determine which message type array we need to modify.
            let message_flows = match msg_type {
                MsgType::Cmd => &mut connection.cmd,
                MsgType::Data => &mut connection.data,
                MsgType::AudioFrame => &mut connection.audio_frame,
                MsgType::VideoFrame => &mut connection.video_frame,
            };

            // If the message flows array exists, find and remove the specific
            // message flow.
            if let Some(flows) = message_flows {
                let flow_idx = flows.iter().position(|flow| {
                    flow.name == msg_name
                        && flow.dest.iter().any(|dest| {
                            dest.app == dest_app
                                && dest.extension == dest_extension
                        })
                });

                if let Some(flow_idx) = flow_idx {
                    // If there are multiple destinations for this message, we
                    // need to remove only the specific destination.
                    let flow = &mut flows[flow_idx];

                    // Find the destination to remove.
                    let dest_idx = flow.dest.iter().position(|dest| {
                        dest.app == dest_app && dest.extension == dest_extension
                    });

                    if let Some(dest_idx) = dest_idx {
                        // Remove the specific destination.
                        flow.dest.remove(dest_idx);

                        // If there are no more destinations, remove the whole
                        // flow.
                        if flow.dest.is_empty() {
                            flows.remove(flow_idx);
                        }

                        // If there are no more flows of this message type, set
                        // the array to None.
                        if flows.is_empty() {
                            *message_flows = None;
                        }

                        // If there are no message flows left in this
                        // connection, remove the connection.
                        if connection.cmd.is_none()
                            && connection.data.is_none()
                            && connection.audio_frame.is_none()
                            && connection.video_frame.is_none()
                        {
                            connections.remove(idx);
                        }

                        // If there are no more connections, set the connections
                        // field to None.
                        if connections.is_empty() {
                            self.connections = None;
                        }

                        return Ok(());
                    }
                }
            }
        }

        // Connection, message flow, or destination not found.
        Err(anyhow!("Connection not found in the graph"))
    }
}
