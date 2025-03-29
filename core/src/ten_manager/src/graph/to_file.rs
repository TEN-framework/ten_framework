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
    nodes_to_add: Option<&Vec<GraphNode>>,
    nodes_to_remove: Option<&Vec<GraphNode>>,
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Map, Value};
    use std::fs;
    use std::io::Read;
    use tempfile::TempDir;
    use ten_rust::graph::node::GraphNode;
    use ten_rust::pkg_info::pkg_type::PkgType;
    use ten_rust::pkg_info::pkg_type_and_name::PkgTypeAndName;

    #[test]
    fn test_update_graph_node_preserves_field_order() -> Result<()> {
        // Create a temporary directory for our test
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

        // Add empty connections array.
        graph1.insert("connections".to_string(), json!([]));

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

        // Read the updated property file.
        let mut updated_property_content = String::new();
        let mut file = fs::File::open(&property_path)?;
        file.read_to_string(&mut updated_property_content)?;

        // Parse the updated property.
        let updated_property: serde_json::Value =
            serde_json::from_str(&updated_property_content)?;

        // Verify the field order is maintained.
        if let Value::Object(map) = updated_property {
            let keys: Vec<&String> = map.keys().collect();

            // Check that the order of fields matches our original order.
            let expected_order = [
                "name",
                "version",
                "description",
                "_ten",
                "license",
                "author",
            ];

            for (i, expected_key) in expected_order.iter().enumerate() {
                assert_eq!(
                    keys[i], expected_key,
                    "Field order not preserved at position {}",
                    i
                );
            }

            // Verify the nodes were updated correctly and order is preserved.
            if let Value::Object(ten) = &map["_ten"] {
                if let Value::Array(graphs) = &ten["predefined_graphs"] {
                    if let Value::Object(graph) = &graphs[0] {
                        if let Value::Array(nodes) = &graph["nodes"] {
                            assert_eq!(nodes.len(), 3, "Should have 3 nodes");

                            // Check the first node is still first.
                            if let Value::Object(first_node) = &nodes[0] {
                                assert_eq!(
                                    first_node["name"], "first-node",
                                    "First node order not preserved"
                                );
                            } else {
                                panic!("First node is not an object");
                            }

                            // Check that second node is now the third one
                            // (since second was removed).
                            if let Value::Object(second_node) = &nodes[1] {
                                assert_eq!(
                                    second_node["name"], "third-node",
                                    "Third node should now be in position 2"
                                );
                            } else {
                                panic!("Second node position is not an object");
                            }

                            // Check that the new node is at the end.
                            if let Value::Object(third_node) = &nodes[2] {
                                assert_eq!(
                                    third_node["name"], "new-node",
                                    "New node should be at the end"
                                );
                            } else {
                                panic!("Third node position is not an object");
                            }
                        } else {
                            panic!("Nodes is not an array");
                        }
                    } else {
                        panic!("Graph is not an object");
                    }
                } else {
                    panic!("Predefined_graphs is not an array");
                }
            } else {
                panic!("_ten is not an object");
            }
        } else {
            panic!("Updated property is not an object");
        }

        // Test removing all nodes.
        let all_remove_nodes = vec![
            GraphNode {
                type_and_name: PkgTypeAndName {
                    pkg_type: PkgType::Extension,
                    name: "first-node".to_string(),
                },
                addon: "first-addon".to_string(),
                extension_group: None,
                app: None,
                property: None,
            },
            GraphNode {
                type_and_name: PkgTypeAndName {
                    pkg_type: PkgType::Extension,
                    name: "third-node".to_string(),
                },
                addon: "third-addon".to_string(),
                extension_group: None,
                app: None,
                property: None,
            },
            GraphNode {
                type_and_name: PkgTypeAndName {
                    pkg_type: PkgType::Extension,
                    name: "new-node".to_string(),
                },
                addon: "new-addon".to_string(),
                extension_group: None,
                app: None,
                property: None,
            },
        ];

        // Update the property to remove all nodes.
        update_graph_node_all_fields(
            &temp_path,
            &mut all_fields,
            "test-graph",
            None,
            Some(&all_remove_nodes),
        )?;

        // Read the updated property file again.
        let mut updated_property_content = String::new();
        let mut file = fs::File::open(&property_path)?;
        file.read_to_string(&mut updated_property_content)?;

        // Parse the updated property.
        let updated_property: serde_json::Value =
            serde_json::from_str(&updated_property_content)?;

        // Verify the field order is still maintained and nodes are empty.
        if let Value::Object(map) = updated_property {
            let keys: Vec<&String> = map.keys().collect();

            // Check that the order of fields matches our original order.
            let expected_order = [
                "name",
                "version",
                "description",
                "_ten",
                "license",
                "author",
            ];

            for (i, expected_key) in expected_order.iter().enumerate() {
                assert_eq!(
                    keys[i], expected_key,
                    "Field order not preserved after removal at position {}",
                    i
                );
            }

            // Verify nodes array is empty.
            if let Value::Object(ten) = &map["_ten"] {
                if let Value::Array(graphs) = &ten["predefined_graphs"] {
                    if let Value::Object(graph) = &graphs[0] {
                        if let Value::Array(nodes) = &graph["nodes"] {
                            assert_eq!(
                                nodes.len(),
                                0,
                                "All nodes should be removed"
                            );
                        } else {
                            panic!("Nodes is not an array");
                        }
                    } else {
                        panic!("Graph is not an object");
                    }
                } else {
                    panic!("Predefined_graphs is not an array");
                }
            } else {
                panic!("_ten is not an object");
            }
        } else {
            panic!("Updated property is not an object");
        }

        Ok(())
    }
}
