//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use anyhow::{anyhow, Context, Result};
use semver::VersionReq;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use ten_rust::pkg_info::pkg_type::PkgType;

use crate::{
    config::TmanConfig,
    package_file::unpackage::extract_and_process_tpkg_file,
    registry::{get_package, get_package_list, SearchCriteria},
};

fn can_package_be_created_in_path(
    path: &Path,
    pkg_name: &String,
) -> Result<()> {
    let target: PathBuf = path.join(pkg_name);

    if target.exists() {
        Err(anyhow!(
            "Cannot create package '{}': '{}' already contains a file or folder with the same name.",
            pkg_name,
            path.display()
        ))
    } else {
        Ok(())
    }
}

pub async fn create_pkg_in_path(
    tman_config: &TmanConfig,
    path: &Path,
    pkg_type: &PkgType,
    pkg_name: &String,
    template_pkg_name: &String,
    template_pkg_version: &VersionReq,
    template_data: Option<&HashMap<String, String>>,
) -> Result<()> {
    // Check that 'path' is a directory.
    if !path.is_dir() {
        return Err(anyhow!(
            "The provided path '{}' is not a directory.",
            path.display()
        ));
    }

    can_package_be_created_in_path(path, pkg_name)?;

    let search_criteria = SearchCriteria {
        // Ensure exact version matching.
        version_req: template_pkg_version.clone(),
    };

    // Search for the package in the registry.
    let mut found_packages = get_package_list(
        tman_config,
        *pkg_type,
        template_pkg_name,
        &search_criteria,
    )
    .await?;

    // Handle case where the package is not found in the registry.
    if found_packages.is_empty() {
        return Err(anyhow!(
            "Package '{}:{}@{}' not found in the registry.",
            pkg_type,
            template_pkg_name,
            template_pkg_version
        ));
    }

    // Get the latest package that meets the requirements.
    found_packages.sort_by(|a, b| {
        b.pkg_registry_info
            .basic_info
            .version
            .cmp(&a.pkg_registry_info.basic_info.version)
    });

    let package = &found_packages[0];
    let package_url = package
        .url
        .to_str()
        .ok_or_else(|| anyhow!("Invalid package URL"))?;

    // Download the package from the registry.
    let mut temp_file = tempfile::NamedTempFile::new().context(
        "Failed to create a temporary file for downloading the package",
    )?;
    get_package(tman_config, package_url, &mut temp_file)
        .await
        .context("Failed to download the package from the registry")?;

    // Define the target directory where the package will be extracted.
    let target_path = path.join(pkg_name);

    // Create the target directory.
    std::fs::create_dir_all(&target_path).with_context(|| {
        format!("Failed to create directory '{}'", target_path.display())
    })?;

    // Convert template_data to serde_json::Value if present.
    let template_ctx = template_data.map(|data| {
        serde_json::to_value(data)
            .expect("Failed to convert template data to JSON")
    });

    // Extract the downloaded package into the target directory.
    extract_and_process_tpkg_file(
        temp_file.path(),
        target_path.to_str().unwrap(),
        template_ctx.as_ref(),
    )
    .context("Failed to extract the package contents")?;

    Ok(())
}
