//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::{collections::HashMap, fmt};

use linked_hash_map::LinkedHashMap;
use regex::Regex;
use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer, Serialize,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ManifestApi {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub property: Option<HashMap<String, ManifestPropertyAttributes>>,

    // Only manifest.json of extension has this field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cmd_in: Option<Vec<ManifestApiCmdLike>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cmd_out: Option<Vec<ManifestApiCmdLike>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_in: Option<Vec<ManifestApiDataLike>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_out: Option<Vec<ManifestApiDataLike>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_frame_in: Option<Vec<ManifestApiDataLike>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_frame_out: Option<Vec<ManifestApiDataLike>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub video_frame_in: Option<Vec<ManifestApiDataLike>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video_frame_out: Option<Vec<ManifestApiDataLike>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ManifestPropertyItem {
    pub attributes: ManifestPropertyAttributes,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ManifestPropertyAttributes {
    #[serde(rename = "type")]
    pub prop_type: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ManifestCmdResult {
    #[serde(deserialize_with = "deserialize_property_items", default)]
    pub property: HashMap<String, ManifestPropertyItem>,

    #[serde(deserialize_with = "deserialize_optional_required", default)]
    pub required: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ManifestApiCmdLike {
    #[serde(deserialize_with = "validate_msg_name")]
    pub name: String,

    #[serde(deserialize_with = "deserialize_optional_property_items", default)]
    pub property: Option<HashMap<String, ManifestPropertyItem>>,

    #[serde(deserialize_with = "deserialize_optional_required", default)]
    pub required: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<ManifestCmdResult>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ManifestApiDataLike {
    #[serde(deserialize_with = "validate_msg_name")]
    pub name: String,

    #[serde(deserialize_with = "deserialize_optional_property_items", default)]
    pub property: Option<HashMap<String, ManifestPropertyItem>>,

    #[serde(deserialize_with = "deserialize_optional_required", default)]
    pub required: Option<Vec<String>>,
}

fn validate_msg_name<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let msg_name: String = String::deserialize(deserializer)?;
    let re = Regex::new(r"^[A-Za-z_][A-Za-z0-9_]*$").unwrap();
    if re.is_match(&msg_name) {
        Ok(msg_name)
    } else {
        Err(serde::de::Error::custom("Invalid message name format, it needs to conform to the pattern ^[A-Za-z_][A-Za-z0-9_]*$"))
    }
}

fn deserialize_property_items<'de, D>(
    deserializer: D,
) -> Result<HashMap<String, ManifestPropertyItem>, D::Error>
where
    D: Deserializer<'de>,
{
    let map: LinkedHashMap<String, ManifestPropertyAttributes> =
        Deserialize::deserialize(deserializer)?;

    let items: HashMap<String, ManifestPropertyItem> = map
        .into_iter()
        .map(|(key, value)| (key, ManifestPropertyItem { attributes: value }))
        .collect();

    Ok(items)
}

fn deserialize_optional_property_items<'de, D>(
    deserializer: D,
) -> Result<Option<HashMap<String, ManifestPropertyItem>>, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_option(PropertyVisitor)
}

struct PropertyVisitor;

impl<'de> Visitor<'de> for PropertyVisitor {
    type Value = Option<HashMap<String, ManifestPropertyItem>>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an optional map of property items")
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(None)
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Preserve the insertion order. Because automatic code generation might
        // be needed in the future, the order defined within the API is very
        // important. Use deserialization methods that can preserve this order.
        let map: LinkedHashMap<String, ManifestPropertyAttributes> =
            Deserialize::deserialize(deserializer)?;

        let items: HashMap<String, ManifestPropertyItem> = map
            .into_iter()
            .map(|(key, value)| {
                (key, ManifestPropertyItem { attributes: value })
            })
            .collect();

        Ok(Some(items))
    }
}

fn deserialize_optional_required<'de, D>(
    deserializer: D,
) -> Result<Option<Vec<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_option(PropertyRequiredVisitor)
}

struct PropertyRequiredVisitor;

impl<'de> Visitor<'de> for PropertyRequiredVisitor {
    type Value = Option<Vec<String>>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an optional list of acquired properties.")
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(None)
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        let items: Vec<String> = Deserialize::deserialize(deserializer)?;

        if items.is_empty() {
            Err(serde::de::Error::custom(
                "the 'required' field should not be an empty list",
            ))
        } else {
            Ok(Some(items))
        }
    }
}
