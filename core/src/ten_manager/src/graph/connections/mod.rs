//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::{fs::OpenOptions, path::Path};

use anyhow::{Context, Result};
use serde_json::Value;

use ten_rust::{
    graph::connection::GraphConnection,
    pkg_info::constants::PROPERTY_JSON_FILENAME,
};

/// Update the connections of a graph in the property.json file.
///
/// This function updates the connections in the property.json file for a
/// specified graph. It adds or removes connections from the existing list,
/// preserving the original order.
///
/// Both `connections_to_add` and `connections_to_remove` are optional, allowing
/// for:
///   - Adding new connections without removing any.
///   - Removing connections without adding any new ones.
///   - Both adding and removing connections in a single call.
///
/// In all cases, the original order of connections in the property.json file is
/// preserved.
pub fn update_graph_connections_all_fields(
    pkg_url: &str,
    property_all_fields: &mut serde_json::Map<String, Value>,
    graph_name: &str,
    connections_to_add: Option<&[GraphConnection]>,
    connections_to_remove: Option<&[GraphConnection]>,
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
                            if let Some(Value::Array(connections_array)) =
                                graph_obj.get_mut("connections")
                            {
                                // Remove connections if requested.
                                if let Some(remove_connections) =
                                    connections_to_remove
                                {
                                    if !remove_connections.is_empty() {
                                        // Convert connections to remove into
                                        // comparable form.
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
                                }

                                // Add new connections if provided.
                                if let Some(add_connections) =
                                    connections_to_add
                                {
                                    if !add_connections.is_empty() {
                                        // First find existing connections by
                                        // app and extension to update them
                                        // instead of adding new ones.
                                        for new_connection in add_connections {
                                            if let Ok(new_connection_value) =
                                                serde_json::to_value(
                                                    new_connection,
                                                )
                                            {
                                                // Find existing connection with
                                                // same app and extension.
                                                let app_value =
                                                    new_connection_value
                                                        .get("app")
                                                        .cloned();
                                                let extension_value =
                                                    new_connection_value
                                                        .get("extension")
                                                        .cloned();

                                                let existing_idx = connections_array
                                                    .iter()
                                                    .position(|conn| {
                                                        let conn_app = conn.get("app");
                                                        let conn_ext = conn.get("extension");

                                                        // Match app and extension.
                                                        (app_value.is_none() && conn_app.is_none() ||
                                                         app_value.as_ref() == conn_app) &&
                                                        (extension_value.as_ref() == conn_ext)
                                                    });

                                                if let Some(idx) = existing_idx
                                                {
                                                    // Update existing
                                                    // connection by merging
                                                    // message fields.
                                                    if let Some(
                                                        Value::Object(conn_obj),
                                                    ) = connections_array
                                                        .get_mut(idx)
                                                    {
                                                        // Update cmd field if
                                                        // present in new
                                                        // connection.
                                                        if let Some(cmd_value) =
                                                            new_connection_value
                                                                .get("cmd")
                                                        {
                                                            if cmd_value
                                                                .is_array()
                                                            {
                                                                // If existing
                                                                // connection
                                                                // has cmd field,
                                                                // append to it.
                                                                if let Some(Value::Array(existing_cmd)) = conn_obj.get_mut("cmd") {
                                                                    // Get new cmd messages to add.
                                                                    if let Some(new_cmds) = cmd_value.as_array() {
                                                                        // Append each new cmd.
                                                                        for cmd in new_cmds {
                                                                            existing_cmd.push(cmd.clone());
                                                                        }
                                                                    }
                                                                } else {
                                                                    // No existing cmd field, add it.
                                                                    conn_obj.insert("cmd".to_string(), cmd_value.clone());
                                                                }
                                                            }
                                                        }

                                                        // Update data field
                                                        // if present in new
                                                        // connection.
                                                        if let Some(
                                                            data_value,
                                                        ) =
                                                            new_connection_value
                                                                .get("data")
                                                        {
                                                            if data_value
                                                                .is_array()
                                                            {
                                                                // If existing
                                                                // connection
                                                                // has data field,
                                                                // append to it.
                                                                if let Some(Value::Array(existing_data)) = conn_obj.get_mut("data") {
                                                                    // Get new data messages to add.
                                                                    if let Some(new_data) = data_value.as_array() {
                                                                        // Append each new data.
                                                                        for data in new_data {
                                                                            existing_data.push(data.clone());
                                                                        }
                                                                    }
                                                                } else {
                                                                    // No existing data field, add it.
                                                                    conn_obj.insert("data".to_string(), data_value.clone());
                                                                }
                                                            }
                                                        }

                                                        // Update audio_frame
                                                        // field if present in
                                                        // new connection.
                                                        if let Some(audio_value) = new_connection_value.get("audio_frame") {
                                                            if audio_value.is_array() {
                                                                // If existing connection has audio_frame field, append to it.
                                                                if let Some(Value::Array(existing_audio)) = conn_obj.get_mut("audio_frame") {
                                                                    // Get new audio messages to add.
                                                                    if let Some(new_audio) = audio_value.as_array() {
                                                                        // Append each new audio.
                                                                        for audio in new_audio {
                                                                            existing_audio.push(audio.clone());
                                                                        }
                                                                    }
                                                                } else {
                                                                    // No existing audio_frame field, add it.
                                                                    conn_obj.insert("audio_frame".to_string(), audio_value.clone());
                                                                }
                                                            }
                                                        }

                                                        // Update video_frame
                                                        // field if present in
                                                        // new connection.
                                                        if let Some(video_value) = new_connection_value.get("video_frame") {
                                                            if video_value.is_array() {
                                                                // If existing connection has video_frame field, append to it.
                                                                if let Some(Value::Array(existing_video)) = conn_obj.get_mut("video_frame") {
                                                                    // Get new video messages to add.
                                                                    if let Some(new_video) = video_value.as_array() {
                                                                        // Append each new video.
                                                                        for video in new_video {
                                                                            existing_video.push(video.clone());
                                                                        }
                                                                    }
                                                                } else {
                                                                    // No existing video_frame field, add it.
                                                                    conn_obj.insert("video_frame".to_string(), video_value.clone());
                                                                }
                                                            }
                                                        }
                                                    }
                                                } else {
                                                    // No existing connection,
                                                    // add a new one.
                                                    connections_array.push(
                                                        new_connection_value,
                                                    );
                                                }
                                            }
                                        }
                                    }
                                }

                                // If all connections are removed, remove the
                                // connections array.
                                if connections_array.is_empty() {
                                    graph_obj.remove("connections");
                                }
                            } else {
                                // No connections array in the graph yet, create
                                // one if we have connections to add.
                                if let Some(add_connections) =
                                    connections_to_add
                                {
                                    if !add_connections.is_empty() {
                                        let mut connections_array = Vec::new();

                                        // Add all new connections to the array.
                                        for new_connection in add_connections {
                                            if let Ok(new_connection_value) =
                                                serde_json::to_value(
                                                    new_connection,
                                                )
                                            {
                                                connections_array
                                                    .push(new_connection_value);
                                            }
                                        }

                                        graph_obj.insert(
                                            "connections".to_string(),
                                            Value::Array(connections_array),
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
    use serde_json::Value;
    use tempfile::TempDir;
    use ten_rust::graph::connection::{GraphDestination, GraphMessageFlow};

    #[test]
    fn test_update_connections_preserves_order() -> Result<()> {
        // Create a temporary directory for our test.
        let temp_dir = TempDir::new()?;
        let test_dir = temp_dir.path().to_str().unwrap().to_string();

        // First, create the initial property.json with a connection.
        let initial_json = r#"{
  "_ten": {
    "predefined_graphs": [
      {
        "name": "test_graph",
        "connections": [
          {
            "app": "http://example.com:8000",
            "extension": "extension_1",
            "cmd": [
              {
                "dest": [
                  {
                    "app": "http://example.com:8000",
                    "extension": "extension_2"
                  }
                ],
                "name": "existing_cmd"
              }
            ]
          }
        ]
      }
    ]
  }
}"#;

        // Expected JSON after adding the connections.
        let expected_json = r#"{
  "_ten": {
    "predefined_graphs": [
      {
        "name": "test_graph",
        "connections": [
          {
            "app": "http://example.com:8000",
            "extension": "extension_1",
            "cmd": [
              {
                "dest": [
                  {
                    "app": "http://example.com:8000",
                    "extension": "extension_2"
                  }
                ],
                "name": "existing_cmd"
              },
              {
                "name": "new_cmd1",
                "dest": [
                  {
                    "app": "http://example.com:8000",
                    "extension": "extension_2"
                  }
                ]
              },
              {
                "name": "new_cmd2",
                "dest": [
                  {
                    "app": "http://example.com:8000",
                    "extension": "extension_3"
                  }
                ]
              }
            ]
          }
        ]
      }
    ]
  }
}"#;

        // Write the initial JSON to property.json.
        let property_path =
            std::path::Path::new(&test_dir).join(PROPERTY_JSON_FILENAME);
        std::fs::write(&property_path, initial_json)?;

        // Parse the initial JSON to all_fields.
        let mut all_fields: serde_json::Map<String, Value> =
            serde_json::from_str(initial_json)?;

        // Create connections to add.
        let connection1 = GraphConnection {
            app: Some("http://example.com:8000".to_string()),
            extension: "extension_1".to_string(),
            cmd: Some(vec![GraphMessageFlow {
                name: "new_cmd1".to_string(),
                dest: vec![GraphDestination {
                    app: Some("http://example.com:8000".to_string()),
                    extension: "extension_2".to_string(),
                    msg_conversion: None,
                }],
            }]),
            data: None,
            audio_frame: None,
            video_frame: None,
        };

        let connection2 = GraphConnection {
            app: Some("http://example.com:8000".to_string()),
            extension: "extension_1".to_string(),
            cmd: Some(vec![GraphMessageFlow {
                name: "new_cmd2".to_string(),
                dest: vec![GraphDestination {
                    app: Some("http://example.com:8000".to_string()),
                    extension: "extension_3".to_string(),
                    msg_conversion: None,
                }],
            }]),
            data: None,
            audio_frame: None,
            video_frame: None,
        };

        let connections_to_add = vec![connection1, connection2];

        // Update the connections in memory and in the file.
        update_graph_connections_all_fields(
            &test_dir,
            &mut all_fields,
            "test_graph",
            Some(&connections_to_add),
            None,
        )?;

        // Read the updated property.json.
        let actual_json = std::fs::read_to_string(&property_path)?;

        // Normalize both JSON strings (parse and reformat to remove whitespace
        // differences).
        let expected_value: serde_json::Value =
            serde_json::from_str(expected_json)?;
        let actual_value: serde_json::Value =
            serde_json::from_str(&actual_json)?;

        // Convert back to normalized strings for comparison
        let normalized_expected =
            serde_json::to_string_pretty(&expected_value)?;
        let normalized_actual = serde_json::to_string_pretty(&actual_value)?;

        if normalized_expected != normalized_actual {
            eprintln!("Expected JSON:\n{}", normalized_expected);
            eprintln!("Actual JSON:\n{}", normalized_actual);

            // Find where they differ.
            let expected_lines: Vec<&str> =
                normalized_expected.lines().collect();
            let actual_lines: Vec<&str> = normalized_actual.lines().collect();

            for (i, (expected_line, actual_line)) in
                expected_lines.iter().zip(actual_lines.iter()).enumerate()
            {
                if expected_line != actual_line {
                    eprintln!("First difference at line {}:", i + 1);
                    eprintln!("Expected: {}", expected_line);
                    eprintln!("Actual:   {}", actual_line);
                    break;
                }
            }

            panic!("JSON content doesn't match");
        }

        Ok(())
    }
}
