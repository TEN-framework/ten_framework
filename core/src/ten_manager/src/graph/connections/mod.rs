//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
pub mod add;
pub mod validate;

use std::{fs::OpenOptions, path::Path};

use anyhow::{Context, Result};
use serde_json::Value;

use ten_rust::{
    graph::connection::{GraphConnection, GraphMessageFlow},
    pkg_info::constants::PROPERTY_JSON_FILENAME,
};

/// Update the connections of a graph in the property.json file.
///
/// This function updates the connections in the property.json file for a
/// specified graph.
///
/// In all cases, the original order of connections in the property.json file is
/// preserved.
pub fn update_graph_connections_all_fields(
    pkg_url: &str,
    property_all_fields: &mut serde_json::Map<String, Value>,
    graph_name: &str,
    connections_to_add: Option<&[GraphConnection]>,
    connections_to_remove: Option<&[GraphConnection]>,
    connections_to_modify_msg_conversion: Option<&[GraphConnection]>,
) -> Result<()> {
    // Get ten object if it exists.
    let ten_obj = match property_all_fields.get_mut("_ten") {
        Some(Value::Object(obj)) => obj,
        _ => return write_property_file(pkg_url, property_all_fields),
    };

    // Get predefined_graphs array if it exists.
    let predefined_graphs = match ten_obj.get_mut("predefined_graphs") {
        Some(Value::Array(graphs)) => graphs,
        _ => return write_property_file(pkg_url, property_all_fields),
    };

    // Find and update the target graph.
    find_and_update_graph(
        predefined_graphs,
        graph_name,
        connections_to_add,
        connections_to_remove,
        connections_to_modify_msg_conversion,
    );

    // Note: if no graph was found, we still need to write back the property
    // file.
    write_property_file(pkg_url, property_all_fields)
}

/// Find the target graph and update its connections.
/// Returns true if the graph was found and updated.
fn find_and_update_graph(
    predefined_graphs: &mut [Value],
    graph_name: &str,
    connections_to_add: Option<&[GraphConnection]>,
    connections_to_remove: Option<&[GraphConnection]>,
    connections_to_modify_msg_conversion: Option<&[GraphConnection]>,
) -> bool {
    for graph_value in predefined_graphs.iter_mut() {
        let graph_obj = match graph_value {
            Value::Object(obj) => obj,
            _ => continue,
        };

        let name = match graph_obj.get("name") {
            Some(Value::String(name)) => name,
            _ => continue,
        };

        if name != graph_name {
            continue;
        }

        // Found the matching graph, update its connections.
        match graph_obj.get_mut("connections") {
            Some(Value::Array(connections_array)) => {
                // Process existing connections.
                update_existing_connections(
                    connections_array,
                    connections_to_add,
                    connections_to_remove,
                    connections_to_modify_msg_conversion,
                );

                // Remove the connections array if it's empty.
                if connections_array.is_empty() {
                    graph_obj.remove("connections");
                }
            }
            _ => {
                // No connections array, create one if needed.
                create_connections_if_needed(graph_obj, connections_to_add);
            }
        }

        return true; // Graph found and processed.
    }

    false // Graph not found.
}

/// Update existing connections array with modifications.
fn update_existing_connections(
    connections_array: &mut Vec<Value>,
    connections_to_add: Option<&[GraphConnection]>,
    connections_to_remove: Option<&[GraphConnection]>,
    connections_to_modify_msg_conversion: Option<&[GraphConnection]>,
) {
    // First, update message conversions if needed.
    if let Some(modify_connections) = connections_to_modify_msg_conversion {
        if !modify_connections.is_empty() {
            update_message_conversions(connections_array, modify_connections);
        }
    }

    // Then remove connections if requested.
    if let Some(remove_connections) = connections_to_remove {
        if !remove_connections.is_empty() {
            remove_specified_connections(connections_array, remove_connections);
        }
    }

    // Finally add or update connections.
    if let Some(add_connections) = connections_to_add {
        if !add_connections.is_empty() {
            add_or_update_connections(connections_array, add_connections);
        }
    }
}

