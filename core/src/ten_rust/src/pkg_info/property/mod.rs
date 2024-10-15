//
// Copyright © 2024 Agora
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

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{
    constants::{PROPERTY_JSON_FILENAME, TEN_FIELD_IN_PROPERTY},
    utils::read_file_to_string,
};
use crate::pkg_info::graph::is_app_default_loc_or_none;
use crate::{json_schema, pkg_info::default_app_loc};
use predefined_graph::PropertyPredefinedGraph;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Property {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _ten: Option<TenInProperty>,

    #[serde(flatten)]
    pub additional_fields: HashMap<String, Value>,
}

impl FromStr for Property {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut property: Property = serde_json::from_str(s)?;
        property.validate_and_complete()?;
        Ok(property)
    }
}

impl Property {
    pub fn validate_and_complete(&mut self) -> Result<()> {
        if let Some(_ten) = &mut self._ten {
            _ten.validate_and_complete()?;
        }
        Ok(())
    }

    pub fn dump_property_to_file(
        &self,
        property_file_path: &PathBuf,
    ) -> Result<()> {
        let mut property_json = serde_json::Map::new();

        if let Some(_ten) = &self._ten {
            property_json.insert(
                TEN_FIELD_IN_PROPERTY.to_string(),
                serde_json::to_value(_ten)?,
            );
        }

        // Merge additional fields back into the property JSON
        for (key, value) in &self.additional_fields {
            property_json.insert(key.clone(), value.clone());
        }

        let new_property_str = serde_json::to_string_pretty(&property_json)?;
        fs::write(property_file_path, new_property_str)?;

        Ok(())
    }

    pub fn get_app_uri(&self) -> String {
        if let Some(_ten) = &self._ten {
            if let Some(uri) = &_ten.uri {
                return uri.clone();
            }
        }

        default_app_loc()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TenInProperty {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub predefined_graphs: Option<Vec<PropertyPredefinedGraph>>,

    #[serde(skip_serializing_if = "is_app_default_loc_or_none")]
    pub uri: Option<String>,

    #[serde(flatten)]
    pub additional_fields: HashMap<String, Value>,
}

impl TenInProperty {
    pub fn validate_and_complete(&mut self) -> Result<()> {
        if let Some(graphs) = &mut self.predefined_graphs {
            for graph in graphs {
                graph.validate_and_complete()?;
            }
        }

        if self.uri.is_none() {
            self.uri = Some(default_app_loc());
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

pub fn parse_property_from_file<P: AsRef<Path>>(
    property_file_path: P,
) -> Result<Property> {
    // Read the contents of the property.json file.
    let content = read_file_to_string(property_file_path)?;

    Property::from_str(&content)
}

pub fn parse_property_in_folder(folder_path: &Path) -> Result<Property> {
    // Path to the property.json file.
    let property_path = folder_path.join(PROPERTY_JSON_FILENAME);

    // Read and parse the property.json file.
    let property = parse_property_from_file(&property_path)?;

    // Validate the property schema only if it is present.
    let validation =
        json_schema::ten_validate_property_json_file(&property_path);

    if let Err(validation_err) = validation {
        return Err(anyhow::anyhow!(
            "Failed to validate {}, caused by {}",
            property_path.display(),
            validation_err
        ));
    }

    Ok(property)
}

#[cfg(test)]
mod tests {
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
        assert!(property.additional_fields.is_empty());
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

        assert_eq!(
            property.additional_fields.get("global_field_1").unwrap(),
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

        let property = Property {
            _ten: Some(ten_in_property),
            additional_fields: HashMap::new(),
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
        assert_eq!(node.get_app_uri(), default_app_loc());

        let dir = tempdir().unwrap();
        let file_path = dir.path().join("property.json");
        property.dump_property_to_file(&file_path).unwrap();

        let saved_content = fs::read_to_string(file_path).unwrap();
        eprintln!("{}", saved_content);
        assert_eq!(saved_content.find(default_app_loc().as_str()), None);
    }
}
