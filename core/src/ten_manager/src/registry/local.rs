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

use anyhow::{anyhow, Context, Result};
use console::Emoji;
use flate2::read::GzDecoder;
use semver::{Version, VersionReq};
use sha2::{Digest, Sha256};
use tar::Archive as TarArchive;
use tempfile::NamedTempFile;
use ten_rust::pkg_info::constants::MANIFEST_JSON_FILENAME;
use walkdir::WalkDir;
use zip::ZipArchive;

use ten_rust::pkg_info::manifest::Manifest;
use ten_rust::pkg_info::pkg_type::PkgType;
use ten_rust::pkg_info::PkgInfo;

use super::found_result::PkgRegistryInfo;
use super::pkg_cache::{find_in_package_cache, store_file_to_package_cache};
use crate::config::TmanConfig;
use crate::constants::TEN_PACKAGE_FILE_EXTENSION;
use crate::file_type::{detect_file_type, FileType};

pub async fn upload_package(
    base_url: &str,
    package_file_path: &str,
    pkg_info: &PkgInfo,
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
        pkg_info.basic_info.type_and_name.pkg_type,
        pkg_info.basic_info.type_and_name.name,
        pkg_info.basic_info.version
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
    tman_config: &TmanConfig,
    pkg_type: &PkgType,
    pkg_name: &str,
    pkg_version: &Version,
    url: &str,
    temp_path: &mut NamedTempFile,
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
            eprintln!(
                "{}  Found the package file ({}) in the package cache, using it directly.",
                Emoji("🚀", ":-("),
                cached_file_path.to_string_lossy()
            );

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

    if tman_config.enable_package_cache {
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

trait ArchiveFindManifest {
    /// Attempt to locate and read the contents of `manifest.json` in the
    /// archive file.
    fn find_manifest(&mut self) -> Result<Option<String>>;
}

impl ArchiveFindManifest for ZipArchive<File> {
    fn find_manifest(&mut self) -> Result<Option<String>> {
        for i in 0..self.len() {
            let mut file_in_zip = self
                .by_index(i)
                .context("Failed to access file within zip archive")?;

            // Check if the file name is `manifest.json`.
            if file_in_zip.name() == MANIFEST_JSON_FILENAME {
                let mut manifest_content = String::new();
                file_in_zip
                    .read_to_string(&mut manifest_content)
                    .context("Failed to read manifest.json from zip")?;
                return Ok(Some(manifest_content));
            }
        }

        // Not found.
        Ok(None)
    }
}

impl ArchiveFindManifest for TarArchive<GzDecoder<File>> {
    fn find_manifest(&mut self) -> Result<Option<String>> {
        for entry_result in self.entries()? {
            let mut entry = entry_result?;
            let entry_path = entry.path()?;

            // Check if the file name is `manifest.json`.
            if let Some(file_name) =
                entry_path.file_name().and_then(|os| os.to_str())
            {
                if file_name == MANIFEST_JSON_FILENAME {
                    let mut manifest_content = String::new();
                    entry
                        .read_to_string(&mut manifest_content)
                        .context("Failed to read manifest.json from tar.gz")?;
                    return Ok(Some(manifest_content));
                }
            }
        }

        // Not found.
        Ok(None)
    }
}

fn find_file_with_criteria(
    base_url: &Path,
    pkg_type: PkgType,
    name: &String,
    version_req: &VersionReq,
) -> Result<Vec<PkgRegistryInfo>> {
    let target_path = base_url.join(pkg_type.to_string()).join(name);

    let mut results = Vec::<PkgRegistryInfo>::new();

    // Traverse the folders of all versions within the specified pkg.
    for version_dir in WalkDir::new(target_path)
        .min_depth(1)
        .max_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        // Check if the folder meets the version requirements.
        if version_req.matches(
            &Version::parse(
                version_dir.file_name().to_str().unwrap_or_default(),
            )
            .context("Invalid version format")?,
        ) {
            // Traverse the files within the folder of that version.
            for entry in WalkDir::new(version_dir.path())
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let path = entry.path();

                // Only process files with the extension equal to the defined
                // `TEN_PACKAGE_FILE_EXTENSION`.
                if path.extension().and_then(|s| s.to_str())
                    == Some(TEN_PACKAGE_FILE_EXTENSION)
                {
                    let file = File::open(path).with_context(|| {
                        format!("Failed to open {:?}", path)
                    })?;

                    let file_type = detect_file_type(path)?;

                    let maybe_manifest = match file_type {
                        FileType::Zip => {
                            let mut archive = ZipArchive::new(file)
                                .context("Failed to read zip archive")?;
                            archive.find_manifest()?
                        }
                        FileType::TarGz => {
                            let mut tar = TarArchive::new(GzDecoder::new(file));
                            tar.find_manifest()?
                        }
                    };

                    // If `find_manifest` finds the manifest content, parse it.
                    if let Some(manifest_content) = maybe_manifest {
                        let manifest = Manifest::from_str(&manifest_content)?;

                        // Generate the download URL from the file path.
                        let download_url = url::Url::from_file_path(path)
                            .map_err(|_| {
                                anyhow!("Failed to convert path to file URL")
                            })?
                            .to_string();

                        // Convert manifest to PkgRegistryInfo.
                        let mut pkg_registry_info: PkgRegistryInfo =
                            (&manifest).try_into()?;

                        pkg_registry_info.download_url = download_url;

                        results.push(pkg_registry_info);
                    }
                }
            }
        }
    }

    Ok(results)
}

