//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::fs::OpenOptions;
use std::path::Path;

use anyhow::{Context, Result};
use serde_json::Value;

use ten_rust::graph::node::GraphNode;
use ten_rust::pkg_info::constants::PROPERTY_JSON_FILENAME;

/// Update a graph node in the property.json file, handling both adding new
/// nodes and removing existing ones.
///
/// Both `nodes_to_add` and `nodes_to_remove` are optional, allowing for:
///   - Adding new nodes without removing any.
///   - Removing nodes without adding any new ones.
///   - Both adding and removing nodes in a single call.
///
/// In all cases, the original order of entries in the property.json file is
/// preserved.
pub fn update_graph_node_all_fields(
    pkg_url: &str,
    property_all_fields: &mut serde_json::Map<String, Value>,
    graph_name: &str,
    nodes_to_add: Option<&[GraphNode]>,
    nodes_to_remove: Option<&[GraphNode]>,
) -> Result<()> {
    // Process _ten.predefined_graphs array.
    if let Some(Value::Object(ten_obj)) = property_all_fields.get_mut("_ten") {
        if let Some(Value::Array(predefined_graphs)) =
            ten_obj.get_mut("predefined_graphs")
        {
            // Find the graph with the matching name.
            for graph_value in predefined_graphs.iter_mut() {
                if let Value::Object(graph_obj) = graph_value {
                    if let Some(Value::String(name)) = graph_obj.get("name") {
                        if name == graph_name {
                            // Found the matching graph, now update its nodes.
                            if let Some(Value::Array(nodes_array)) =
                                graph_obj.get_mut("nodes")
                            {
                                // Remove nodes if requested.
                                if let Some(remove_nodes) = nodes_to_remove {
                                    if !remove_nodes.is_empty() {
                                        // First, convert nodes to remove into
                                        // comparable form.
                                        let nodes_to_remove_serialized: Vec<
                                            String,
                                        > = remove_nodes
                                            .iter()
                                            .filter_map(|node| {
                                                let node_value =
                                                    serde_json::to_value(node)
                                                        .ok()?;
                                                serde_json::to_string(
                                                    &node_value,
                                                )
                                                .ok()
                                            })
                                            .collect();

                                        // Filter out nodes to remove.
                                        nodes_array.retain(|item| {
                                            if let Ok(item_str) =
                                                serde_json::to_string(item)
                                            {
                                                !nodes_to_remove_serialized
                                                    .contains(&item_str)
                                            } else {
                                                // Keep items that can't be
                                                // serialized.
                                                true
                                            }
                                        });
                                    }
                                }

                                // Add new nodes if provided.
                                if let Some(new_nodes) = nodes_to_add {
                                    if !new_nodes.is_empty() {
                                        // Append new nodes to the existing
                                        // array.
                                        for new_node in new_nodes {
                                            if let Ok(new_node_value) =
                                                serde_json::to_value(new_node)
                                            {
                                                nodes_array
                                                    .push(new_node_value);
                                            }
                                        }
                                    }
                                }
                            } else {
                                // No nodes array in the graph yet, create one
                                // if we have nodes to add.
                                if let Some(new_nodes) = nodes_to_add {
                                    if !new_nodes.is_empty() {
                                        let mut nodes_array = Vec::new();

                                        // Add all new nodes to the array.
                                        for new_node in new_nodes {
                                            if let Ok(new_node_value) =
                                                serde_json::to_value(new_node)
                                            {
                                                nodes_array
                                                    .push(new_node_value);
                                            }
                                        }

                                        graph_obj.insert(
                                            "nodes".to_string(),
                                            Value::Array(nodes_array),
                                        );
                                    }
                                }
                            }

                            // Process connections if nodes were removed.
                            if let Some(remove_nodes) = nodes_to_remove {
                                if !remove_nodes.is_empty() {
                                    // Now handle connections - done after nodes
                                    // to avoid borrow issues.
                                    update_connections_for_removed_nodes(
                                        graph_obj,
                                        remove_nodes,
                                    );
                                }
                            }

                            // We've found and updated the graph, no need to
                            // continue.
                            break;
                        }
                    }
                }
            }
        }
    }

    // Write the updated property back to the file.
    let property_path = Path::new(pkg_url).join(PROPERTY_JSON_FILENAME);
    let property_file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(property_path)
        .context("Failed to open property.json file")?;

    // Serialize the property_all_fields map directly to preserve field order.
    serde_json::to_writer_pretty(property_file, &property_all_fields)
        .context("Failed to write to property.json file")?;

    Ok(())
}

