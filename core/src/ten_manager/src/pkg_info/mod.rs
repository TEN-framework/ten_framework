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
use ten_rust::{
    base_dir_pkg_info::PkgsInfoInApp,
    pkg_info::{get_app_installed_pkgs, pkg_type::PkgType, PkgInfo},
};

/// Retrieves information about all installed packages for the specified
/// application.
pub fn tman_get_all_installed_pkgs_info_of_app(
    tman_config: Arc<TmanConfig>,
    app_path: &Path,
    out: Arc<Box<dyn TmanOutput>>,
) -> Result<Vec<PkgInfo>> {
    let pkg_info_struct = get_app_installed_pkgs(app_path, false, &mut None)?;

    // Use the to_vec method to combine all package types into a single vector.
    let all_pkgs = pkg_info_struct.to_vec();

    if tman_config.verbose {
        out.normal_line(&format!("{:?}", all_pkgs));
    }
    Ok(all_pkgs)
}

pub fn get_pkg_in_app<'a>(
    pkgs_info_in_app: &'a PkgsInfoInApp,
    pkg_type: &PkgType,
    pkg_name: &Option<&String>,
) -> Result<Option<&'a PkgInfo>> {
    match pkg_type {
        PkgType::App => Ok(pkgs_info_in_app.app_pkg_info.as_ref()),
        PkgType::Extension => {
            assert!(pkg_name.is_some());

            match &pkgs_info_in_app.extension_pkg_info {
                Some(pkgs) => Ok(pkgs.iter().find(|pkg| {
                    pkg.manifest.as_ref().is_some_and(|m| {
                        m.type_and_name.name == *pkg_name.unwrap()
                    })
                })),
                None => Ok(None),
            }
        }
        _ => Err(anyhow::anyhow!("Unsupported package type: {:?}", pkg_type)),
    }
}
