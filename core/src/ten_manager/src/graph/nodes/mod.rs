//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::fs::OpenOptions;
use std::path::Path;
use std::str::FromStr;

use anyhow::{Context, Result};
use serde_json::Value;

use ten_rust::graph::graph_info::GraphInfo;
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
    nodes_to_modify_property: Option<&[GraphNode]>,
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
                                        // Filter out nodes to remove based on
                                        // key fields only, ignoring property.
                                        nodes_array.retain(|item| {
                                            // For each node in the array, check if it matches any node in nodes_to_remove.
                                            if let Value::Object(item_obj) = item {
                                                // For each node to remove, check if the current node matches.
                                                !remove_nodes.iter().any(|remove_node| {
                                                    // Match the type.
                                                    let type_match = match item_obj.get("type") {
                                                        Some(Value::String(item_type)) => {
                                                            if let Ok(pkg_type) = ten_rust::pkg_info::pkg_type::PkgType::from_str(item_type) {
                                                                pkg_type == remove_node.type_and_name.pkg_type
                                                            } else {
                                                                false
                                                            }
                                                        },
                                                        _ => false
                                                    };

                                                    // Match the name.
                                                    let name_match = match item_obj.get("name") {
                                                        Some(Value::String(item_name)) =>
                                                            item_name == &remove_node.type_and_name.name,
                                                        _ => false
                                                    };

                                                    // Match the addon.
                                                    let addon_match = match item_obj.get("addon") {
                                                        Some(Value::String(item_addon)) =>
                                                            item_addon == &remove_node.addon,
                                                        _ => false
                                                    };

                                                    // Match the extension_group if it exists in the node to remove.
                                                    let extension_group_match = match (&remove_node.extension_group, item_obj.get("extension_group")) {
                                                        (Some(group), Some(Value::String(item_group))) =>
                                                            group == item_group,
                                                        (None, None) => true,
                                                         // Node to remove doesn't specify extension_group.
                                                        (None, Some(_)) => false,
                                                        // Node to remove has extension_group but item doesn't.
                                                        (Some(_), None) => false,
                                                        // Other cases (like mismatched types) don't match.
                                                        _ => false,
                                                    };

                                                    // Match the app if it exists in the node to remove.
                                                    let app_match = match (&remove_node.app, item_obj.get("app")) {
                                                        (Some(app), Some(Value::String(item_app))) =>
                                                            app == item_app,
                                                        (None, None) => true,
                                                        // Node to remove doesn't specify app.
                                                        (None, Some(_)) => false,
                                                        // Node to remove has app but item doesn't.
                                                        (Some(_), None) => false,
                                                        // Other cases (like mismatched types) don't match.
                                                        _ => false,
                                                    };

                                                    // All fields match.
                                                    type_match && name_match && addon_match && extension_group_match && app_match
                                                })
                                            } else {
                                                // Keep non-object values.
                                                true
                                            }
                                        });
                                    }
                                }

                                // Modify node properties if requested.
                                if let Some(modify_nodes) =
                                    nodes_to_modify_property
                                {
                                    if !modify_nodes.is_empty() {
                                        // For each node in the graph.
                                        for node_value in nodes_array.iter_mut()
                                        {
                                            if let Value::Object(node_obj) =
                                                node_value
                                            {
                                                // Check against each node to
                                                // modify.
                                                for modify_node in modify_nodes
                                                {
                                                    // Only proceed if the node
                                                    // has a property to apply.
                                                    if modify_node
                                                        .property
                                                        .is_none()
                                                    {
                                                        continue;
                                                    }

                                                    // Match the type.
                                                    let type_match = match node_obj.get("type") {
                                                        Some(Value::String(node_type)) => {
                                                            if let Ok(pkg_type) = ten_rust::pkg_info::pkg_type::PkgType::from_str(node_type) {
                                                                pkg_type == modify_node.type_and_name.pkg_type
                                                            } else {
                                                                false
                                                            }
                                                        },
                                                        _ => false
                                                    };

                                                    // Match the name.
                                                    let name_match = match node_obj.get("name") {
                                                        Some(Value::String(node_name)) =>
                                                            node_name == &modify_node.type_and_name.name,
                                                        _ => false
                                                    };

                                                    // Match the addon.
                                                    let addon_match =
                                                        match node_obj
                                                            .get("addon")
                                                        {
                                                            Some(
                                                                Value::String(
                                                                    node_addon,
                                                                ),
                                                            ) => node_addon
                                                                == &modify_node
                                                                    .addon,
                                                            _ => false,
                                                        };

                                                    // Match the extension_group.
                                                    let extension_group_match = match (&modify_node.extension_group, node_obj.get("extension_group")) {
                                                        (Some(group), Some(Value::String(node_group))) =>
                                                            group == node_group,
                                                        (None, None) => true,
                                                        // Modified node doesn't specify extension_group but graph node does.
                                                        (None, Some(_)) => false,
                                                        // Modified node has extension_group but graph node doesn't.
                                                        (Some(_), None) => false,
                                                        // Other cases (like mismatched types) don't match.
                                                        _ => false,
                                                    };

                                                    // Match the app.
                                                    let app_match = match (
                                                        &modify_node.app,
                                                        node_obj.get("app"),
                                                    ) {
                                                        (
                                                            Some(app),
                                                            Some(
                                                                Value::String(
                                                                    node_app,
                                                                ),
                                                            ),
                                                        ) => app == node_app,
                                                        (None, None) => true,
                                                        // Modified node doesn't
                                                        // specify app but graph
                                                        // node does.
                                                        (None, Some(_)) => {
                                                            false
                                                        }
                                                        // Modified node has app
                                                        // but graph node
                                                        // doesn't.
                                                        (Some(_), None) => {
                                                            false
                                                        }
                                                        // Other cases (like
                                                        // mismatched types)
                                                        // don't match.
                                                        _ => false,
                                                    };

                                                    // If all fields match,
                                                    // update the property.
                                                    if type_match
                                                        && name_match
                                                        && addon_match
                                                        && extension_group_match
                                                        && app_match
                                                    {
                                                        if let Some(property) =
                                                            &modify_node
                                                                .property
                                                        {
                                                            node_obj.insert(
                                                                "property"
                                                                    .to_string(
                                                                    ),
                                                                property
                                                                    .clone(),
                                                            );
                                                        }

                                                        // No need to check
                                                        // further modify_nodes
                                                        // for this graph node.
                                                        break;
                                                    }
                                                }
                                            }
                                        }
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

pub fn extension_graph_node_find_by_loc<'a>(
    graph_info: &'a GraphInfo,
    app: Option<&String>,
    extension: &String,
) -> Option<&'a GraphNode> {
    graph_info.graph.nodes.iter().find(|node| {
        node.app.as_ref() == app && node.type_and_name.name == *extension
    })
}
