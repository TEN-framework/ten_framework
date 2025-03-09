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
    package_info::tman_get_all_installed_pkgs_info_of_app,
};

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
    tman_config: &TmanConfig,
    pkgs_cache: &mut HashMap<String, Vec<PkgInfo>>,
    base_dir: &String,
    out: &Arc<Box<dyn TmanOutput>>,
) -> Result<()> {
    // Check whether the package information for the specified **base_dir**
    // already exists in the cache.
    if pkgs_cache.get(base_dir).is_some() {
        return Ok(());
    }

    // If there is no data in the cache, attempt to retrieve the package
    // information.
    let app_path = PathBuf::from(base_dir);
    match tman_get_all_installed_pkgs_info_of_app(
        tman_config,
        &app_path,
        out.clone(),
    ) {
        Ok(pkgs) => {
            pkgs_cache.insert(base_dir.to_string(), pkgs.clone());
            Ok(())
        }
        Err(err) => Err(err),
    }
}
