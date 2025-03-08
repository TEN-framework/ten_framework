//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::path::PathBuf;

use anyhow::Result;
use ten_rust::pkg_info::PkgInfo;

use super::DesignerState;
use crate::package_info::tman_get_all_installed_pkgs_info_of_app;

/// Retrieves and caches all installed packages for the given application.
///
/// # Arguments
///
/// * `state` - A mutable reference to the designer state containing the base
///   directory and other relevant configuration.
///
/// # Errors
///
/// Returns an error if the base directory is not set or if fetching package
/// information fails.
pub fn get_all_pkgs(
    state: &mut DesignerState,
    base_dir: &String,
) -> Result<Vec<PkgInfo>> {
    // Check whether the package information for the specified **base_dir**
    // already exists in the cache.
    if let Some(cached_pkgs) = state.pkgs_cache.get(base_dir) {
        return Ok(cached_pkgs.clone());
    }

    // If there is no data in the cache, attempt to retrieve the package
    // information.
    let app_path = PathBuf::from(base_dir);
    match tman_get_all_installed_pkgs_info_of_app(
        &state.tman_config,
        &app_path,
        state.out.clone(),
    ) {
        Ok(pkgs) => {
            state.pkgs_cache.insert(base_dir.to_string(), pkgs.clone());
            return Ok(pkgs);
        }
        Err(err) => {
            return Err(err);
        }
    }
}
