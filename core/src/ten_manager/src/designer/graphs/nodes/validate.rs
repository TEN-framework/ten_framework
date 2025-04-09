//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::sync::{Arc, RwLock};

use actix_web::{web, Responder};
use serde::{Deserialize, Serialize};

use ten_rust::pkg_info::pkg_type::PkgType;

use crate::designer::DesignerState;

#[derive(Serialize, Deserialize)]
pub struct ValidateGraphNodeRequestPayload {
    pub addon_app_base_dir: Option<String>,
    pub node_name: String,
    pub addon_name: String,
    pub extension_group_name: Option<String>,
    pub app_uri: Option<String>,
    pub property: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize)]
pub struct ValidateGraphNodeResponsePayload {
    pub success: bool,
}

/// Trait for request payloads that can be validated against extension schemas.
pub trait GraphNodeValidatable {
    fn get_addon_app_base_dir(&self) -> &Option<String>;
    fn get_addon_name(&self) -> &str;
    fn get_app_uri(&self) -> &Option<String>;
    fn get_property(&self) -> &Option<serde_json::Value>;
}

/// Validates a request payload against the addon extension schema.
pub fn validate_node_request<T: GraphNodeValidatable>(
    request: &T,
    state: &mut DesignerState,
) -> Result<(), String> {
    validate_node(
        request.get_addon_app_base_dir(),
        request.get_addon_name(),
        request.get_app_uri(),
        request.get_property(),
        state,
    )
}

pub fn validate_node(
    addon_app_base_dir: &Option<String>,
    addon_name: &str,
    app_uri: &Option<String>,
    property: &Option<serde_json::Value>,
    state: &DesignerState,
) -> Result<(), String> {
    // If addon_app_base_dir is not specified, skip the checks.
    if addon_app_base_dir.is_none() {
        return Ok(());
    }

    let addon_app_base_dir = addon_app_base_dir.as_ref().unwrap();

    // Find PkgsInfoInApp using addon_app_base_dir.
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
                let app_uri_matches = match (app_uri, &ten.uri) {
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
                      app_uri, ten.uri
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
                    && manifest.type_and_name.name == addon_name
                {
                    // Found matching extension - check property schema if it
                    // exists and request has property.
                    if let (Some(prop), Some(schema_store)) =
                        (property, &ext_pkg.schema_store)
                    {
                        if let Some(property_schema) = &schema_store.property {
                            if let Err(e) = property_schema.validate_json(prop)
                            {
                                return Err(format!(
                                    "Property validation failed: {}",
                                    e
                                ));
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
            addon_name
        ));
    }

    Ok(())
}

pub async fn validate_graph_node_endpoint(
    request_payload: web::Json<ValidateGraphNodeRequestPayload>,
    state: web::Data<Arc<RwLock<DesignerState>>>,
) -> Result<impl Responder, actix_web::Error> {
    let state_guard = state.read().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!(
            "Failed to acquire write lock: {}",
            e
        ))
    })?;

    let validation_result = validate_node(
        &request_payload.addon_app_base_dir,
        &request_payload.addon_name,
        &request_payload.app_uri,
        &request_payload.property,
        &state_guard,
    );

    match validation_result {
        Ok(()) => {
            let response = ValidateGraphNodeResponsePayload { success: true };
            Ok(web::Json(response))
        }
        Err(error_message) => {
            Err(actix_web::error::ErrorBadRequest(error_message))
        }
    }
}
