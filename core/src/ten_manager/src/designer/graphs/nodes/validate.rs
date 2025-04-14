//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use ten_rust::pkg_info::PkgInfo;

/// Trait for request payloads that can be validated against extension schemas.
pub trait GraphNodeValidatable {
    fn get_addon_name(&self) -> &str;
    fn get_app_uri(&self) -> &Option<String>;
    fn get_property(&self) -> &Option<serde_json::Value>;
}

fn validate_node(
    property: &Option<serde_json::Value>,
    extension_pkg_info: &PkgInfo,
) -> Result<(), String> {
    // Found matching extension - check property schema if it
    // exists and request has property.
    if let (Some(prop), Some(schema_store)) =
        (property, &extension_pkg_info.schema_store)
    {
        if let Some(property_schema) = &schema_store.property {
            if let Err(e) = property_schema.validate_json(prop) {
                return Err(format!("Property validation failed: {}", e));
            }
        }
    }

    Ok(())
}

/// Validates a request payload against the addon extension schema.
pub fn validate_node_request<T: GraphNodeValidatable>(
    request: &T,
    extension_pkg_info: &PkgInfo,
) -> Result<(), String> {
    validate_node(request.get_property(), extension_pkg_info)
}
