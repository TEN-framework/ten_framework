//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
pub mod predefined_graph;

use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use super::{
    constants::{PROPERTY_JSON_FILENAME, TEN_FIELD_IN_PROPERTY},
    utils::read_file_to_string,
};
use crate::graph::is_app_default_loc_or_none;
use crate::{json_schema, pkg_info::localhost};
use predefined_graph::PredefinedGraph;

/// Represents the property configuration of a TEN package.
///
/// The property configuration consists of two parts:
/// 1. A special `_ten` field that contains TEN-specific configuration.
/// 2. All fields from property.json, stored with preserved order.
///
/// This structure is typically serialized to and deserialized from a JSON file
/// named `property.json` in the package directory. The `_ten` field serves as a
/// cache for the "_ten" field in property.json for faster access, while
/// `all_fields` stores all fields from property.json with preserved order.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Property {
    /// TEN-specific configuration properties as a cache for faster access.
    /// This field is not serialized and is only used for internal processing.
    #[serde(skip)]
    pub _ten: Option<TenInProperty>,

    /// All fields from property.json, stored with order preserved.
    /// This includes the "_ten" field if it exists in property.json.
    #[serde(flatten)]
    pub all_fields: Map<String, Value>,
}

/// Implements the `FromStr` trait for `Property` to enable parsing from a
/// string.
///
/// This implementation allows creating a `Property` instance from a JSON
/// string, which is useful for loading property configurations from files or
/// string literals. After parsing the JSON, it automatically validates and
/// completes the property configuration to ensure it meets all requirements.
///
/// # Arguments
/// * `s` - A string slice containing a valid JSON representation of a property
///   configuration.
///
/// # Returns
/// * `Ok(Property)` - A validated and completed property configuration if
///   parsing succeeds.
/// * `Err` - An error if JSON parsing fails or if validation fails.
impl FromStr for Property {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut property: Property = serde_json::from_str(s)?;

        // Extract _ten field from all_fields if it exists.
        if let Some(ten_value) = property.all_fields.get(TEN_FIELD_IN_PROPERTY)
        {
            let ten_in_property: TenInProperty =
                serde_json::from_value(ten_value.clone())?;
            property._ten = Some(ten_in_property);
        }

        property.validate_and_complete()?;
        Ok(property)
    }
}

impl Property {
    /// Validates and completes the property configuration.
    ///
    /// This method ensures that the property configuration is valid and
    /// complete. If the `_ten` field is present, it delegates validation to
    /// the `TenInProperty` struct's own validation method.
    ///
    /// # Returns
    /// - `Ok(())` if validation succeeds.
    /// - `Err` containing the validation error if validation fails.
    pub fn validate_and_complete(&mut self) -> Result<()> {
        if let Some(_ten) = &mut self._ten {
            _ten.validate_and_complete()?;
        }
        Ok(())
    }

    /// Serializes the property configuration to a JSON file.
    ///
    /// This method writes the `all_fields` map directly to the specified file
    /// path. This preserves the original order of fields in the
    /// `property.json` file.
    ///
    /// # Arguments
    /// * `property_file_path` - The path where the property JSON file should be
    ///   written.
    ///
    /// # Returns
    /// * `Ok(())` if the file was successfully written.
    /// * `Err` containing the error if serialization or file writing fails.
    pub fn dump_property_to_file(
        &self,
        property_file_path: &PathBuf,
    ) -> Result<()> {
        let new_property_str = serde_json::to_string_pretty(&self.all_fields)?;
        fs::write(property_file_path, new_property_str)?;

        Ok(())
    }

    /// Returns the application URI from the property configuration.
    ///
    /// # Returns
    /// * The URI specified in the TEN configuration if it exists.
    /// * A default localhost URI if no URI is specified in the configuration.
    ///
    /// This method retrieves the URI from the TEN-specific part of the property
    /// configuration. If the TEN configuration is missing or doesn't contain a
    /// URI, it falls back to a default localhost address.
    pub fn get_app_uri(&self) -> String {
        if let Some(_ten) = &self._ten {
            if let Some(uri) = &_ten.uri {
                return uri.clone();
            }
        }

        localhost()
    }
}

/// Represents the TEN-specific configuration within a property.json file.
///
/// This structure holds TEN Framework specific settings that are stored in the
/// property.json file of a package.
///
/// # Fields
/// * `predefined_graphs` - Optional list of predefined graphs that the package
///   provides.
/// * `uri` - Optional URI for the application. If not specified, defaults to
///   localhost.
/// * `additional_fields` - Captures any additional fields in the TEN
///   configuration that aren't explicitly defined in this struct.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TenInProperty {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub predefined_graphs: Option<Vec<PredefinedGraph>>,

    #[serde(skip_serializing_if = "is_app_default_loc_or_none")]
    pub uri: Option<String>,

    #[serde(flatten)]
    pub additional_fields: HashMap<String, Value>,
}

