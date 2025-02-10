//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use semver::Version;
use ten_rust::pkg_info::pkg_type::PkgType;

use crate::config::get_default_package_cache_folder;

/// Search under `package_cache` to see if there is a corresponding file for
/// `pkg_type/pkg_name/pkg_version/`. If found, return its path; otherwise,
/// return `None`."
pub fn find_in_package_cache(
    pkg_type: &PkgType,
    name: &str,
    version: &Version,
    file_name: &str,
) -> Result<Option<PathBuf>> {
    let cache_dir = get_default_package_cache_folder();
    let pkg_file = cache_dir
        .join(pkg_type.to_string())
        .join(name)
        .join(version.to_string())
        .join(file_name);

    if !pkg_file.exists() {
        return Ok(None);
    }

    Ok(Some(pkg_file))
}

/// Place the newly retrieved package file into
/// `package_cache/pkg_type/pkg_name/pkg_version/`. The filename should remain
/// the same as the original.
pub fn store_file_to_package_cache(
    pkg_type: &PkgType,
    name: &str,
    version: &Version,
    file_name: &str,
    downloaded_path: &Path,
) -> Result<()> {
    let cache_dir = get_default_package_cache_folder();
    let pkg_dir = cache_dir
        .join(pkg_type.to_string())
        .join(name)
        .join(version.to_string());

    fs::create_dir_all(&pkg_dir).with_context(|| {
        format!("Failed to create cache dir {}", pkg_dir.display())
    })?;

    let dest_path = pkg_dir.join(file_name);

    fs::copy(downloaded_path, &dest_path).with_context(|| {
        format!("Failed to copy into cache {}", dest_path.display())
    })?;

    Ok(())
}
