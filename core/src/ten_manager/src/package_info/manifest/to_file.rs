//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::fs::OpenOptions;
use std::path::Path;

use anyhow::Result;
use serde_json::Value;

use ten_rust::pkg_info::{
    constants::MANIFEST_JSON_FILENAME, manifest::dependency::ManifestDependency,
};

/// Updates the all_fields map in the manifest and writes it to a file.
///
/// This function preserves the order of existing dependencies and appends new
/// ones if needed. It is used to update the manifest.json file with new
/// dependencies.
pub fn update_manifest_all_fields(
    pkg_url: &str,
    manifest_all_fields: &mut serde_json::Map<String, Value>,
    new_dependency: Option<ManifestDependency>,
    is_present: bool,
    deps_to_remove: &[usize],
    dependencies: Option<&Vec<ManifestDependency>>,
    local_path_if_any: Option<&String>,
) -> Result<()> {
    if !is_present && new_dependency.is_some() {
        // We need to add a new dependency to all_fields.
        if let Some(Value::Array(dependencies_array)) =
            manifest_all_fields.get_mut("dependencies")
        {
            // all_fields already has a dependencies array, append new
            // dependency.
            let new_dep_value = if let Some(local_path) = local_path_if_any {
                // Create JSON for local dependency.
                let mut dep_obj = serde_json::Map::new();
                dep_obj.insert(
                    "path".to_string(),
                    Value::String(local_path.clone()),
                );
                Value::Object(dep_obj)
            } else {
                // Create JSON for registry dependency using
                // ManifestDependency::from.
                serde_json::to_value(new_dependency.unwrap())?
            };

            // Append the new dependency to the existing array.
            dependencies_array.push(new_dep_value);
        } else {
            // No dependencies array in all_fields yet, create a new one.
            let mut dependencies_array = Vec::new();

            // Add the new dependency to the array.
            let new_dep_value = if let Some(local_path) = local_path_if_any {
                // Create JSON for local dependency.
                let mut dep_obj = serde_json::Map::new();
                dep_obj.insert(
                    "path".to_string(),
                    Value::String(local_path.clone()),
                );
                Value::Object(dep_obj)
            } else {
                // Create JSON for registry dependency using
                // ManifestDependency::from.
                serde_json::to_value(new_dependency.unwrap())?
            };

            dependencies_array.push(new_dep_value);
            manifest_all_fields.insert(
                "dependencies".to_string(),
                Value::Array(dependencies_array),
            );
        }
    } else if !deps_to_remove.is_empty() {
        // We need to remove some dependencies from all_fields.
        if let Some(Value::Array(dependencies_array)) =
            manifest_all_fields.get_mut("dependencies")
        {
            // Create a new array with the dependencies to keep.
            let mut new_deps_array = Vec::new();

            // Get the updated dependencies as JSON values.
            if let Some(deps) = dependencies {
                for dep in deps {
                    let dep_value = serde_json::to_value(dep)?;
                    new_deps_array.push(dep_value);
                }
            }

            // Replace the old array with the new one.
            *dependencies_array = new_deps_array;
        }
    }

    // Write the updated manifest back to the file
    let manifest_path = Path::new(pkg_url).join(MANIFEST_JSON_FILENAME);
    let manifest_file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(manifest_path)?;

    // Serialize the all_fields map directly to preserve field order.
    serde_json::to_writer_pretty(manifest_file, &manifest_all_fields)?;

    Ok(())
}
