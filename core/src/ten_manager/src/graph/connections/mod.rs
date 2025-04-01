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

use ten_rust::graph::connection::GraphConnection;
use ten_rust::pkg_info::constants::PROPERTY_JSON_FILENAME;

/// Update the connections of a graph in the property.json file.
///
/// This function updates the connections in the property.json file for a
/// specified graph. It replaces all existing connections with the provided new
/// connections.
pub fn update_graph_connections_all_fields(
    pkg_url: &str,
    property_all_fields: &mut serde_json::Map<String, Value>,
    graph_name: &str,
    new_connections: &Option<Vec<GraphConnection>>,
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
                            // Found the matching graph, now update its
                            // connections.

                            // If there are no new connections, remove the
                            // connections array if it exists
                            if new_connections.is_none()
                                || new_connections.as_ref().unwrap().is_empty()
                            {
                                graph_obj.remove("connections");
                            } else {
                                // Convert the connections to JSON Value
                                if let Ok(connections_value) =
                                    serde_json::to_value(new_connections)
                                {
                                    // Update or create the connections array
                                    graph_obj.insert(
                                        "connections".to_string(),
                                        connections_value,
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