/// Helper function to update connections after nodes are removed.
fn update_connections_for_removed_nodes(
    graph_obj: &mut serde_json::Map<String, Value>,
    remove_nodes: &[GraphNode],
) {
    // Create a list of node identifiers to remove.
    let nodes_names_to_remove: Vec<(String, Option<String>)> = remove_nodes
        .iter()
        .map(|node| (node.type_and_name.name.clone(), node.app.clone()))
        .collect();

    if let Some(Value::Array(connections_array)) =
        graph_obj.get_mut("connections")
    {
        // 1. Remove entire connections with matching app and extension.
        connections_array.retain(|connection| {
            if let Value::Object(conn_obj) = connection {
                if let (Some(Value::String(extension)), app_value) =
                    (conn_obj.get("extension"), conn_obj.get("app"))
                {
                    let app = if let Some(Value::String(app_str)) = app_value {
                        Some(app_str.clone())
                    } else {
                        None
                    };

                    // Check if this connection's extension matches any node to
                    // remove.
                    !nodes_names_to_remove.iter().any(|(name, node_app)| {
                        name == extension && node_app == &app
                    })
                } else {
                    true // Keep connections without extension field.
                }
            } else {
                true // Keep non-object values.
            }
        });

        // 2. Remove destinations from message flows in remaining connections.
        for connection in connections_array.iter_mut() {
            if let Value::Object(conn_obj) = connection {
                // Process each type of message flow (cmd, data, audio_frame,
                // video_frame).
                for flow_type in &["cmd", "data", "audio_frame", "video_frame"]
                {
                    if let Some(Value::Array(flows)) =
                        conn_obj.get_mut(*flow_type)
                    {
                        // For each flow in this type.
                        for flow in flows.iter_mut() {
                            if let Value::Object(flow_obj) = flow {
                                // Get the destinations of this flow.
                                if let Some(Value::Array(dest_array)) =
                                    flow_obj.get_mut("dest")
                                {
                                    // Remove destinations referring to removed
                                    // nodes.
                                    dest_array.retain(|dest| {
                                        if let Value::Object(dest_obj) = dest {
                                            if let (
                                                Some(Value::String(extension)),
                                                app_value,
                                            ) = (
                                                dest_obj.get("extension"),
                                                dest_obj.get("app"),
                                            ) {
                                                let app = if let Some(
                                                    Value::String(app_str),
                                                ) = app_value
                                                {
                                                    Some(app_str.clone())
                                                } else {
                                                    None
                                                };

                                                // Keep if not in the nodes to
                                                // remove list.
                                                !nodes_names_to_remove
                                                    .iter()
                                                    .any(|(name, node_app)| {
                                                        name == extension
                                                            && node_app == &app
                                                    })
                                            } else {
                                                // Keep destinations without
                                                // extension field.
                                                true
                                            }
                                        } else {
                                            // Keep non-object values.
                                            true
                                        }
                                    });
                                }
                            }
                        }

                        // Remove empty flows (flows with no destinations).
                        flows.retain(|flow| {
                            if let Value::Object(flow_obj) = flow {
                                if let Some(Value::Array(dest_array)) =
                                    flow_obj.get("dest")
                                {
                                    !dest_array.is_empty()
                                } else {
                                    true // Keep flows without dest field.
                                }
                            } else {
                                true // Keep non-object values.
                            }
                        });

                        // If the flow array is now empty, remove it from the
                        // connection.
                        if flows.is_empty() {
                            conn_obj.remove(*flow_type);
                        }
                    }
                }
            }
        }

        // Remove empty connections (those with no message flows).
        connections_array.retain(|connection| {
            if let Value::Object(conn_obj) = connection {
                // Check if any message flow exists and is non-empty.
                let has_cmd = conn_obj.get("cmd").is_some_and(|v| {
                    if let Value::Array(arr) = v {
                        !arr.is_empty()
                    } else {
                        false
                    }
                });
                let has_data = conn_obj.get("data").is_some_and(|v| {
                    if let Value::Array(arr) = v {
                        !arr.is_empty()
                    } else {
                        false
                    }
                });
                let has_audio = conn_obj.get("audio_frame").is_some_and(|v| {
                    if let Value::Array(arr) = v {
                        !arr.is_empty()
                    } else {
                        false
                    }
                });
                let has_video = conn_obj.get("video_frame").is_some_and(|v| {
                    if let Value::Array(arr) = v {
                        !arr.is_empty()
                    } else {
                        false
                    }
                });

                has_cmd || has_data || has_audio || has_video
            } else {
                true // Keep non-object values.
            }
        });

        // If all connections are removed, remove the connections array.
        if connections_array.is_empty() {
            graph_obj.remove("connections");
        }
    }
}