/// Update message conversions for connections in the array.
fn update_message_conversions(
    connections_array: &mut [Value],
    modify_connections: &[GraphConnection],
) {
    for connection_value in connections_array.iter_mut() {
        let conn_obj = match connection_value {
            Value::Object(obj) => obj,
            _ => continue,
        };

        // Get the app and extension from the current connection.
        let conn_app = conn_obj.get("app").and_then(|v| v.as_str());
        let conn_extension =
            match conn_obj.get("extension").and_then(|v| v.as_str()) {
                Some(ext) => ext,
                None => continue,
            };

        // Find matching connection to modify.
        for modify_conn in modify_connections {
            // Check if app and extension match.
            let app_match = match (conn_app, &modify_conn.app) {
                (None, None) => true,
                (Some(app1), Some(app2)) => app1 == app2,
                _ => false,
            };

            if !app_match || conn_extension != modify_conn.extension {
                continue;
            }

            // Connection matched, update message conversions for each type.
            update_msg_conversion_for_type(
                conn_obj,
                "cmd",
                modify_conn.cmd.as_ref(),
            );
            update_msg_conversion_for_type(
                conn_obj,
                "data",
                modify_conn.data.as_ref(),
            );
            update_msg_conversion_for_type(
                conn_obj,
                "audio_frame",
                modify_conn.audio_frame.as_ref(),
            );
            update_msg_conversion_for_type(
                conn_obj,
                "video_frame",
                modify_conn.video_frame.as_ref(),
            );

            break; // No need to check further modify_connections.
        }
    }
}

/// Remove connections that match those in the remove list.
fn remove_specified_connections(
    connections_array: &mut Vec<Value>,
    remove_connections: &[GraphConnection],
) {
    // Convert connections to remove into comparable form.
    let connections_to_remove_serialized: Vec<String> = remove_connections
        .iter()
        .filter_map(|conn| {
            let conn_value = serde_json::to_value(conn).ok()?;
            serde_json::to_string(&conn_value).ok()
        })
        .collect();

    // Filter out connections to remove.
    connections_array.retain(|item| {
        if let Ok(item_str) = serde_json::to_string(item) {
            !connections_to_remove_serialized.contains(&item_str)
        } else {
            // Keep items that can't be serialized.
            true
        }
    });
}

/// Add new connections or update existing ones.
fn add_or_update_connections(
    connections_array: &mut Vec<Value>,
    add_connections: &[GraphConnection],
) {
    for new_connection in add_connections {
        let new_connection_value = match serde_json::to_value(new_connection) {
            Ok(value) => value,
            Err(_) => continue,
        };

        // Get app and extension values.
        let app_value = new_connection_value.get("app").cloned();
        let extension_value = new_connection_value.get("extension").cloned();

        // Find existing connection index.
        let existing_idx = connections_array.iter().position(|conn| {
            let conn_app = conn.get("app");
            let conn_ext = conn.get("extension");

            // Match by app and extension.
            (app_value.is_none() && conn_app.is_none()
                || app_value.as_ref() == conn_app)
                && (extension_value.as_ref() == conn_ext)
        });

        if let Some(idx) = existing_idx {
            // Update existing connection.
            if let Some(Value::Object(conn_obj)) =
                connections_array.get_mut(idx)
            {
                update_connection_fields(conn_obj, &new_connection_value);
            }
        } else {
            // Add new connection.
            connections_array.push(new_connection_value);
        }
    }
}

/// Update fields in an existing connection object.
fn update_connection_fields(
    conn_obj: &mut serde_json::Map<String, Value>,
    new_connection_value: &Value,
) {
    // Update message type arrays: cmd, data, audio_frame, video_frame.
    update_message_field(conn_obj, new_connection_value, "cmd");
    update_message_field(conn_obj, new_connection_value, "data");
    update_message_field(conn_obj, new_connection_value, "audio_frame");
    update_message_field(conn_obj, new_connection_value, "video_frame");
}

/// Update a specific message field in a connection.
fn update_message_field(
    conn_obj: &mut serde_json::Map<String, Value>,
    new_connection_value: &Value,
    field_name: &str,
) {
    // Check if the field exists in the new connection.
    let field_value = match new_connection_value.get(field_name) {
        Some(value) if value.is_array() => value,
        _ => return, // Skip if not present or not an array.
    };

    match conn_obj.get_mut(field_name) {
        Some(Value::Array(existing_field)) => {
            // Append to existing field array.
            if let Some(new_messages) = field_value.as_array() {
                for msg in new_messages {
                    existing_field.push(msg.clone());
                }
            }
        }
        _ => {
            // No existing field, add it.
            conn_obj.insert(field_name.to_string(), field_value.clone());
        }
    }
}

