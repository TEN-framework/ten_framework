//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::{collections::HashMap, path::PathBuf, sync::Arc};

use anyhow::Result;
use ten_rust::pkg_info::PkgInfo;

use crate::{
    config::TmanConfig, output::TmanOutput,
    pkg_info::tman_get_all_installed_pkgs_info_of_app,
};

/// Retrieves and caches all installed packages for the given app.
///
/// This function checks if package information for the specified base directory
/// already exists in the cache. If it does, it returns immediately. Otherwise,
/// it fetches the package information and stores it in the cache.
///
/// # Arguments
///
/// * `tman_config` - Configuration for the TEN manager.
/// * `pkgs_cache` - A mutable reference to the package cache, mapping base
///   directories to lists of packages.
/// * `base_dir` - The base directory of the app to retrieve packages for.
/// * `out` - Output interface for logging and displaying information.
///
/// # Errors
///
/// Returns an error if fetching package information fails, which can happen if:
/// - The base directory doesn't exist or is invalid.
/// - There are issues reading package manifests.
/// - Package information cannot be parsed correctly.
pub fn get_all_pkgs(
    tman_config: Arc<TmanConfig>,
    pkgs_cache: &mut HashMap<String, Vec<PkgInfo>>,
    base_dir: &String,
    out: &Arc<Box<dyn TmanOutput>>,
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
    let result = tman_get_all_installed_pkgs_info_of_app(
        tman_config,
        &app_path,
        out.clone(),
    );

    match result {
        Ok(pkgs) => {
            // Store the packages in the cache using the base directory as the
            // key.
            pkgs_cache.insert(base_dir.to_string(), pkgs);
            Ok(())
        }
        Err(err) => Err(err),
    }
}
