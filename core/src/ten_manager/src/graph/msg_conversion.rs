//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::collections::HashMap;

use anyhow::Result;

use ten_rust::{
    base_dir_pkg_info::{PkgsInfoInApp, PkgsInfoInAppWithBaseDir},
    graph::msg_conversion::{
        MsgAndResultConversion, MsgConversionMode, MsgConversionRule,
    },
    pkg_info::{
        get_pkg_info_for_extension_addon,
        manifest::api::{
            ManifestApiCmdResult, ManifestApiMsg, ManifestApiPropertyAttributes,
        },
        message::{MsgDirection, MsgType},
        value_type::ValueType,
    },
};

use crate::constants::TEN_NAME_RULE_PATH;

// Helper function to parse a path string and navigate to the specified location
// in the property map.
fn navigate_property_path_mut<'a>(
    properties: &'a mut HashMap<String, ManifestApiPropertyAttributes>,
    path: &str,
) -> Result<&'a mut ManifestApiPropertyAttributes> {
    // Split the path by '.' for object properties and handle array indices.
    let path_parts: Vec<&str> = path.split('.').collect();

    let mut current_props = properties;

    for (i, part) in path_parts.iter().enumerate() {
        // Handle array index notation: part[index].
        let (name, index) = if let Some(bracket_pos) = part.find('[') {
            // Get the property name.
            let name = &part[0..bracket_pos];

            // Get the index string.
            let index_str = &part[bracket_pos + 1..part.len() - 1];

            // Parse the index.
            let index = index_str.parse::<usize>().map_err(|_| {
                anyhow::anyhow!("Invalid array index in path: {}", part)
            })?;

            (name, Some(index))
        } else {
            (*part, None)
        };

        // If this is the last part, return a reference to it.
        if i == path_parts.len() - 1 {
            // Create the property if it doesn't exist.
            if !current_props.contains_key(name) {
                current_props.insert(
                    name.to_string(),
                    ManifestApiPropertyAttributes {
                        prop_type: if index.is_some() {
                            ValueType::Array
                        } else {
                            ValueType::Object
                        },
                        items: None,
                        properties: None,
                        required: None,
                    },
                );
            }

            // Get the property.
            let prop = current_props.get_mut(name).unwrap();

            // Handle array index if present.
            if index.is_some() {
                // Make sure it's an array type.
                if prop.prop_type != ValueType::Array {
                    return Err(anyhow::anyhow!(
                        "Property {} is not an array",
                        name
                    ));
                }

                // Make sure items is defined.
                if prop.items.is_none() {
                    prop.items =
                        Some(Box::new(ManifestApiPropertyAttributes {
                            prop_type: ValueType::Object,
                            items: None,
                            properties: None,
                            required: None,
                        }));
                }

                // We can't actually index into the array here because arrays
                // don't store their items that way in the schema. We return the
                // items definition instead.
                return Ok(prop.items.as_mut().unwrap());
            }

            return Ok(prop);
        }

        // If not the last part, continue navigation.
        if !current_props.contains_key(name) {
            // Create intermediate object properties if they don't exist.
            current_props.insert(
                name.to_string(),
                ManifestApiPropertyAttributes {
                    prop_type: if index.is_some() {
                        ValueType::Array
                    } else {
                        ValueType::Object
                    },
                    items: None,
                    properties: Some(HashMap::new()),
                    required: None,
                },
            );
        }

        let prop = current_props.get_mut(name).unwrap();

        // Handle array index if present.
        if index.is_some() {
            // Make sure it's an array type.
            if prop.prop_type != ValueType::Array {
                return Err(anyhow::anyhow!(
                    "Property {} is not an array",
                    name
                ));
            }

            // Make sure items is defined.
            if prop.items.is_none() {
                prop.items = Some(Box::new(ManifestApiPropertyAttributes {
                    prop_type: ValueType::Object,
                    items: None,
                    properties: Some(HashMap::new()),
                    required: None,
                }));
            }

            // We need to ensure our items have properties to navigate into.
            let items = prop.items.as_mut().unwrap();
            if items.properties.is_none() {
                items.properties = Some(HashMap::new());
            }

            // Navigate into the array item's properties
            current_props = items.properties.as_mut().unwrap();
        } else {
            // Make sure it's an object type.
            if prop.prop_type != ValueType::Object {
                return Err(anyhow::anyhow!(
                    "Property {} is not an object",
                    name
                ));
            }

            // Make sure properties is defined.
            if prop.properties.is_none() {
                prop.properties = Some(HashMap::new());
            }

            // Navigate into the object's properties.
            current_props = prop.properties.as_mut().unwrap();
        }
    }

    Err(anyhow::anyhow!(
        "Failed to navigate property path: {}",
        path
    ))
}

