//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;

use anyhow::{anyhow, Context, Result};
use console::Emoji;
use semver::{Version, VersionReq};
use serde_json;
use sha2::{Digest, Sha256};
use tempfile::NamedTempFile;
use walkdir::WalkDir;

use ten_rust::pkg_info::manifest::Manifest;
use ten_rust::pkg_info::pkg_type::PkgType;
use ten_rust::pkg_info::PkgInfo;

use super::found_result::{
    get_pkg_registry_info_from_manifest, PkgRegistryInfo,
};
use super::pkg_cache::{find_in_package_cache, store_file_to_package_cache};
use crate::config::{is_verbose, TmanConfig};
use crate::constants::{
    DEFAULT_REGISTRY_PAGE_SIZE, TEN_PACKAGE_FILE_EXTENSION,
};
use crate::output::TmanOutput;

pub async fn upload_package(
    base_url: &str,
    package_file_path: &str,
    pkg_info: &PkgInfo,
    _out: Arc<Box<dyn TmanOutput>>,
) -> Result<String> {
    let mut path_url = url::Url::parse(base_url)
        .map_err(|e| anyhow!("Invalid file URL: {}", e))?
        .to_file_path()
        .map_err(|_| anyhow!("Failed to convert file URL to path"))?
        .to_string_lossy()
        .into_owned();

    // Ensure the base URL ends with a '/'.
    path_url = if path_url.ends_with('/') {
        path_url.to_string()
    } else {
        format!("{}/", path_url)
    };

    // Construct the directory path.
    let dir_path = PathBuf::from(format!(
        "{}{}/{}/{}/",
        path_url,
        pkg_info.manifest.type_and_name.pkg_type,
        pkg_info.manifest.type_and_name.name,
        pkg_info.manifest.version.clone()
    ));

    // Check if the directory exists, and only create it if it doesn't.
    if !dir_path.exists() {
        fs::create_dir_all(&dir_path).with_context(|| {
            format!("Failed to create directory '{}'", dir_path.display())
        })?;
    }

    // Construct the full file path for the new location.
    let file_stem = Path::new(package_file_path)
        .file_stem()
        .ok_or_else(|| anyhow::anyhow!("Invalid file path provided"))?
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("Invalid unicode in file path"))?;

    let extension = Path::new(package_file_path)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| format!(".{}", ext))
        .unwrap_or_default();

    let new_file_name = format!("{}_{}{}", file_stem, pkg_info.hash, extension);

    let full_path = dir_path.join(new_file_name);

    // Copy the file to the new path.
    fs::copy(package_file_path, &full_path).with_context(|| {
        format!(
            "Failed to copy file from '{}' to '{}'",
            package_file_path,
            full_path.display()
        )
    })?;

    // Serialize and write the manifest to a JSON file.
    let manifest_json = serde_json::to_string_pretty(&pkg_info.manifest)
        .with_context(|| "Failed to serialize manifest to JSON")?;

    let manifest_file_name =
        format!("{}_{}_manifest.json", file_stem, pkg_info.hash);
    let manifest_path = dir_path.join(manifest_file_name);

    fs::write(&manifest_path, manifest_json).with_context(|| {
        format!("Failed to write manifest to '{}'", manifest_path.display())
    })?;

    Ok(full_path.to_string_lossy().to_string())
}

/// Calculate the hash of the file content to determine whether the file content
/// is the same when using the local registry.
fn calc_file_hash(path: &Path) -> Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];

    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }

    let hash = hasher.finalize();
    Ok(format!("{:x}", hash))
}

/// Check if a file exists and is a regular file.
pub fn check_file_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        return Err(anyhow::anyhow!("File does not exist: {}", path.display()));
    }
    if !path.is_file() {
        return Err(anyhow::anyhow!("Path is not a file: {}", path.display()));
    }
    Ok(())
}

/// Extract the filename from a path.
pub fn extract_filename_from_path(path: &Path) -> Option<String> {
    path.file_name().map(|f| f.to_string_lossy().to_string())
}

