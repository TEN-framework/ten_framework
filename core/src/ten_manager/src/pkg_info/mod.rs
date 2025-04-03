//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
pub mod get_all_pkgs;
pub mod manifest;

use std::{path::Path, sync::Arc};

use anyhow::Result;

use crate::{config::TmanConfig, output::TmanOutput};
use ten_rust::pkg_info::{get_app_installed_pkgs, PkgInfo};

/// Retrieves information about all installed packages for the specified
/// application.
pub fn tman_get_all_installed_pkgs_info_of_app(
    tman_config: Arc<TmanConfig>,
    app_path: &Path,
    out: Arc<Box<dyn TmanOutput>>,
) -> Result<Vec<PkgInfo>> {
    let pkg_info_struct = get_app_installed_pkgs(app_path)?;

    // Combine all package types into a single vector.
    let mut all_pkgs = Vec::new();
    if let Some(app_info) = &pkg_info_struct.app_pkg_info {
        all_pkgs.push(app_info.clone());
    }
    if let Some(ext_info) = &pkg_info_struct.extension_pkg_info {
        all_pkgs.extend(ext_info.clone());
    }
    if let Some(proto_info) = &pkg_info_struct.protocol_pkg_info {
        all_pkgs.extend(proto_info.clone());
    }
    if let Some(addon_info) = &pkg_info_struct.addon_loader_pkg_info {
        all_pkgs.extend(addon_info.clone());
    }
    if let Some(sys_info) = &pkg_info_struct.system_pkg_info {
        all_pkgs.extend(sys_info.clone());
    }

    if tman_config.verbose {
        out.normal_line(&format!("{:?}", all_pkgs));
    }
    Ok(all_pkgs)
}
