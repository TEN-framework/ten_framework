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

/// Update the manifest file, handling both adding new dependencies and removing
/// existing ones.
///
/// Both `deps_to_add` and `deps_to_remove` are optional, allowing for:
///   - Adding new dependencies without removing any.
///   - Removing dependencies without adding any new ones.
///   - Both adding and removing dependencies in a single call.
///
/// In both cases, the original order of entries in the manifest file is
/// preserved.
pub fn update_manifest_all_fields(
    pkg_url: &str,
    manifest_all_fields: &mut serde_json::Map<String, Value>,
    deps_to_add: Option<&Vec<ManifestDependency>>,
    deps_to_remove: Option<&Vec<ManifestDependency>>,
) -> Result<()> {
    // Process dependencies array.
    if let Some(Value::Array(dependencies_array)) =
        manifest_all_fields.get_mut("dependencies")
    {
        // Remove dependencies if requested.
        if let Some(remove_deps) = deps_to_remove {
            if !remove_deps.is_empty() {
                // First, convert dependencies to remove into comparable form.
                let deps_to_remove_serialized: Vec<String> = remove_deps
                    .iter()
                    .filter_map(|dep| {
                        let dep_value = serde_json::to_value(dep).ok()?;
                        serde_json::to_string(&dep_value).ok()
                    })
                    .collect();

                // Filter out dependencies to remove.
                dependencies_array.retain(|item| {
                    if let Ok(item_str) = serde_json::to_string(item) {
                        !deps_to_remove_serialized.contains(&item_str)
                    } else {
                        true // Keep items that can't be serialized.
                    }
                });
            }
        }

        // Add new dependencies if provided.
        if let Some(new_deps) = deps_to_add {
            if !new_deps.is_empty() {
                // Append new dependencies to the existing array.
                for new_dep in new_deps {
                    if let Ok(new_dep_value) = serde_json::to_value(new_dep) {
                        dependencies_array.push(new_dep_value);
                    }
                }
            }
        }
    } else if let Some(new_deps) = deps_to_add {
        // No dependencies array in all_fields yet, create a new one if we have
        // new dependencies.
        if !new_deps.is_empty() {
            let mut dependencies_array = Vec::new();

            // Add all new dependencies to the array.
            for new_dep in new_deps {
                if let Ok(new_dep_value) = serde_json::to_value(new_dep) {
                    dependencies_array.push(new_dep_value);
                }
            }

            manifest_all_fields.insert(
                "dependencies".to_string(),
                Value::Array(dependencies_array),
            );
        }
    }

    // Write the updated manifest back to the file.
    let manifest_path = Path::new(pkg_url).join(MANIFEST_JSON_FILENAME);
    let manifest_file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(manifest_path)?;

    // Serialize the all_fields map directly to preserve field order.
    serde_json::to_writer_pretty(manifest_file, &manifest_all_fields)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use semver::VersionReq;
    use serde_json::{json, Map, Value};
    use std::fs;
    use std::io::Read;
    use tempfile::TempDir;
    use ten_rust::pkg_info::pkg_type::PkgType;

    #[test]
    fn test_update_manifest_preserves_field_order() -> Result<()> {
        // Create a temporary directory for our test
        let temp_dir = TempDir::new()?;
        let temp_path = temp_dir.path().to_str().unwrap().to_string();

        // Create a manifest with fields in a specific order
        let mut all_fields = Map::new();
        all_fields.insert("type".to_string(), json!("extension"));
        all_fields.insert("name".to_string(), json!("test-extension"));
        all_fields.insert("version".to_string(), json!("1.0.0"));
        all_fields.insert("description".to_string(), json!("Test Extension"));
        all_fields.insert("author".to_string(), json!("Test Author"));
        all_fields.insert("license".to_string(), json!("Apache-2.0"));

        // Initial dependencies with specific order
        let mut deps = Vec::new();
        deps.push(json!({
            "type": "extension",
            "name": "first-dependency",
            "version": "^1.0.0"
        }));
        deps.push(json!({
            "type": "extension",
            "name": "second-dependency",
            "version": "^2.0.0"
        }));
        deps.push(json!({
            "type": "extension",
            "name": "third-dependency",
            "version": "^3.0.0"
        }));
        all_fields.insert("dependencies".to_string(), Value::Array(deps));

        all_fields.insert("keywords".to_string(), json!(["test", "extension"]));

        // Write the initial manifest to the temp directory
        let manifest_path = Path::new(&temp_path).join(MANIFEST_JSON_FILENAME);
        let manifest_file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&manifest_path)?;
        serde_json::to_writer_pretty(manifest_file, &all_fields)?;

        // Create a new dependency to add
        let new_dep = ManifestDependency::RegistryDependency {
            pkg_type: PkgType::Extension,
            name: "new-dependency".to_string(),
            version_req: VersionReq::parse("^4.0.0").unwrap(),
        };
        let new_deps = vec![new_dep];

        // Create a dependency to remove (the second one)
        let remove_dep = ManifestDependency::RegistryDependency {
            pkg_type: PkgType::Extension,
            name: "second-dependency".to_string(),
            version_req: VersionReq::parse("^2.0.0").unwrap(),
        };
        let remove_deps = vec![remove_dep];

        // Update the manifest: add one dependency and remove one dependency in
        // a single call
        update_manifest_all_fields(
            &temp_path,
            &mut all_fields,
            Some(&new_deps),
            Some(&remove_deps),
        )?;

        // Read the updated manifest file
        let mut updated_manifest_content = String::new();
        let mut file = fs::File::open(&manifest_path)?;
        file.read_to_string(&mut updated_manifest_content)?;

        // Parse the updated manifest
        let updated_manifest: serde_json::Value =
            serde_json::from_str(&updated_manifest_content)?;

        // Verify the field order is maintained
        if let Value::Object(map) = updated_manifest {
            let keys: Vec<&String> = map.keys().collect();

            // Check that the order of fields matches our original order
            let expected_order = [
                "type",
                "name",
                "version",
                "description",
                "author",
                "license",
                "dependencies",
                "keywords",
            ];

            for (i, expected_key) in expected_order.iter().enumerate() {
                assert_eq!(
                    keys[i], expected_key,
                    "Field order not preserved at position {}",
                    i
                );
            }

            // Verify the dependencies were updated correctly and order is
            // preserved
            if let Value::Array(deps) = &map["dependencies"] {
                assert_eq!(deps.len(), 3, "Should have 3 dependencies");

                // Check the first dependency is still first
                if let Value::Object(first_dep) = &deps[0] {
                    assert_eq!(
                        first_dep["name"], "first-dependency",
                        "First dependency order not preserved"
                    );
                } else {
                    panic!("First dependency is not an object");
                }

                // Check that second dependency is now the third one (since
                // second was removed)
                if let Value::Object(second_dep) = &deps[1] {
                    assert_eq!(
                        second_dep["name"], "third-dependency",
                        "Third dependency should now be in position 2"
                    );
                } else {
                    panic!("Second dependency position is not an object");
                }

                // Check that the new dependency is at the end
                if let Value::Object(third_dep) = &deps[2] {
                    assert_eq!(
                        third_dep["name"], "new-dependency",
                        "New dependency should be at the end"
                    );
                } else {
                    panic!("Third dependency position is not an object");
                }
            } else {
                panic!("Dependencies is not an array");
            }
        } else {
            panic!("Updated manifest is not an object");
        }

        // Test removing all dependencies
        let all_remove_deps = vec![
            ManifestDependency::RegistryDependency {
                pkg_type: PkgType::Extension,
                name: "first-dependency".to_string(),
                version_req: VersionReq::parse("^1.0.0").unwrap(),
            },
            ManifestDependency::RegistryDependency {
                pkg_type: PkgType::Extension,
                name: "third-dependency".to_string(),
                version_req: VersionReq::parse("^3.0.0").unwrap(),
            },
            ManifestDependency::RegistryDependency {
                pkg_type: PkgType::Extension,
                name: "new-dependency".to_string(),
                version_req: VersionReq::parse("^4.0.0").unwrap(),
            },
        ];

        // Update the manifest to remove all dependencies
        update_manifest_all_fields(
            &temp_path,
            &mut all_fields,
            None,
            Some(&all_remove_deps),
        )?;

        // Read the updated manifest file again
        let mut updated_manifest_content = String::new();
        let mut file = fs::File::open(&manifest_path)?;
        file.read_to_string(&mut updated_manifest_content)?;

        // Parse the updated manifest
        let updated_manifest: serde_json::Value =
            serde_json::from_str(&updated_manifest_content)?;

        // Verify the field order is still maintained and dependencies are empty
        if let Value::Object(map) = updated_manifest {
            let keys: Vec<&String> = map.keys().collect();

            // Check that the order of fields matches our original order
            let expected_order = [
                "type",
                "name",
                "version",
                "description",
                "author",
                "license",
                "dependencies",
                "keywords",
            ];

            for (i, expected_key) in expected_order.iter().enumerate() {
                assert_eq!(
                    keys[i], expected_key,
                    "Field order not preserved after removal at position {}",
                    i
                );
            }

            // Verify dependencies array is empty
            if let Value::Array(deps) = &map["dependencies"] {
                assert_eq!(deps.len(), 0, "All dependencies should be removed");
            } else {
                panic!("Dependencies is not an array");
            }
        } else {
            panic!("Updated manifest is not an object");
        }

        Ok(())
    }
}
