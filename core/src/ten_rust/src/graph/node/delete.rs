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