/// Create a new connections array if needed.
fn create_connections_if_needed(
    graph_obj: &mut serde_json::Map<String, Value>,
    connections_to_add: Option<&[GraphConnection]>,
) {
    // Only create if we have connections to add.
    if let Some(add_connections) = connections_to_add {
        if add_connections.is_empty() {
            return;
        }

        let mut connections_array = Vec::new();

        // Add all new connections to the array.
        for new_connection in add_connections {
            if let Ok(new_connection_value) =
                serde_json::to_value(new_connection)
            {
                connections_array.push(new_connection_value);
            }
        }

        if !connections_array.is_empty() {
            graph_obj.insert(
                "connections".to_string(),
                Value::Array(connections_array),
            );
        }
    }
}

/// Write the property file back to disk.
fn write_property_file(
    pkg_url: &str,
    property_all_fields: &serde_json::Map<String, Value>,
) -> Result<()> {
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

/// Helper function to update message conversion for a specific message type in
/// a connection.
fn update_msg_conversion_for_type(
    conn_obj: &mut serde_json::Map<String, Value>,
    msg_type: &str,
    flows: Option<&Vec<GraphMessageFlow>>,
) {
    // Return early if there are no flows to process.
    let flows = match flows {
        Some(flows) => flows,
        None => return,
    };

    // Get the message type array if it exists.
    let conn_flows = match conn_obj.get_mut(msg_type) {
        Some(Value::Array(flows)) => flows,
        _ => return,
    };

    // Process each flow in the connection.
    for conn_flow in conn_flows.iter_mut() {
        let conn_flow_obj = match conn_flow {
            Value::Object(obj) => obj,
            _ => continue,
        };

        let flow_name = match conn_flow_obj.get("name") {
            Some(Value::String(name)) => name,
            _ => continue,
        };

        // Find the matching flow in the modify connection.
        let modify_flow = match flows.iter().find(|f| &f.name == flow_name) {
            Some(flow) => flow,
            None => continue,
        };

        // Get destinations array if it exists.
        let conn_dests = match conn_flow_obj.get_mut("dest") {
            Some(Value::Array(dests)) => dests,
            _ => continue,
        };

        update_destinations(conn_dests, modify_flow);
    }
}

/// Update message conversion in destinations.
fn update_destinations(
    conn_dests: &mut [Value],
    modify_flow: &GraphMessageFlow,
) {
    for conn_dest in conn_dests.iter_mut() {
        let conn_dest_obj = match conn_dest {
            Value::Object(obj) => obj,
            _ => continue,
        };

        // Get destination extension and app.
        let dest_ext =
            match conn_dest_obj.get("extension").and_then(|v| v.as_str()) {
                Some(ext) => ext,
                None => continue,
            };
        let dest_app = conn_dest_obj.get("app").and_then(|v| v.as_str());

        // Find matching destination in the modify flow.
        for modify_dest in &modify_flow.dest {
            // Check if app and extension match.
            let app_match = match (dest_app, &modify_dest.app) {
                (None, None) => true,
                (Some(app1), Some(app2)) => app1 == app2,
                _ => false,
            };

            if !app_match || dest_ext != modify_dest.extension {
                continue;
            }

            // Update message conversion.
            match &modify_dest.msg_conversion {
                Some(msg_conversion) => {
                    // Convert msg_conversion to JSON value and insert.
                    if let Ok(conversion_value) =
                        serde_json::to_value(msg_conversion)
                    {
                        conn_dest_obj.insert(
                            "msg_conversion".to_string(),
                            conversion_value,
                        );
                    }
                }
                None => {
                    // Remove msg_conversion if it's None in the modify
                    // destination.
                    conn_dest_obj.remove("msg_conversion");
                }
            }

            // Found matching destination, no need to check more
            // modify_destinations.
            break;
        }
    }
}
