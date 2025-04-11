//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use uuid::Uuid;

use super::{
    constants::{PROPERTY_JSON_FILENAME, TEN_FIELD_IN_PROPERTY},
    pkg_type::PkgType,
    utils::read_file_to_string,
};
use crate::graph::{graph_info::GraphInfo, is_app_default_loc_or_none};
use crate::json_schema;

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
pub fn parse_property_from_str(
    s: &str,
    graphs_cache: &mut HashMap<Uuid, GraphInfo>,
    app_base_dir: Option<String>,
    belonging_pkg_type: Option<PkgType>,
    belonging_pkg_name: Option<String>,
) -> Result<Property> {
    let mut property: Property = serde_json::from_str(s)?;

    // Extract _ten field from all_fields if it exists.
    if let Some(ten_value) = property.all_fields.get(TEN_FIELD_IN_PROPERTY) {
        // Process the _ten field manually instead of using
        // serde_json::from_value directly. Create a TenInProperty with empty
        // predefined_graphs.
        let mut ten_in_property = TenInProperty {
            predefined_graphs: None,
            uri: None,
            additional_fields: HashMap::new(),
        };

        // Get other fields from ten_value using serde.
        if let Value::Object(map) = ten_value {
            // Extract and process predefined_graphs specially.
            if let Some(Value::Array(graphs_array)) =
                map.get("predefined_graphs")
            {
                let mut graph_infos = Vec::new();

                for graph_value in graphs_array {
                    let graph: GraphInfo =
                        serde_json::from_value(graph_value.clone())?;
                    graph_infos.push(graph);
                }

                validate_predefined_graphs(&graph_infos)?;

                let mut temp_graphs_cache = HashMap::new();
                let mut graph_uuids = Vec::new();

                for mut graph in graph_infos {
                    graph.validate_and_complete()?;

                    graph.belonging_pkg_type = belonging_pkg_type;
                    graph.belonging_pkg_name = belonging_pkg_name.clone();
                    graph.app_base_dir = app_base_dir.clone();

                    let uuid = Uuid::new_v4();
                    temp_graphs_cache.insert(uuid, graph);
                    graph_uuids.push(uuid);
                }

                graphs_cache.extend(temp_graphs_cache);
                ten_in_property.predefined_graphs = Some(graph_uuids);
            }

            // Handle uri if present
            if let Some(uri_value) = map.get("uri") {
                if let Some(uri_str) = uri_value.as_str() {
                    ten_in_property.uri = Some(uri_str.to_string());
                }
            }

            // Handle all other fields as additional_fields
            for (key, value) in map {
                if key != "predefined_graphs" && key != "uri" {
                    ten_in_property
                        .additional_fields
                        .insert(key.to_string(), value.clone());
                }
            }
        }

        property._ten = Some(ten_in_property);
    }

    property.validate_and_complete()?;

    Ok(property)
}

/// Validates that all predefined graphs have unique names.
fn validate_predefined_graphs(graphs: &[GraphInfo]) -> Result<()> {
    // Check for duplicate graph names in a separate scope to limit the
    // lifetime of the HashSet to just this validation step
    let mut seen_graph_names = std::collections::HashSet::new();

    for graph in graphs.iter() {
        if let Some(name) = &graph.name {
            // Note: We're storing references to graph names, which is correct.
            if !seen_graph_names.insert(name) {
                return Err(anyhow::anyhow!(
                    "Duplicate predefined graph name detected: '{}'. \
                    Each predefined_graph must have a unique 'name'.",
                    name
                ));
            }
        }
    }

    Ok(())
}

impl Property {
    /// Validates and completes the property configuration.
    ///
    /// This method ensures that the property configuration is valid and
    /// complete.
    pub fn validate_and_complete(&mut self) -> Result<()> {
        Ok(())
    }

    /// Serializes the property configuration to a JSON file.
    ///
    /// This method writes the `all_fields` map directly to the specified file
    /// path. This preserves the original order of fields in the
    /// `property.json` file.
    pub fn dump_property_to_file(
        &self,
        property_file_path: &PathBuf,
    ) -> Result<()> {
        let new_property_str = serde_json::to_string_pretty(&self.all_fields)?;
        fs::write(property_file_path, new_property_str)?;

        Ok(())
    }

    /// Returns the application URI from the property configuration.
    pub fn get_app_uri(&self) -> Option<String> {
        if let Some(_ten) = &self._ten {
            if let Some(uri) = &_ten.uri {
                return Some(uri.clone());
            }
        }

        None
    }
}

/// Represents the TEN-specific configuration within a property.json file.
///
/// This structure holds TEN Framework specific settings that are stored in the
/// property.json file of a package.
///
/// # Fields
/// * `predefined_graphs` - Optional list of UUIDs that reference graphs in the
///   graphs_cache. These graphs are predefined by the package.
/// * `uri` - Optional URI for the application. If not specified, defaults to
///   localhost.
/// * `additional_fields` - Captures any additional fields in the TEN
///   configuration that aren't explicitly defined in this struct.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TenInProperty {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub predefined_graphs: Option<Vec<Uuid>>,

    #[serde(skip_serializing_if = "is_app_default_loc_or_none")]
    pub uri: Option<String>,

    #[serde(flatten)]
    pub additional_fields: HashMap<String, Value>,
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
fn parse_property_from_file<P: AsRef<Path>>(
    property_file_path: P,
    graphs_cache: &mut HashMap<Uuid, GraphInfo>,
    app_base_dir: Option<String>,
    belonging_pkg_type: Option<PkgType>,
    belonging_pkg_name: Option<String>,
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
    parse_property_from_str(
        &content,
        graphs_cache,
        app_base_dir,
        belonging_pkg_type,
        belonging_pkg_name,
    )
    .map(Some)
}

pub fn parse_property_in_folder(
    folder_path: &Path,
    graphs_cache: &mut HashMap<Uuid, GraphInfo>,
    app_base_dir: Option<String>,
    belonging_pkg_type: Option<PkgType>,
    belonging_pkg_name: Option<String>,
) -> Result<Option<Property>> {
    // Path to the property.json file.
    let property_path = folder_path.join(PROPERTY_JSON_FILENAME);

    // Read and parse the property.json file.
    let property = parse_property_from_file(
        &property_path,
        graphs_cache,
        app_base_dir,
        belonging_pkg_type,
        belonging_pkg_name,
    )
    .with_context(|| format!("Failed to load {}.", property_path.display()))?;

    Ok(property)
}
