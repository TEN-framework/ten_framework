//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
pub mod found_result;
mod local;
mod pkg_cache;
pub mod pkg_list_cache;
mod remote;

use std::sync::Arc;

use anyhow::{anyhow, Result};
use found_result::PkgRegistryInfo;
use semver::{Version, VersionReq};
use tempfile::NamedTempFile;

use ten_rust::pkg_info::{pkg_type::PkgType, PkgInfo};

use super::config::TmanConfig;
use super::constants::DEFAULT;
use crate::output::TmanOutput;

pub async fn upload_package(
    tman_config: Arc<TmanConfig>,
    package_file_path: &str,
    pkg_info: &PkgInfo,
    out: Arc<Box<dyn TmanOutput>>,
) -> Result<String> {
    let default_registry_url = tman_config
        .registry
        .get(DEFAULT)
        .ok_or_else(|| anyhow!("Default registry not found"))?
        .index
        .clone();

    let parsed_registry_url = match url::Url::parse(&default_registry_url) {
        Ok(url) => url,
        Err(_) => return Err(anyhow!("Invalid URL: {}", default_registry_url)),
    };

    match parsed_registry_url.scheme() {
        "file" => {
            local::upload_package(
                &default_registry_url,
                package_file_path,
                pkg_info,
                out,
            )
            .await
        }
        "https" => {
            remote::upload_package(
                tman_config,
                &default_registry_url,
                package_file_path,
                pkg_info,
                out,
            )
            .await
        }
        _ => Err(anyhow!(
            "Unrecognized URL scheme: {}",
            parsed_registry_url.scheme()
        )),
    }
}

pub async fn get_package(
    tman_config: Arc<TmanConfig>,
    pkg_type: &PkgType,
    pkg_name: &str,
    pkg_version: &Version,
    url: &str,
    temp_path: &mut NamedTempFile,
    out: Arc<Box<dyn TmanOutput>>,
) -> Result<()> {
    let parsed_url =
        url::Url::parse(url).map_err(|_| anyhow!("Invalid URL: {}", url))?;

    match parsed_url.scheme() {
        "file" => {
            local::get_package(
                tman_config,
                pkg_type,
                pkg_name,
                pkg_version,
                url,
                temp_path,
                out,
            )
            .await
        }
        "https" => {
            remote::get_package(
                tman_config,
                pkg_type,
                pkg_name,
                pkg_version,
                url,
                temp_path,
                out,
            )
            .await
        }
        _ => Err(anyhow!("Failed to get package to any configured registry.")),
    }
}

/// Retrieves a list of packages from the registry that match the specified
/// criteria.
///
/// # Arguments
/// * `tman_config` - Configuration containing registry information.
/// * `pkg_type` - Type of package to search for (e.g., app, extension).
/// * `name` - Name of the package to search for.
/// * `version_req` - Version requirement to filter packages.
/// * `out` - Output interface for logging.
///
/// # Returns
/// A vector of `PkgRegistryInfo` containing information about matching
/// packages.
///
/// # Errors
/// * If the default registry is not configured.
/// * If the registry URL is invalid.
/// * If the URL scheme is not supported (only "file" and "https" are
///   supported).
/// * If there's an error retrieving the package list from the registry.
pub async fn get_package_list(
    tman_config: Arc<TmanConfig>,
    pkg_type: PkgType,
    name: &String,
    version_req: &VersionReq,
    out: Arc<Box<dyn TmanOutput>>,
) -> Result<Vec<PkgRegistryInfo>> {
    // Retrieve the default registry URL from configuration.
    let default_registry_url = tman_config
        .registry
        .get(DEFAULT)
        .ok_or_else(|| anyhow!("Default registry not found"))?
        .index
        .clone();

    // Parse the registry URL to determine the scheme (file or https).
    let parsed_registry_url = url::Url::parse(&default_registry_url)
        .map_err(|_| anyhow!("Invalid URL: {}", default_registry_url))?;

    // Delegate to the appropriate handler based on the URL scheme.
    let results = match parsed_registry_url.scheme() {
        "file" => {
            local::get_package_list(
                tman_config.clone(),
                &default_registry_url,
                pkg_type,
                name,
                version_req,
                out,
            )
            .await?
        }
        "https" => {
            remote::get_package_list(
                tman_config.clone(),
                &default_registry_url,
                pkg_type,
                name,
                version_req,
                out,
            )
            .await?
        }
        _ => {
            return Err(anyhow!(
                "Unsupported URL scheme: {}",
                parsed_registry_url.scheme()
            ));
        }
    };

    Ok(results)
}

pub async fn delete_package(
    tman_config: Arc<TmanConfig>,
    pkg_type: PkgType,
    name: &String,
    version: &Version,
    hash: &String,
    out: Arc<Box<dyn TmanOutput>>,
) -> Result<()> {
    // Retrieve the default registry URL.
    let default_registry_url = tman_config
        .registry
        .get(DEFAULT)
        .ok_or_else(|| anyhow!("Default registry not found"))?
        .index
        .clone();

    let parsed_registry_url = url::Url::parse(&default_registry_url)
        .map_err(|_| anyhow!("Invalid URL: {}", default_registry_url))?;

    match parsed_registry_url.scheme() {
        "file" => {
            local::delete_package(
                &default_registry_url,
                pkg_type,
                name,
                version,
                hash,
                out,
            )
            .await
        }
        "https" => {
            remote::delete_package(
                tman_config,
                &default_registry_url,
                pkg_type,
                name,
                version,
                hash,
                out,
            )
            .await
        }
        _ => Err(anyhow!(
            "Unsupported URL scheme: {}",
            parsed_registry_url.scheme()
        )),
    }
}