/// Determine whether the locally cached file and the target file in the local
/// registry have the same hash.
fn is_same_file_by_hash(
    cache_file: &Path,
    registry_file_url: &str,
) -> Result<bool> {
    let registry_file_path = url::Url::parse(registry_file_url)
        .map_err(|e| anyhow::anyhow!("Invalid file URL: {}", e))?
        .to_file_path()
        .map_err(|_| anyhow::anyhow!("Failed to convert file URL to path"))?;

    if !registry_file_path.exists() {
        panic!(
            "Should not happen. The file does not exist in the local registry: {}",
            registry_file_path.display()
        );
    }

    let hash_cache = calc_file_hash(cache_file)?;
    let hash_registry = calc_file_hash(&registry_file_path)?;

    Ok(hash_cache == hash_registry)
}

pub async fn get_package(
    tman_config: Arc<tokio::sync::RwLock<TmanConfig>>,
    pkg_type: &PkgType,
    pkg_name: &str,
    pkg_version: &Version,
    url: &str,
    temp_path: &mut NamedTempFile,
    out: Arc<Box<dyn TmanOutput>>,
) -> Result<()> {
    // First, try to retrieve the same package file from the cache.
    let registry_file_path = url::Url::parse(url)
        .map_err(|e| anyhow::anyhow!("Invalid file URL: {}", e))?
        .to_file_path()
        .map_err(|_| anyhow::anyhow!("Failed to convert file URL to path"))?;

    let file_name = registry_file_path
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("downloaded file has invalid name"))?;

    if let Some(cached_file_path) = find_in_package_cache(
        pkg_type,
        pkg_name,
        pkg_version,
        &file_name.to_string_lossy(),
    )? {
        // We need to check whether the cached file and the target file have the
        // same content (i.e., the same hash).
        if let Ok(true) = is_same_file_by_hash(&cached_file_path, url) {
            // If the content is the same, directly copy the cached file to
            // `temp_path`.
            if is_verbose(tman_config.clone()).await {
                out.normal_line(&format!(
                    "{}  Found the package file ({}) in the package cache, using it directly.",
                    Emoji("🚀", ":-)"),
                    cached_file_path.to_string_lossy()
                ));
            }

            fs::copy(&cached_file_path, temp_path.path()).with_context(
                || {
                    format!(
                        "Failed to copy from cache {}",
                        cached_file_path.display()
                    )
                },
            )?;
            return Ok(());
        }
    }

    // Not found in the package cache, so proceed with the standard process to
    // retrieve it from the registry.

    let path_url_str = url::Url::parse(url)
        .map_err(|e| anyhow!("Invalid file URL: {}", e))?
        .to_file_path()
        .map_err(|_| anyhow!("Failed to convert file URL to path"))?
        .to_string_lossy()
        .into_owned();

    let path_url = PathBuf::from_str(&path_url_str)?;

    if !path_url.exists() {
        return Err(anyhow!(
            "The package directory does not exist: {}",
            path_url.display()
        ));
    }

    fs::copy(&path_url, temp_path.path())?;

    if tman_config.read().await.enable_package_cache {
        // Place the downloaded file into the cache.
        store_file_to_package_cache(
            pkg_type,
            pkg_name,
            pkg_version,
            &file_name.to_string_lossy(),
            &path_url,
        )?;
    }

    Ok(())
}