// Helper function to find a property at the specified path in a read-only
// property map.
fn navigate_property_path<'a>(
    properties: Option<&'a HashMap<String, ManifestApiPropertyAttributes>>,
    path: &str,
) -> Option<&'a ManifestApiPropertyAttributes> {
    properties?;

    let properties = properties.unwrap();

    // Split the path by dots for object properties and handle array indices
    let path_parts: Vec<&str> = path.split('.').collect();

    let mut current_props = properties;

    for (i, part) in path_parts.iter().enumerate() {
        // Handle array index notation: part[index]
        let (name, index) = if let Some(bracket_pos) = part.find('[') {
            // Get the property name.
            let name = &part[0..bracket_pos];

            // Get the index string.
            let index_str = &part[bracket_pos + 1..part.len() - 1];

            // Parse the index.
            if let Ok(index) = index_str.parse::<usize>() {
                (name, Some(index))
            } else {
                return None;
            }
        } else {
            (*part, None)
        };

        // Check if the property exists.
        if !current_props.contains_key(name) {
            return None;
        }

        let prop = &current_props[name];

        // If this is the last part, return it.
        if i == path_parts.len() - 1 {
            // Handle array index if present.
            if index.is_some() {
                // For arrays, we return the items template.
                if prop.prop_type != ValueType::Array || prop.items.is_none() {
                    return None;
                }
                return Some(prop.items.as_ref().unwrap());
            }

            return Some(prop);
        }

        // Handle array index if present.
        if index.is_some() {
            // We need to navigate into array items.
            if prop.prop_type != ValueType::Array || prop.items.is_none() {
                return None;
            }

            let items = prop.items.as_ref().unwrap();
            let properties = items.properties.as_ref()?;

            current_props = properties;
        } else {
            // Navigate into object properties.
            if prop.prop_type != ValueType::Object || prop.properties.is_none()
            {
                return None;
            }

            current_props = prop.properties.as_ref().unwrap();
        }
    }

    None
}

