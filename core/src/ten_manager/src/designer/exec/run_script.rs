//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::sync::{Arc, RwLock};

use anyhow::Result;
use ten_rust::pkg_info::{pkg_type::PkgType, PkgInfo};

use crate::designer::DesignerState;

pub fn extract_command_from_manifest(
    base_dir: &String,
    name: &String,
    state: Arc<RwLock<DesignerState>>,
) -> Result<String> {
    let state = state.read().unwrap();

    let all_pkgs = state.pkgs_cache.get(base_dir).unwrap();

    // Find the package of type == app.
    let app_pkgs: Vec<&PkgInfo> = all_pkgs
        .iter()
        .filter(|p| {
            p.manifest
                .as_ref()
                .is_some_and(|m| m.type_and_name.pkg_type == PkgType::App)
        })
        .collect();

    if app_pkgs.len() != 1 {
        return Err(anyhow::anyhow!(
            "There should be exactly one app package, found 0 or more"
        ));
    }

    let app_pkg = app_pkgs[0];

    // Find script that matches `name`.
    let script_cmd = match app_pkg
        .manifest
        .as_ref()
        .and_then(|m| m.scripts.as_ref())
        .unwrap_or(&std::collections::HashMap::new())
        .get(name)
    {
        Some(cmd) => cmd.clone(),
        None => {
            return Err(anyhow::anyhow!(
                "Script '{}' not found in app manifest",
                name
            ));
        }
    };

    Ok(script_cmd)
}