fn find_file_with_criteria(
    base_url: &Path,
    pkg_type: Option<PkgType>,
    name: Option<&String>,
    version_req: Option<&VersionReq>,
) -> Result<Vec<PkgRegistryInfo>> {
    let mut results = Vec::<PkgRegistryInfo>::new();

    // Determine which path to search based on pkg_type and name, and handle
    // search logic directly.
    match (pkg_type, name) {
        (Some(pkg_type), Some(name_str)) => {
            // Search specific pkg_type and name.
            let search_path = base_url.join(pkg_type.to_string());
            if search_path.exists() {
                let mut path_results =
                    search_versions(&search_path, name_str, version_req)?;
                results.append(&mut path_results);
            }
        }
        (Some(pkg_type), None) => {
            // Search all names under specific pkg_type.
            let search_path = base_url.join(pkg_type.to_string());
            if search_path.exists() {
                for entry in (std::fs::read_dir(&search_path)?).flatten() {
                    if entry.file_type()?.is_dir() {
                        let name_str =
                            entry.file_name().to_string_lossy().to_string();
                        let mut name_results = search_versions(
                            &search_path,
                            &name_str,
                            version_req,
                        )?;
                        results.append(&mut name_results);
                    }
                }
            }
        }
        (None, Some(name)) => {
            // Search all package types for this name.
            for entry in (std::fs::read_dir(base_url)?).flatten() {
                if entry.file_type()?.is_dir() {
                    let type_dir = entry.path();
                    let name_dir = type_dir.join(name);
                    if name_dir.exists() {
                        let mut type_results =
                            search_versions(&type_dir, name, version_req)?;
                        results.append(&mut type_results);
                    }
                }
            }
        }
        (None, None) => {
            // Search all package types and names.

            for type_entry in (std::fs::read_dir(base_url)?).flatten() {
                if type_entry.file_type()?.is_dir() {
                    let type_dir = type_entry.path();
                    for name_entry in (std::fs::read_dir(&type_dir)?).flatten()
                    {
                        if name_entry.file_type()?.is_dir() {
                            let mut name_results = search_versions(
                                &type_dir,
                                name_entry
                                    .file_name()
                                    .to_string_lossy()
                                    .as_ref(),
                                version_req,
                            )?;
                            results.append(&mut name_results);
                        }
                    }
                }
            }
        }
    }

    Ok(results)
}

// Helper function to search for versions.
fn search_versions(
    base_dir: &Path,
    name: &str,
    version_req: Option<&VersionReq>,
) -> Result<Vec<PkgRegistryInfo>> {
    let mut results = Vec::<PkgRegistryInfo>::new();
    let target_path = base_dir.join(name);

    // Traverse the folders of all versions within the specified package.
    for version_dir in WalkDir::new(target_path)
        .min_depth(1)
        .max_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let version_str = version_dir.file_name().to_str().unwrap_or_default();
        let version = match Version::parse(version_str) {
            Ok(v) => v,
            Err(_) => continue, // Skip invalid version directories.
        };

        // Check if the folder meets the version requirements.
        if version_req.is_none()
            || version_req
                .as_ref()
                .is_some_and(|req| req.matches(&version))
        {
            // Traverse the files within the folder of that version.
            for file in WalkDir::new(version_dir.path())
                .min_depth(1)
                .max_depth(1)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let path = file.path();

                // Look for .tpkg files.
                if path
                    .file_name()
                    .and_then(|f| f.to_str())
                    .is_some_and(|f| f.ends_with(TEN_PACKAGE_FILE_EXTENSION))
                {
                    // Get the file stem to look for the corresponding manifest
                    // file.
                    if let Some(file_stem) =
                        path.file_stem().and_then(|f| f.to_str())
                    {
                        // Look for the matching manifest file:
                        // {file_stem}_manifest.json
                        let manifest_file_name =
                            format!("{}_manifest.json", file_stem);
                        let manifest_path =
                            path.with_file_name(&manifest_file_name);

                        // Read the manifest file if it exists.
                        if manifest_path.exists() {
                            let manifest_content =
                                fs::read_to_string(&manifest_path)
                                    .with_context(|| {
                                        format!(
                                            "Failed to read manifest file: {}",
                                            manifest_path.display()
                                        )
                                    })?;

                            let manifest =
                                Manifest::from_str(&manifest_content)?;

                            // Generate the download URL from the file path.
                            let download_url = url::Url::from_file_path(path)
                                .map_err(|_| {
                                    anyhow!(
                                        "Failed to convert path to file URL"
                                    )
                                })?
                                .to_string();

                            // Convert manifest to PkgRegistryInfo.
                            let mut pkg_registry_info: PkgRegistryInfo =
                                get_pkg_registry_info_from_manifest(
                                    &download_url,
                                    &manifest,
                                )?;

                            pkg_registry_info.download_url = download_url;

                            results.push(pkg_registry_info);
                        }
                    }
                }
            }
        }
    }

    Ok(results)
}