impl TenInProperty {
    /// Validates and completes the TEN property configuration.
    ///
    /// This method performs two main tasks:
    /// 1. Validates that all predefined graphs have unique names.
    /// 2. Ensures each predefined graph is valid by calling its own validation
    ///    method.
    /// 3. Sets a default URI (localhost) if none is specified.
    ///
    /// # Returns
    /// * `Ok(())` if validation passes and completion is successful.
    /// * `Err` with a descriptive message if validation fails
    pub fn validate_and_complete(&mut self) -> Result<()> {
        if let Some(graphs) = &mut self.predefined_graphs {
            // Check for duplicate graph names in a separate scope to limit the
            // lifetime of the HashSet to just this validation step
            {
                let mut seen_graph_names = std::collections::HashSet::new();
                for graph in graphs.iter() {
                    // Note: We're storing references to graph names, which is
                    // correct.
                    if !seen_graph_names.insert(&graph.name) {
                        return Err(anyhow::anyhow!(
                            "Duplicate predefined graph name detected: '{}'. \
                            Each predefined_graph must have a unique 'name'.",
                            graph.name
                        ));
                    }
                }
            }

            // Validate each individual graph.
            for graph in graphs {
                graph.validate_and_complete()?;
            }
        }

        // Set default URI if none is provided.
        if self.uri.is_none() {
            self.uri = Some(localhost());
        }

        Ok(())
    }
}

pub fn check_property_json_of_pkg(pkg_dir: &str) -> Result<()> {
    let property_json_path = Path::new(pkg_dir).join(PROPERTY_JSON_FILENAME);
    if !property_json_path.exists() {
        return Ok(());
    }

    json_schema::ten_validate_property_json_file(
        property_json_path.to_str().unwrap(),
    )
}

/// Parses a property.json file into a Property struct.
///
/// This function reads the contents of the specified property file,
/// deserializes it into a Property struct, and validates the structure.
///
/// # Arguments
/// * `property_file_path` - Path to the property.json file.
///
/// # Returns
/// * `Result<Property>` - The parsed and validated Property struct on success,
///   or an error if the file cannot be read or the content is invalid.
pub fn parse_property_from_file<P: AsRef<Path>>(
    property_file_path: P,
) -> Result<Option<Property>> {
    if !property_file_path.as_ref().exists() {
        return Ok(None);
    }

    // Validate the property schema only if it is present.
    json_schema::ten_validate_property_json_file(&property_file_path)
        .with_context(|| {
            format!(
                "Failed to validate {}.",
                property_file_path.as_ref().display()
            )
        })?;

    // Read the contents of the property.json file.
    let content = read_file_to_string(property_file_path)?;

    // Parse the content and validate the property structure.
    Property::from_str(&content).map(Some)
}

pub fn parse_property_in_folder(
    folder_path: &Path,
) -> Result<Option<Property>> {
    // Path to the property.json file.
    let property_path = folder_path.join(PROPERTY_JSON_FILENAME);

    // Read and parse the property.json file.
    let property =
        parse_property_from_file(&property_path).with_context(|| {
            format!("Failed to load {}.", property_path.display())
        })?;

    Ok(property)
}

#[cfg(test)]
mod tests {
    use crate::graph::msg_conversion::{MsgConversionMode, MsgConversionType};

    use super::*;
    use std::fs::{self};
    use tempfile::tempdir;

    #[test]
    fn test_parse_property_with_known_fields() {
        let json_data = r#"
        {
            "_ten": {
                "predefined_graphs": [],
                "uri": "http://example.com"
            }
        }
        "#;

        let property: Property = json_data.parse().unwrap();

        assert!(property._ten.is_some());
        let ten_in_property = property._ten.unwrap();

        assert_eq!(ten_in_property.uri.unwrap(), "http://example.com");
        assert!(ten_in_property.predefined_graphs.is_some());
        assert!(ten_in_property.additional_fields.is_empty());
        assert_eq!(property.all_fields.len(), 1); // Should contain _ten field.
        assert!(property.all_fields.contains_key("_ten"));
    }

    #[test]
    fn test_parse_property_with_additional_fields() {
        let json_data = r#"
        {
            "_ten": {
                "predefined_graphs": [],
                "uri": "http://example.com",
                "extra_field_1": "value1"
            },
            "global_field_1": "global_value1"
        }
        "#;

        let property: Property = json_data.parse().unwrap();

        assert!(property._ten.is_some());
        let ten_in_property = property._ten.unwrap();
        assert_eq!(ten_in_property.uri.unwrap(), "http://example.com");
        assert!(ten_in_property.predefined_graphs.is_some());
        assert_eq!(
            ten_in_property
                .additional_fields
                .get("extra_field_1")
                .unwrap(),
            "value1"
        );

        // Should contain _ten and global_field_1.
        assert_eq!(property.all_fields.len(), 2);
        assert_eq!(
            property.all_fields.get("global_field_1").unwrap(),
            "global_value1"
        );
    }

