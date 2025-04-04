//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#[cfg(test)]
mod tests {
    use anyhow::Result;
    use serde_json::Value;
    use tempfile::TempDir;

    use ten_manager::graph::update_graph_connections_all_fields;
    use ten_rust::graph::connection::{
        GraphConnection, GraphDestination, GraphMessageFlow,
    };
    use ten_rust::pkg_info::constants::PROPERTY_JSON_FILENAME;

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
