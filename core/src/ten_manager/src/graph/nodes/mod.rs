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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Map, Value};
    use std::fs;
    use tempfile::TempDir;
    use ten_rust::graph::node::GraphNode;
    use ten_rust::pkg_info::pkg_type::PkgType;
    use ten_rust::pkg_info::pkg_type_and_name::PkgTypeAndName;

    #[test]
    fn test_update_graph_node_preserves_field_order() -> Result<()> {
        // Create a temporary directory for our test.
        let temp_dir = TempDir::new()?;
        let temp_path = temp_dir.path().to_str().unwrap().to_string();

        // Create a property with fields in a specific order.
        let mut all_fields = Map::new();
        all_fields.insert("name".to_string(), json!("test-app"));
        all_fields.insert("version".to_string(), json!("1.0.0"));
        all_fields.insert("description".to_string(), json!("Test App"));

        // Create _ten field with predefined_graphs.
        let mut ten_obj = Map::new();
        let mut graphs = Vec::new();

        // Create a graph with nodes.
        let mut graph1 = Map::new();
        graph1.insert("name".to_string(), json!("test-graph"));
        graph1.insert("auto_start".to_string(), json!(true));

        // Initial nodes with specific order.
        let mut nodes = Vec::new();
        nodes.push(json!({
            "type": "extension",
            "name": "first-node",
            "addon": "first-addon"
        }));
        nodes.push(json!({
            "type": "extension",
            "name": "second-node",
            "addon": "second-addon"
        }));
        nodes.push(json!({
            "type": "extension",
            "name": "third-node",
            "addon": "third-addon"
        }));
        graph1.insert("nodes".to_string(), Value::Array(nodes));

        // Add connections array with connections between the nodes.
        let mut connections = Vec::new();

        // Add a connection from first-node to second-node.
        connections.push(json!({
            "extension": "first-node",
            "cmd": [{
                "name": "test-cmd",
                "dest": [{
                    "extension": "second-node"
                }]
            }]
        }));

        // Add a connection from second-node to third-node and first-node.
        connections.push(json!({
            "extension": "second-node",
            "cmd": [{
                "name": "test-cmd",
                "dest": [{
                    "extension": "third-node"
                }, {
                    "extension": "first-node"
                }]
            }]
        }));

        // Add a connection from third-node to first-node.
        connections.push(json!({
            "extension": "third-node",
            "data": [{
                "name": "test-data",
                "dest": [{
                    "extension": "first-node"
                }]
            }]
        }));

        graph1.insert("connections".to_string(), Value::Array(connections));

        graphs.push(Value::Object(graph1));
        ten_obj.insert("predefined_graphs".to_string(), Value::Array(graphs));
        all_fields.insert("_ten".to_string(), Value::Object(ten_obj));

        // Add more fields after _ten.
        all_fields.insert("license".to_string(), json!("Apache-2.0"));
        all_fields.insert("author".to_string(), json!("Test Author"));

        // Write the initial property to the temp directory.
        let property_path = Path::new(&temp_path).join(PROPERTY_JSON_FILENAME);
        let property_file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&property_path)?;
        serde_json::to_writer_pretty(property_file, &all_fields)?;

        // Create a new node to add.
        let new_node = GraphNode {
            type_and_name: PkgTypeAndName {
                pkg_type: PkgType::Extension,
                name: "new-node".to_string(),
            },
            addon: "new-addon".to_string(),
            extension_group: None,
            app: None,
            property: None,
        };
        let new_nodes = vec![new_node];

        // Create a node to remove (the second one).
        let remove_node = GraphNode {
            type_and_name: PkgTypeAndName {
                pkg_type: PkgType::Extension,
                name: "second-node".to_string(),
            },
            addon: "second-addon".to_string(),
            extension_group: None,
            app: None,
            property: None,
        };
        let remove_nodes = vec![remove_node];

        // Update the property: add one node and remove one node in a single
        // call.
        update_graph_node_all_fields(
            &temp_path,
            &mut all_fields,
            "test-graph",
            Some(&new_nodes),
            Some(&remove_nodes),
        )?;

        // Read the updated property.json file back in to verify changes.
        let updated_property_json = fs::read_to_string(&property_path)?;
        let updated_property: Map<String, Value> =
            serde_json::from_str(&updated_property_json)?;

        // Verify order of top-level fields.
        let field_names: Vec<&String> = updated_property.keys().collect();
        assert_eq!(field_names[0], "name");
        assert_eq!(field_names[1], "version");
        assert_eq!(field_names[2], "description");
        assert_eq!(field_names[3], "_ten");
        assert_eq!(field_names[4], "license");
        assert_eq!(field_names[5], "author");

        // Verify nodes were updated correctly.
        let updated_graph = &updated_property["_ten"]["predefined_graphs"][0];
        let updated_nodes = updated_graph["nodes"].as_array().unwrap();

        // Check nodes order (first, third, new).
        assert_eq!(updated_nodes.len(), 3);
        assert_eq!(updated_nodes[0]["name"], "first-node");
        assert_eq!(updated_nodes[1]["name"], "third-node");
        assert_eq!(updated_nodes[2]["name"], "new-node");

        // Verify connections were updated correctly.
        let updated_connections =
            updated_graph["connections"].as_array().unwrap();

        // We should now have 1 connection instead of 3 (the third-node
        // connection is the only one left).
        assert_eq!(updated_connections.len(), 1);

        // The remaining connection should be from third-node.
        let third_conn = &updated_connections[0];
        assert_eq!(third_conn["extension"], "third-node");
        assert!(third_conn["data"].as_array().is_some());
        assert_eq!(third_conn["data"][0]["name"], "test-data");
        assert_eq!(third_conn["data"][0]["dest"][0]["extension"], "first-node");

        // The connections from first-node and second-node should be gone
        // - first-node connection had only second-node as destination, so it's
        //   removed.
        // - second-node connection was removed explicitly.

        Ok(())
    }

    #[test]
    fn test_update_graph_node_removes_all_connections() -> Result<()> {
        // Create a temporary directory for our test.
        let temp_dir = TempDir::new()?;
        let temp_path = temp_dir.path().to_str().unwrap().to_string();

        // Create a property with a graph that has nodes and connections.
        let mut all_fields = Map::new();
        all_fields.insert("name".to_string(), json!("test-app"));

        // Create _ten field with predefined_graphs.
        let mut ten_obj = Map::new();
        let mut graphs = Vec::new();

        // Create a graph with nodes.
        let mut graph1 = Map::new();
        graph1.insert("name".to_string(), json!("test-graph"));

        // Create nodes.
        let mut nodes = Vec::new();
        nodes.push(json!({
            "type": "extension",
            "name": "node1",
            "addon": "addon1"
        }));
        nodes.push(json!({
            "type": "extension",
            "name": "node2",
            "addon": "addon2"
        }));
        graph1.insert("nodes".to_string(), Value::Array(nodes));

        // Create connections where node1 and node2 both connect to each other.
        let mut connections = Vec::new();
        connections.push(json!({
            "extension": "node1",
            "cmd": [{
                "name": "cmd1",
                "dest": [{
                    "extension": "node2"
                }]
            }]
        }));
        connections.push(json!({
            "extension": "node2",
            "data": [{
                "name": "data1",
                "dest": [{
                    "extension": "node1"
                }]
            }]
        }));
        graph1.insert("connections".to_string(), Value::Array(connections));

        graphs.push(Value::Object(graph1));
        ten_obj.insert("predefined_graphs".to_string(), Value::Array(graphs));
        all_fields.insert("_ten".to_string(), Value::Object(ten_obj));

        // Write the initial property to the temp directory.
        let property_path = Path::new(&temp_path).join(PROPERTY_JSON_FILENAME);
        let property_file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&property_path)?;
        serde_json::to_writer_pretty(property_file, &all_fields)?;

        // Create nodes to remove (both node1 and node2).
        let remove_nodes = vec![
            GraphNode {
                type_and_name: PkgTypeAndName {
                    pkg_type: PkgType::Extension,
                    name: "node1".to_string(),
                },
                addon: "addon1".to_string(),
                extension_group: None,
                app: None,
                property: None,
            },
            GraphNode {
                type_and_name: PkgTypeAndName {
                    pkg_type: PkgType::Extension,
                    name: "node2".to_string(),
                },
                addon: "addon2".to_string(),
                extension_group: None,
                app: None,
                property: None,
            },
        ];

        // Update the property: remove both nodes.
        update_graph_node_all_fields(
            &temp_path,
            &mut all_fields,
            "test-graph",
            None,
            Some(&remove_nodes),
        )?;

        // Read the updated property.json file back in to verify changes.
        let updated_property_json = fs::read_to_string(&property_path)?;
        let updated_property: Map<String, Value> =
            serde_json::from_str(&updated_property_json)?;

        // Verify nodes were removed.
        let updated_graph = &updated_property["_ten"]["predefined_graphs"][0];
        let updated_nodes = updated_graph["nodes"].as_array().unwrap();
        assert_eq!(updated_nodes.len(), 0);

        // Verify connections field is completely removed.
        assert!(!updated_graph
            .as_object()
            .unwrap()
            .contains_key("connections"));

        Ok(())
    }

    #[test]
    fn test_update_graph_node_removes_connection_destinations() -> Result<()> {
        // Create a temporary directory for our test.
        let temp_dir = TempDir::new()?;
        let temp_path = temp_dir.path().to_str().unwrap().to_string();

        // Create a property with a graph that has nodes and connections.
        let mut all_fields = Map::new();
        all_fields.insert("name".to_string(), json!("test-app"));

        // Create _ten field with predefined_graphs.
        let mut ten_obj = Map::new();
        let mut graphs = Vec::new();

        // Create a graph with nodes.
        let mut graph1 = Map::new();
        graph1.insert("name".to_string(), json!("test-graph"));

        // Create nodes.
        let mut nodes = Vec::new();
        nodes.push(json!({
            "type": "extension",
            "name": "node1",
            "addon": "addon1"
        }));
        nodes.push(json!({
            "type": "extension",
            "name": "node2",
            "addon": "addon2"
        }));
        nodes.push(json!({
            "type": "extension",
            "name": "node3",
            "addon": "addon3"
        }));
        graph1.insert("nodes".to_string(), Value::Array(nodes));

        // Create a connection with multiple message types and destinations.
        let mut connections = Vec::new();
        connections.push(json!({
            "extension": "node1",
            "cmd": [{
                "name": "cmd1",
                "dest": [
                    { "extension": "node2" },
                    { "extension": "node3" }
                ]
            }],
            "data": [{
                "name": "data1",
                "dest": [
                    { "extension": "node2" }
                ]
            }]
        }));

        // Add a connection with all message types pointing to node2.
        connections.push(json!({
            "extension": "node3",
            "cmd": [{
                "name": "cmd3",
                "dest": [{ "extension": "node2" }]
            }],
            "data": [{
                "name": "data3",
                "dest": [{ "extension": "node2" }]
            }],
            "audio_frame": [{
                "name": "audio3",
                "dest": [{ "extension": "node2" }]
            }],
            "video_frame": [{
                "name": "video3",
                "dest": [{ "extension": "node2" }]
            }]
        }));

        graph1.insert("connections".to_string(), Value::Array(connections));

        graphs.push(Value::Object(graph1));
        ten_obj.insert("predefined_graphs".to_string(), Value::Array(graphs));
        all_fields.insert("_ten".to_string(), Value::Object(ten_obj));

        // Write the initial property to the temp directory.
        let property_path = Path::new(&temp_path).join(PROPERTY_JSON_FILENAME);
        let property_file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&property_path)?;
        serde_json::to_writer_pretty(property_file, &all_fields)?;

        // Create a node to remove (node2).
        let remove_node = GraphNode {
            type_and_name: PkgTypeAndName {
                pkg_type: PkgType::Extension,
                name: "node2".to_string(),
            },
            addon: "addon2".to_string(),
            extension_group: None,
            app: None,
            property: None,
        };
        let remove_nodes = vec![remove_node];

        // Update the property: remove node2.
        update_graph_node_all_fields(
            &temp_path,
            &mut all_fields,
            "test-graph",
            None,
            Some(&remove_nodes),
        )?;

        // Read the updated property.json file back in to verify changes.
        let updated_property_json = fs::read_to_string(&property_path)?;
        let updated_property: Map<String, Value> =
            serde_json::from_str(&updated_property_json)?;

        // Verify nodes were updated.
        let updated_graph = &updated_property["_ten"]["predefined_graphs"][0];
        let updated_nodes = updated_graph["nodes"].as_array().unwrap();
        assert_eq!(updated_nodes.len(), 2);

        // Verify connections were updated.
        let updated_connections =
            updated_graph["connections"].as_array().unwrap();

        // There should be 1 connection left (from node1 to node3).
        assert_eq!(updated_connections.len(), 1);
        assert_eq!(updated_connections[0]["extension"], "node1");

        // Check that the cmd flow only has node3 as destination.
        let cmd_flows = updated_connections[0]["cmd"].as_array().unwrap();
        assert_eq!(cmd_flows.len(), 1);
        let cmd_dests = cmd_flows[0]["dest"].as_array().unwrap();
        assert_eq!(cmd_dests.len(), 1);
        assert_eq!(cmd_dests[0]["extension"], "node3");

        // The data flow should be removed since its only destination was node2.
        assert!(!updated_connections[0]
            .as_object()
            .unwrap()
            .contains_key("data"));

        // The connection from node3 should be completely removed since all its
        // destinations were to node2.
        assert_eq!(updated_connections.len(), 1);

        Ok(())
    }
}
