//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::{collections::HashMap, path::PathBuf};

use anyhow::Result;
use ten_rust::{
    base_dir_pkg_info::BaseDirPkgInfo, pkg_info::get_app_installed_pkgs,
};

/// Retrieves and caches all installed packages for the given app.
///
/// This function checks if package information for the specified base directory
/// already exists in the cache. If it does, it returns immediately. Otherwise,
/// it fetches the package information and stores it in the cache.
pub fn get_all_pkgs(
    pkgs_cache: &mut HashMap<String, BaseDirPkgInfo>,
    base_dir: &String,
) -> Result<()> {
    // Check whether the package information for the specified base_dir already
    // exists in the cache.
    if pkgs_cache.get(base_dir).is_some() {
        return Ok(());
    }

    // If there is no data in the cache, retrieve the package information and
    // store it in the cache.
    let app_path = PathBuf::from(base_dir);

    // Fetch package information.
    let result_pkgs = get_app_installed_pkgs(&app_path);

    match result_pkgs {
        Ok(pkgs) => {
            pkgs_cache.insert(base_dir.to_string(), pkgs);
            Ok(())
        }
        Err(err) => Err(err),
    }
}
