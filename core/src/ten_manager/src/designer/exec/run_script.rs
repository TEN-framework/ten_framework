//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::sync::{Arc, RwLock};

use anyhow::Result;

use crate::designer::DesignerState;

pub fn extract_command_from_manifest(
    base_dir: &String,
    name: &String,
    state: Arc<RwLock<DesignerState>>,
) -> Result<String> {
    let state = state.read().unwrap();

    let base_dir_pkg_info = state.pkgs_cache.get(base_dir).unwrap();

    // Get the app package directly from PkgsInfoInApp.
    let app_pkg = match &base_dir_pkg_info.app_pkg_info {
        Some(app_pkg) => app_pkg,
        None => {
            return Err(anyhow::anyhow!(
                "There should be exactly one app package, found 0"
            ));
        }
    };

    // Find script that matches `name`.
    let script_cmd = match app_pkg
        .manifest
        .scripts
        .as_ref()
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