    #[test]
    fn test_dump_property_to_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("property.json");

        let ten_in_property = TenInProperty {
            predefined_graphs: Some(vec![]),
            uri: Some(String::from("http://example.com")),
            additional_fields: HashMap::new(),
        };

        // Create a property with all_fields containing _ten data.
        let mut all_fields = Map::new();
        all_fields.insert(
            TEN_FIELD_IN_PROPERTY.to_string(),
            serde_json::to_value(&ten_in_property).unwrap(),
        );

        let property = Property {
            _ten: Some(ten_in_property),
            all_fields,
        };

        property.dump_property_to_file(&file_path).unwrap();

        let saved_content = fs::read_to_string(file_path).unwrap();
        let expected_content = r#"
        {
            "_ten": {
                "predefined_graphs": [],
                "uri": "http://example.com"
            }
        }
        "#
        .trim();

        let saved_json: serde_json::Value =
            serde_json::from_str(&saved_content).unwrap();
        let expected_json: serde_json::Value =
            serde_json::from_str(expected_content).unwrap();

        assert_eq!(saved_json, expected_json);
    }

    #[test]
    fn test_parse_and_dump_property_with_additional_fields() {
        let json_data = r#"
        {
            "_ten": {
                "predefined_graphs": [],
                "uri": "http://example.com",
                "extra_field_1": "value1"
            },
            "global_field_1": "global_value1"
        }
        "#;

        let property: Property = json_data.parse().unwrap();

        let dir = tempdir().unwrap();
        let file_path = dir.path().join("property.json");

        property.dump_property_to_file(&file_path).unwrap();

        let saved_content = fs::read_to_string(file_path).unwrap();
        let saved_json: serde_json::Value =
            serde_json::from_str(&saved_content).unwrap();
        let original_json: serde_json::Value =
            serde_json::from_str(json_data).unwrap();

        assert_eq!(saved_json, original_json);
    }

    #[test]
    fn test_dump_property_without_localhost_app_in_graph() {
        let json_data = include_str!("test_data_embed/property.json");
        let property: Property = json_data.parse().unwrap();
        assert!(property._ten.is_some());
        let ten = property._ten.as_ref().unwrap();
        let predefined_graphs = ten.predefined_graphs.as_ref().unwrap();
        let nodes = &predefined_graphs.first().as_ref().unwrap().graph.nodes;
        let node = nodes.first().unwrap();
        assert_eq!(node.get_app_uri(), localhost());

        let dir = tempdir().unwrap();
        let file_path = dir.path().join("property.json");
        property.dump_property_to_file(&file_path).unwrap();

        let saved_content = fs::read_to_string(file_path).unwrap();
        eprintln!("{}", saved_content);
        assert_eq!(saved_content.find(localhost().as_str()), None);
    }

    #[test]
    fn test_dump_property_with_msg_conversion() {
        let prop_str = include_str!(
            "test_data_embed/dump_property_with_msg_conversion.json"
        );
        let property: Property = prop_str.parse().unwrap();
        assert!(property._ten.is_some());

        let ten = property._ten.as_ref().unwrap();
        let predefined_graphs = ten.predefined_graphs.as_ref().unwrap();
        let connections = &predefined_graphs
            .first()
            .as_ref()
            .unwrap()
            .graph
            .connections
            .as_ref()
            .unwrap();
        let connection = connections.first().unwrap();
        let cmd = connection.cmd.as_ref().unwrap();
        assert_eq!(cmd.len(), 1);

        let cmd_dest = &cmd.first().unwrap().dest;
        assert_eq!(cmd_dest.len(), 1);

        let msg_conversion =
            cmd_dest.first().unwrap().msg_conversion.as_ref().unwrap();
        assert_eq!(
            msg_conversion.msg.conversion_type,
            MsgConversionType::PerProperty
        );

        let rules = &msg_conversion.msg.rules;
        assert!(rules.keep_original.is_none());
        assert_eq!(rules.rules.len(), 2);

        let rule = rules.rules.first().unwrap();
        assert_eq!(rule.conversion_mode, MsgConversionMode::FixedValue);
        assert!(rule.original_path.is_none());
        assert_eq!(rule.value.as_ref().unwrap(), "hello_mapping");

        let dir = tempdir().unwrap();
        let file_path = dir.path().join("property.json");

        property.dump_property_to_file(&file_path).unwrap();

        let saved_content = fs::read_to_string(file_path).unwrap();
        eprintln!("{}", saved_content);
        assert!(saved_content.contains("msg_conversion"));
    }
}