pub async fn get_package_list(
    _tman_config: &TmanConfig,
    base_url: &str,
    pkg_type: PkgType,
    name: &String,
    version_req: &VersionReq,
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

    let result = find_file_with_criteria(
        Path::new(&path_url),
        pkg_type,
        name,
        version_req,
    )?;

    Ok(result)
}

pub async fn delete_package(
    base_url: &str,
    pkg_type: PkgType,
    name: &String,
    version: &Version,
    hash: &String,
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

#[cfg(test)]
mod tests {
    use semver::VersionReq;

    use super::*;

    #[test]
    fn test_semver_version_req() -> Result<()> {
        let mut rc =
            VersionReq::parse("3.0.0")?.matches(&Version::parse("3.0.0")?);
        assert!(rc);

        rc = VersionReq::parse("3.0.0")?.matches(&Version::parse("3.0.1")?);
        assert!(rc);

        rc = VersionReq::parse("3.0.0")?.matches(&Version::parse("3.1.0")?);
        assert!(rc);

        rc = VersionReq::parse("3.0.0")?
            .matches(&Version::parse("3.1.0-alpha")?);
        assert!(!rc);

        rc = VersionReq::parse("3.0.0")?
            .matches(&Version::parse("3.1.0+build3")?);
        assert!(rc);

        rc = VersionReq::parse("3.0.0")?
            .matches(&Version::parse("3.1.0-beta+build4")?);
        assert!(!rc);

        rc = VersionReq::parse("3.0.0")?.matches(&Version::parse("4.0.0")?);
        assert!(!rc);

        rc = VersionReq::parse("3.0.0")?.matches(&Version::parse("4.0.0")?);
        assert!(!rc);

        rc = VersionReq::parse(">=2.0.0")?.matches(&Version::parse("2.1.0+3")?);
        assert!(rc);

        rc = VersionReq::parse(">=2.0.0")?.matches(&Version::parse("2.0.0+3")?);
        assert!(rc);

        Ok(())
    }

    #[test]
    fn test_semver_version() -> Result<()> {
        let mut rc = Version::parse("3.0.1")? > Version::parse("3.0.0")?;
        assert!(rc);

        rc = Version::parse("3.0.0")? > Version::parse("3.0.0-alpha")?;
        assert!(rc);

        rc = Version::parse("3.0.0")? < Version::parse("3.0.1-alpha")?;
        assert!(rc);

        rc = Version::parse("3.0.0-alpha")? < Version::parse("3.0.0-beta")?;
        assert!(rc);

        rc = Version::parse("3.0.0")? == Version::parse("3.0.0")?;
        assert!(rc);

        Ok(())
    }
}
