//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use ten_rust::pkg_info::pkg_type::PkgType;

use crate::designer::DesignerState;

/// Trait for request payloads that can be validated against extension schemas.
pub trait ExtensionSchemaValidatable {
    fn get_addon_app_base_dir(&self) -> &Option<String>;
    fn get_addon_name(&self) -> &str;
    fn get_app_uri(&self) -> &Option<String>;
    fn get_property(&self) -> &Option<serde_json::Value>;
}

/// Validates a request payload against the addon extension schema.
pub fn validate_node_request<T: ExtensionSchemaValidatable>(
    request: &T,
    state: &mut DesignerState,
) -> Result<(), String> {
    // If addon_app_base_dir is not specified, skip the checks.
    if request.get_addon_app_base_dir().is_none() {
        return Ok(());
    }

    let addon_app_base_dir = request.get_addon_app_base_dir().as_ref().unwrap();

    // Find BaseDirPkgInfo using addon_app_base_dir.
    let base_dir_pkg_info = match state.pkgs_cache.get(addon_app_base_dir) {
        Some(pkg_info) => pkg_info,
        None => {
            return Err(format!(
                "Addon app base directory '{}' not found in pkgs_cache",
                addon_app_base_dir
            ))
        }
    };

    // Check that app_uri matches with the app_pkg_info's uri if set.
    if let Some(app_pkg_info) = &base_dir_pkg_info.app_pkg_info {
        if let Some(property) = &app_pkg_info.property {
            if let Some(ten) = &property._ten {
                let app_uri_matches = match (request.get_app_uri(), &ten.uri) {
                    // Both sides have no URI - OK.
                    (None, None) => true,
                    // Both have URI - must match.
                    (Some(req_uri), Some(app_uri)) => req_uri == app_uri,
                    // One side has URI and the other doesn't - not OK.
                    _ => false,
                };

                if !app_uri_matches {
                    return Err(format!(
                        "Request app_uri {:?} doesn't match app package uri {:?}",
                        request.get_app_uri(), ten.uri
                    ));
                }
            }
        }
    }

    // Find extension PkgInfo and validate property against json schema if
    // present.
    if let Some(extensions) = &base_dir_pkg_info.extension_pkg_info {
        for ext_pkg in extensions {
            if let Some(manifest) = &ext_pkg.manifest {
                if manifest.type_and_name.pkg_type == PkgType::Extension
                    && manifest.type_and_name.name == request.get_addon_name()
                {
                    // Found matching extension - check property schema if it
                    // exists.
                    if let Some(api) = &manifest.api {
                        if let Some(property_schema) = &api.property {
                            eprintln!(
                                "=-=-= property_schema: {:?}",
                                property_schema
                            );

                            // If request payload has property and extension has
                            // schema, validate.
                            if let Some(property) = request.get_property() {
                                // Convert property_schema to a JSON Schema
                                // document.
                                let schema_json = serde_json::json!({
                                    "type": "object",
                                    "properties": property_schema,
                                    "additionalProperties": false
                                });

                                // Create validator from schema.
                                let validator = match jsonschema::Validator::new(
                                    &schema_json,
                                ) {
                                    Ok(v) => v,
                                    Err(e) => {
                                        return Err(format!(
                                        "Failed to compile property schema: {}",
                                        e
                                    ))
                                    }
                                };

                                // Validate property against schema.
                                if let Err(errors) =
                                    validator.validate(property)
                                {
                                    return Err(format!(
                                        "Property validation failed: {}",
                                        errors
                                    ));
                                }
                            }
                        }
                    }

                    return Ok(());
                }
            }
        }

        // If we got here, the extension wasn't found.
        return Err(format!(
            "Extension '{}' not found in addon_app_base_dir",
            request.get_addon_name()
        ));
    }

    Ok(())
}