#[allow(clippy::too_many_arguments)]
pub async fn get_package_list(
    _tman_config: Arc<tokio::sync::RwLock<TmanConfig>>,
    base_url: &str,
    pkg_type: Option<PkgType>,
    name: Option<String>,
    version_req: Option<VersionReq>,
    page_size: Option<u32>,
    page: Option<u32>,
    _out: &Arc<Box<dyn TmanOutput>>,
) -> Result<Vec<PkgRegistryInfo>> {
    let mut path_url = url::Url::parse(base_url)
        .map_err(|e| anyhow!("Invalid file URL: {}", e))?
        .to_file_path()
        .map_err(|_| anyhow!("Failed to convert file URL to path"))?
        .to_string_lossy()
        .into_owned();

    // Ensure the base URL ends with a '/'.
    path_url = if path_url.ends_with('/') {
        path_url.to_string()
    } else {
        format!("{}/", path_url)
    };

    let version_req_ref = version_req.as_ref();
    let name_ref = name.as_ref();

    // Get all matching packages
    let all_results = find_file_with_criteria(
        Path::new(&path_url),
        pkg_type,
        name_ref,
        version_req_ref,
    )?;

    // If page is specified, paginate the results.
    if let Some(page_num) = page {
        let page_size_value =
            page_size.unwrap_or(DEFAULT_REGISTRY_PAGE_SIZE) as usize;
        let start_idx = (page_num as usize - 1) * page_size_value;

        // Return empty result if start index is beyond the array length.
        if start_idx >= all_results.len() {
            return Ok(Vec::new());
        }

        let end_idx =
            std::cmp::min(start_idx + page_size_value, all_results.len());
        Ok(all_results[start_idx..end_idx].to_vec())
    } else {
        // If no page is specified, return all results.
        Ok(all_results)
    }
}

pub async fn delete_package(
    base_url: &str,
    pkg_type: PkgType,
    name: &String,
    version: &Version,
    hash: &String,
    _out: Arc<Box<dyn TmanOutput>>,
) -> Result<()> {
    let mut path_url = url::Url::parse(base_url)
        .map_err(|e| anyhow!("Invalid file URL: {}", e))?
        .to_file_path()
        .map_err(|_| anyhow!("Failed to convert file URL to path"))?
        .to_string_lossy()
        .into_owned();

    // Ensure the base URL ends with a '/'.
    path_url = if path_url.ends_with('/') {
        path_url.to_string()
    } else {
        format!("{}/", path_url)
    };

    // Construct the directory path.
    let dir_path = PathBuf::from(format!(
        "{}{}/{}/{}/",
        path_url, pkg_type, name, version
    ));

    if dir_path.exists() {
        // Iterate over the files in the directory.
        for entry in fs::read_dir(&dir_path).with_context(|| {
            format!("Failed to read directory '{}'", dir_path.display())
        })? {
            let entry = entry.with_context(|| {
                format!(
                    "Failed to read entry in directory '{}'",
                    dir_path.display()
                )
            })?;
            let file_path = entry.path();

            // Check if the file name matches the pattern.
            if let Some(file_name) =
                file_path.file_name().and_then(|name| name.to_str())
            {
                if let Some(file_stem) = Path::new(file_name)
                    .file_stem()
                    .and_then(|stem| stem.to_str())
                {
                    if file_stem.ends_with(&format!("_{}", hash)) {
                        // Delete the file.
                        fs::remove_file(&file_path).with_context(|| {
                            format!(
                                "Failed to delete file '{}'",
                                file_path.display()
                            )
                        })?;
                    }
                }
            }
        }
    }

    Ok(())
}
