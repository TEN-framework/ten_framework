//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::{collections::HashMap, path::PathBuf, sync::Arc};

use anyhow::Result;
use ten_rust::base_dir_pkg_info::BaseDirPkgInfo;
use ten_rust::pkg_info::pkg_type::PkgType;
use ten_rust::pkg_info::PkgInfo;

use crate::{
    config::TmanConfig, output::TmanOutput,
    pkg_info::tman_get_all_installed_pkgs_info_of_app,
};

/// Converts a Vec<PkgInfo> to BaseDirPkgInfo by categorizing packages by type
fn convert_to_base_dir_pkg_info(pkgs: Vec<PkgInfo>) -> BaseDirPkgInfo {
    let mut app_pkg_info = None;
    let mut extension_pkg_info = Vec::new();
    let mut protocol_pkg_info = Vec::new();
    let mut addon_loader_pkg_info = Vec::new();
    let mut system_pkg_info = Vec::new();

    for pkg in pkgs {
        if let Some(manifest) = &pkg.manifest {
            match manifest.type_and_name.pkg_type {
                PkgType::App => {
                    app_pkg_info = Some(pkg);
                }
                PkgType::Extension => {
                    extension_pkg_info.push(pkg);
                }
                PkgType::Protocol => {
                    protocol_pkg_info.push(pkg);
                }
                PkgType::AddonLoader => {
                    addon_loader_pkg_info.push(pkg);
                }
                PkgType::System => {
                    system_pkg_info.push(pkg);
                }
                PkgType::Invalid => {}
            }
        }
    }

    BaseDirPkgInfo {
        app_pkg_info,
        extension_pkg_info: if extension_pkg_info.is_empty() {
            None
        } else {
            Some(extension_pkg_info)
        },
        protocol_pkg_info: if protocol_pkg_info.is_empty() {
            None
        } else {
            Some(protocol_pkg_info)
        },
        addon_loader_pkg_info: if addon_loader_pkg_info.is_empty() {
            None
        } else {
            Some(addon_loader_pkg_info)
        },
        system_pkg_info: if system_pkg_info.is_empty() {
            None
        } else {
            Some(system_pkg_info)
        },
    }
}

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
///   directories to BaseDirPkgInfo.
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
    pkgs_cache: &mut HashMap<String, BaseDirPkgInfo>,
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
    let result_pkgs = tman_get_all_installed_pkgs_info_of_app(
        tman_config,
        &app_path,
        out.clone(),
    );

    match result_pkgs {
        Ok(pkgs) => {
            // Convert Vec<PkgInfo> to BaseDirPkgInfo and store in the cache
            let base_dir_pkg_info = convert_to_base_dir_pkg_info(pkgs);
            pkgs_cache.insert(base_dir.to_string(), base_dir_pkg_info);
            Ok(())
        }
        Err(err) => Err(err),
    }
}