fn convert_rules_to_schema_properties(
    conversion_rules: &[MsgConversionRule],
    ten_name_rule_index: Option<usize>,
    src_schema_properties: Option<
        &HashMap<String, ManifestApiPropertyAttributes>,
    >,
    dest_schema_properties: &mut HashMap<String, ManifestApiPropertyAttributes>,
) -> Result<()> {
    // Process each conversion rule.
    for (index, rule) in conversion_rules.iter().enumerate() {
        // Skip the _ten.name rule if we found it earlier.
        if Some(index) == ten_name_rule_index {
            continue;
        }

        // Process the rule based on conversion mode.
        match rule.conversion_mode {
            MsgConversionMode::FixedValue => {
                // Process fixed value rule.
                if let Some(value) = &rule.value {
                    // Parse the path and get a mutable reference to the
                    // property.
                    let path = &rule.path;

                    // Create/replace the property at the path.
                    let property_attrs = if path.contains(".")
                        || path.contains("[")
                    {
                        // For complex paths, navigate to the location and get
                        // property.
                        match navigate_property_path_mut(
                            dest_schema_properties,
                            path,
                        ) {
                            Ok(prop) => prop,
                            Err(e) => {
                                return Err(anyhow::anyhow!(
                                    "Failed to navigate to path {}: {}",
                                    path,
                                    e
                                ));
                            }
                        }
                    } else {
                        // For top-level properties, simply get or create the
                        // entry.
                        if dest_schema_properties.contains_key(path) {
                            dest_schema_properties.remove(path);
                        }

                        dest_schema_properties.entry(path.clone()).or_insert(ManifestApiPropertyAttributes {
                            prop_type: ValueType::Object, // Will be replaced based on value.
                            items: None,
                            properties: None,
                            required: None,
                        })
                    };

                    // Determine property type based on the value.
                    if value.is_i64() {
                        let int_val = value.as_i64().unwrap();

                        // Check if it fits within uint64.
                        if int_val >= 0 {
                            property_attrs.prop_type = ValueType::Uint64;
                        } else {
                            property_attrs.prop_type = ValueType::Int64;
                        }
                    } else if value.is_u64() {
                        property_attrs.prop_type = ValueType::Uint64;
                    } else if value.is_f64() {
                        property_attrs.prop_type = ValueType::Float64;
                    } else if value.is_boolean() {
                        property_attrs.prop_type = ValueType::Bool;
                    } else if value.is_string() {
                        property_attrs.prop_type = ValueType::String;
                    } else {
                        return Err(anyhow::anyhow!(
                            "Unsupported value type: {}",
                            value
                        ));
                    }
                }
            }
            MsgConversionMode::FromOriginal => {
                // For FromOriginal mode, we need to copy a property from the
                // source schema.
                if let Some(original_path) = &rule.original_path {
                    if let Some(src_prop) = navigate_property_path(
                        src_schema_properties,
                        original_path,
                    ) {
                        // Create/replace the property at the destination
                        // path.
                        let dest_path = &rule.path;

                        // For complex paths, we need to navigate and set
                        // the property.
                        if dest_path.contains(".") || dest_path.contains("[") {
                            match navigate_property_path_mut(
                                dest_schema_properties,
                                dest_path,
                            ) {
                                Ok(dest_prop) => {
                                    // Copy the source property attributes
                                    // to the destination.
                                    *dest_prop = src_prop.clone();
                                }
                                Err(e) => {
                                    return Err(anyhow::anyhow!(
                                            "Failed to navigate to destination path {}: {}",
                                            dest_path, e
                                        ));
                                }
                            }
                        } else {
                            // For top-level properties, simply set or
                            // replace the entry.
                            if dest_schema_properties.contains_key(dest_path) {
                                dest_schema_properties.remove(dest_path);
                            }
                            dest_schema_properties
                                .insert(dest_path.clone(), src_prop.clone());
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn get_msg_schema<'a>(
    graph_app_base_dir: &Option<String>,
    app: &Option<String>,
    extension_addon: &String,
    uri_to_pkg_info: &'a HashMap<Option<String>, PkgsInfoInAppWithBaseDir>,
    pkgs_cache: &'a HashMap<String, PkgsInfoInApp>,
    msg_direction: &MsgDirection,
    msg_type: &MsgType,
    msg_name: &str,
) -> Result<Option<&'a ManifestApiMsg>> {
    let msg_schema = if let Some(extension_pkg_info) =
        get_pkg_info_for_extension_addon(
            pkgs_cache,
            uri_to_pkg_info,
            graph_app_base_dir,
            app,
            extension_addon,
        ) {
        extension_pkg_info
            .manifest
            .api
            .as_ref()
            .and_then(|api| match msg_direction {
                MsgDirection::Out => match msg_type {
                    MsgType::Cmd => api.cmd_out.as_ref(),
                    MsgType::Data => api.data_out.as_ref(),
                    MsgType::AudioFrame => api.audio_frame_out.as_ref(),
                    MsgType::VideoFrame => api.video_frame_out.as_ref(),
                },
                MsgDirection::In => match msg_type {
                    MsgType::Cmd => api.cmd_in.as_ref(),
                    MsgType::Data => api.data_in.as_ref(),
                    MsgType::AudioFrame => api.audio_frame_in.as_ref(),
                    MsgType::VideoFrame => api.video_frame_in.as_ref(),
                },
            })
            .and_then(|msg_out| {
                msg_out.iter().find(|msg| msg.name == *msg_name)
            })
    } else {
        None
    };

    Ok(msg_schema)
}

#[allow(clippy::too_many_arguments)]
pub fn msg_conversion_get_final_target_schema(
    uri_to_pkg_info: &HashMap<Option<String>, PkgsInfoInAppWithBaseDir>,
    graph_app_base_dir: &Option<String>,
    pkgs_cache: &HashMap<String, PkgsInfoInApp>,
    src_app: &Option<String>,
    src_extension_addon: &String,
    dest_app: &Option<String>,
    dest_extension_addon: &String,
    msg_type: &MsgType,
    src_msg_name: &str,
    dest_msg_name: &str,
    ten_name_rule_index: Option<usize>,
    msg_conversion: &MsgAndResultConversion,
) -> Result<(ManifestApiMsg, Option<ManifestApiCmdResult>)> {
    // Get the source message schema.
    let src_msg_schema = get_msg_schema(
        graph_app_base_dir,
        src_app,
        src_extension_addon,
        uri_to_pkg_info,
        pkgs_cache,
        &MsgDirection::Out,
        msg_type,
        src_msg_name,
    )?;

    eprintln!(
        "src_msg_schema: {}",
        serde_json::to_string_pretty(&src_msg_schema).unwrap()
    );

    // Create a new message schema to store the converted properties.
    let mut converted_schema: ManifestApiMsg = ManifestApiMsg {
        name: dest_msg_name.to_string(),
        property: Some(HashMap::new()),
        required: None,
        result: None,
    };

    // If keep_original flag is true, start with the source message schema.
    if let Some(keep_original) = msg_conversion.msg.rules.keep_original {
        if keep_original {
            if let Some(src_msg_schema) = src_msg_schema {
                converted_schema = src_msg_schema.clone();
                converted_schema.required = src_msg_schema.required.clone();

                // Update the name to the destination message name.
                converted_schema.name = dest_msg_name.to_string();
            } else {
                // Not having a source msg schema is a normal situation, so even
                // if `keep_original` is true, we don't need to return an error.
            }
        }
    }

    // Ensure property map exists.
    if converted_schema.property.is_none() {
        converted_schema.property = Some(HashMap::new());
    }

    // Process each conversion rule.
    convert_rules_to_schema_properties(
        &msg_conversion.msg.rules.rules,
        ten_name_rule_index,
        src_msg_schema
            .as_ref()
            .and_then(|schema| schema.property.as_ref()),
        converted_schema.property.as_mut().unwrap(),
    )?;

    let mut converted_result_schema: Option<ManifestApiCmdResult> = None;

    if let Some(result_conversion) = &msg_conversion.result {
        // Create a new message schema to store the converted properties.
        converted_result_schema = Some(ManifestApiCmdResult {
            property: Some(HashMap::new()),
            required: None,
        });

        // If keep_original flag is true, start with the source message schema.
        if let Some(keep_original) = result_conversion.rules.keep_original {
            if keep_original {
                // If source message schema exists and has a result schema, use
                // it
                let src_result_schema = src_msg_schema
                    .as_ref()
                    .and_then(|schema| schema.result.as_ref());

                if let Some(result_schema) = src_result_schema {
                    converted_result_schema = Some(result_schema.clone());
                }

                // Not having a source msg schema is a normal situation, so even
                // if `keep_original` is true, we don't need to return an error.
            }
        }

        // Ensure property map exists.
        if converted_result_schema.as_ref().unwrap().property.is_none() {
            converted_result_schema.as_mut().unwrap().property =
                Some(HashMap::new());
        }

        let dest_msg_schema = get_msg_schema(
            graph_app_base_dir,
            dest_app,
            dest_extension_addon,
            uri_to_pkg_info,
            pkgs_cache,
            &MsgDirection::In,
            msg_type,
            dest_msg_name,
        )?;

        eprintln!(
            "dest_msg_schema: {}",
            serde_json::to_string_pretty(&dest_msg_schema).unwrap(),
        );

        convert_rules_to_schema_properties(
            &result_conversion.rules.rules,
            None,
            dest_msg_schema
                .as_ref()
                .and_then(|schema| schema.result.as_ref())
                .and_then(|result| result.property.as_ref()),
            converted_result_schema
                .as_mut()
                .unwrap()
                .property
                .as_mut()
                .unwrap(),
        )?;
    }

    Ok((converted_schema, converted_result_schema))
}

pub fn msg_conversion_get_dest_msg_name(
    src_msg_name: &str,
    msg_conversion: &MsgAndResultConversion,
) -> Result<(String, Option<usize>)> {
    let mut dest_msg_name = src_msg_name.to_string();
    let mut ten_name_rule_index = None;

    // Find the special `_ten.name` rule if it exists.
    for (index, rule) in msg_conversion.msg.rules.rules.iter().enumerate() {
        if rule.path == TEN_NAME_RULE_PATH
            && rule.conversion_mode == MsgConversionMode::FixedValue
        {
            if let Some(value) = &rule.value {
                if value.is_string() {
                    dest_msg_name =
                        value.as_str().unwrap_or(src_msg_name).to_string();
                    ten_name_rule_index = Some(index);
                    break;
                }
            }
        }
    }

    Ok((dest_msg_name, ten_name_rule_index))
}
