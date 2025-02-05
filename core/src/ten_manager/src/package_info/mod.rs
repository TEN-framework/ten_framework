//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
pub mod predefined_graphs;

use std::path::Path;

use anyhow::Result;

use ten_rust::pkg_info::{
    get_all_installed_pkgs_info_of_app_to_hashmap, PkgInfo,
};

pub fn tman_get_all_installed_pkgs_info_of_app(
    app_path: &Path,
) -> Result<Vec<PkgInfo>> {
    let pkgs_info = get_all_installed_pkgs_info_of_app_to_hashmap(app_path)?;
    Ok(pkgs_info.into_values().collect())
}
