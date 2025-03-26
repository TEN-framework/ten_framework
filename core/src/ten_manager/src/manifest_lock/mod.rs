//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::collections::HashMap;
use std::path::Path;
use std::{fs, str::FromStr};

use anyhow::Result;
use console::Emoji;
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};

use ten_rust::json_schema::validate_manifest_lock_json_string;
use ten_rust::pkg_info::constants::{
    MANIFEST_JSON_FILENAME, MANIFEST_LOCK_JSON_FILENAME,
};
use ten_rust::pkg_info::manifest::dependency::ManifestDependency;
use ten_rust::pkg_info::manifest::support::ManifestSupport;
use ten_rust::pkg_info::manifest::Manifest;
use ten_rust::pkg_info::pkg_basic_info::PkgBasicInfo;
use ten_rust::pkg_info::pkg_type::PkgType;
use ten_rust::pkg_info::pkg_type_and_name::PkgTypeAndName;
use ten_rust::pkg_info::PkgInfo;

// Helper function to check if an Option<Vec> is None or an empty Vec.
fn is_none_or_empty<T>(option: &Option<Vec<T>>) -> bool {
    match option {
        None => true,
        Some(vec) => vec.is_empty(),
    }
}

// The `manifest-lock.json` structure.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ManifestLock {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<u32>,

    #[serde(skip_serializing_if = "is_none_or_empty")]
    pub packages: Option<Vec<ManifestLockItem>>,
}

type LockedPkgsInfo<'a> = &'a Vec<&'a PkgInfo>;

impl From<LockedPkgsInfo<'_>> for ManifestLock {
    // Convert a complete `Resolve` to a ManifestLock which can be serialized to
    // a `manifest-lock.json` file.
    fn from(resolve: LockedPkgsInfo) -> Self {
        ManifestLock {
            version: Some(1), // Not used for now.
            packages: Some(
                resolve
                    .iter()
                    .map(|pkg_info| ManifestLockItem::from(pkg_info.to_owned()))
                    .collect(),
            ),
        }
    }
}

impl FromStr for ManifestLock {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        validate_manifest_lock_json_string(s)?;
        let manifest_lock: ManifestLock = serde_json::from_str(s)?;

        Ok(manifest_lock)
    }
}

impl ManifestLock {
    pub fn get_pkgs(&self) -> HashMap<PkgTypeAndName, PkgInfo> {
        self.packages
            .as_ref()
            .map(|pkgs| {
                pkgs.iter()
                    .map(|pkg| {
                        let pkg_info: PkgInfo = pkg.into();
                        ((&pkg_info).into(), pkg_info)
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn print_changes(&self, old_resolve: &ManifestLock) {
        let old_pkgs = old_resolve.get_pkgs();
        let new_pkgs = self.get_pkgs();

        let mut added_pkgs = vec![];
        let mut removed_pkgs = vec![];
        let mut updated_pkgs = vec![];

        for (idt, old_pkg) in old_pkgs.iter() {
            let contains = new_pkgs.contains_key(idt);
            if !contains {
                removed_pkgs.push(old_pkg);
            } else {
                let new_pkg = new_pkgs.get(idt).unwrap();
                if let (Some(old_manifest), Some(new_manifest)) =
                    (&old_pkg.manifest, &new_pkg.manifest)
                {
                    if old_manifest.version != new_manifest.version {
                        updated_pkgs.push((old_pkg, new_pkg));
                    }
                }
            }
        }

        for (idt, new_pkg) in new_pkgs.iter() {
            let contains = old_pkgs.contains_key(idt);
            if !contains {
                added_pkgs.push(new_pkg);
            }
        }

        if !added_pkgs.is_empty() {
            for pkg in added_pkgs.iter() {
                println!(
                    "{}  Adding package {} v{}",
                    Emoji("➕", ""),
                    pkg.manifest.as_ref().map_or("unknown".to_string(), |m| m
                        .type_and_name
                        .name
                        .clone()),
                    pkg.manifest.as_ref().map_or("unknown".to_string(), |m| m
                        .version
                        .to_string())
                );
            }
        }

        if !removed_pkgs.is_empty() {
            for pkg in removed_pkgs.iter() {
                println!(
                    "{}  Removing package {} v{}",
                    Emoji("🗑️", ""),
                    pkg.manifest.as_ref().map_or("unknown".to_string(), |m| m
                        .type_and_name
                        .name
                        .clone()),
                    pkg.manifest.as_ref().map_or("unknown".to_string(), |m| m
                        .version
                        .to_string())
                );
            }
        }

        if !updated_pkgs.is_empty() {
            for (old_pkg, new_pkg) in updated_pkgs.iter() {
                println!(
                    "{}  Updating package {} v{} to v{}",
                    Emoji("🔄", ""),
                    old_pkg.manifest.as_ref().map_or(
                        "unknown".to_string(),
                        |m| m.type_and_name.name.clone()
                    ),
                    old_pkg
                        .manifest
                        .as_ref()
                        .map_or("unknown".to_string(), |m| m
                            .version
                            .to_string()),
                    new_pkg
                        .manifest
                        .as_ref()
                        .map_or("unknown".to_string(), |m| m
                            .version
                            .to_string())
                );
            }
        }
    }
}

fn are_equal_lockfiles(lock_file_path: &Path, resolve_str: &str) -> bool {
    // Read the contents of the lock file.
    let lock_file_str = crate::fs::read_file_to_string(lock_file_path)
        .unwrap_or_else(|_| "".to_string());

    // Compare the lock file contents with the new resolve string.
    lock_file_str.lines().eq(resolve_str.lines())
}

// Serialize the `ManifestLock` to a JSON string and write it to the lock file.
pub fn write_pkg_lockfile<P: AsRef<Path>>(
    manifest_lock: &ManifestLock,
    app_path: P,
) -> Result<bool> {
    // Serialize the `ManifestLock` to a JSON string.
    let encodable_resolve_str = serde_json::to_string_pretty(manifest_lock)?;

    let lock_file_path = app_path.as_ref().join(MANIFEST_LOCK_JSON_FILENAME);

    // If the lock file contents haven't changed, we don't need to rewrite it.
    if are_equal_lockfiles(lock_file_path.as_ref(), &encodable_resolve_str) {
        return Ok(false);
    }

    // TODO(xilin): Maybe RWlock is needed.
    fs::write(lock_file_path, encodable_resolve_str)?;
    Ok(true)
}

fn parse_manifest_lock_from_file<P: AsRef<Path>>(
    manifest_lock_file_path: P,
) -> Result<ManifestLock> {
    // Read the contents of the manifest-lock.json file.
    let content = crate::fs::read_file_to_string(manifest_lock_file_path)?;

    ManifestLock::from_str(&content)
}

pub fn parse_manifest_lock_in_folder(
    folder_path: &Path,
) -> Result<ManifestLock> {
    // Path to the manifest-lock.json file.
    let manifest_lock_file_path = folder_path.join(MANIFEST_LOCK_JSON_FILENAME);

    // Read and parse the manifest-lock.json file.
    let manifest_lock =
        parse_manifest_lock_from_file(manifest_lock_file_path.clone())?;

    Ok(manifest_lock)
}

// The `dependencies` field structure in `manifest-lock.json`.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ManifestLockItem {
    #[serde(rename = "type")]
    pub pkg_type: String,

    pub name: String,
    pub version: Version,
    pub hash: String,

    #[serde(skip_serializing_if = "is_none_or_empty")]
    pub dependencies: Option<Vec<ManifestLockItemDependencyItem>>,

    #[serde(skip_serializing_if = "is_none_or_empty")]
    pub supports: Option<Vec<ManifestSupport>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

impl TryFrom<&ManifestLockItem> for PkgTypeAndName {
    type Error = anyhow::Error;

    fn try_from(manifest: &ManifestLockItem) -> Result<Self> {
        Ok(PkgTypeAndName {
            pkg_type: PkgType::from_str(&manifest.pkg_type).unwrap(),
            name: manifest.name.clone(),
        })
    }
}

impl TryFrom<&ManifestLockItem> for PkgBasicInfo {
    type Error = anyhow::Error;

    fn try_from(manifest: &ManifestLockItem) -> Result<Self> {
        Ok(PkgBasicInfo {
            type_and_name: PkgTypeAndName::try_from(manifest)?,
            version: manifest.version.clone(),
            // If manifest.supports is None, then supports is an empty vector.
            supports: manifest.supports.clone().unwrap_or_default(),
        })
    }
}

fn get_encodable_deps_from_pkg_deps(
    manifest_deps: Vec<ManifestDependency>,
) -> Vec<ManifestLockItemDependencyItem> {
    manifest_deps
        .into_iter()
        .map(|dep| match dep {
            ManifestDependency::RegistryDependency {
                pkg_type, name, ..
            } => ManifestLockItemDependencyItem {
                pkg_type: pkg_type.to_string(),
                name,
            },
            ManifestDependency::LocalDependency { path, base_dir } => {
                // For local dependencies, we need to extract info from the
                // manifest.
                let abs_path = std::path::Path::new(&base_dir).join(&path);
                let dep_manifest_path = abs_path.join(MANIFEST_JSON_FILENAME);

                let manifest_result =
                    ten_rust::pkg_info::manifest::parse_manifest_from_file(
                        &dep_manifest_path,
                    );

                match manifest_result {
                    Ok(local_manifest) => ManifestLockItemDependencyItem {
                        pkg_type: local_manifest
                            .type_and_name
                            .pkg_type
                            .to_string(),
                        name: local_manifest.type_and_name.name,
                    },
                    Err(_) => {
                        // If we can't parse the manifest, use a placeholder.
                        ManifestLockItemDependencyItem {
                            pkg_type: "unknown".to_string(),
                            name: path,
                        }
                    }
                }
            }
        })
        .collect()
}

impl<'a> From<&'a PkgInfo> for ManifestLockItem {
    fn from(pkg_info: &'a PkgInfo) -> Self {
        let dependencies = match &pkg_info.manifest {
            Some(manifest) => manifest
                .dependencies
                .as_ref()
                .map(|deps| get_encodable_deps_from_pkg_deps(deps.clone())),
            None => None,
        };

        let (pkg_type, name, version, supports) =
            if let Some(manifest) = &pkg_info.manifest {
                (
                    manifest.type_and_name.pkg_type.to_string(),
                    manifest.type_and_name.name.clone(),
                    manifest.version.clone(),
                    manifest.supports.clone().unwrap_or_default(),
                )
            } else {
                (
                    "unknown".to_string(),
                    "unknown".to_string(),
                    Version::new(0, 0, 0),
                    Vec::new(),
                )
            };

        Self {
            pkg_type,
            name,
            version,
            hash: pkg_info.hash.to_string(),
            dependencies,
            supports: if supports.is_empty() {
                None
            } else {
                Some(supports)
            },
            path: pkg_info.local_dependency_path.clone(),
        }
    }
}

impl<'a> From<&'a ManifestLockItem> for PkgInfo {
    fn from(locked_item: &'a ManifestLockItem) -> Self {
        use ten_rust::pkg_info::manifest::dependency::ManifestDependency;
        use ten_rust::pkg_info::pkg_type_and_name::PkgTypeAndName;

        let dependencies_option =
            locked_item.clone().dependencies.map(|deps| {
                deps.into_iter()
                    .map(|dep| {
                        let pkg_type = match PkgType::from_str(&dep.pkg_type) {
                            Ok(pkg_type) => pkg_type,
                            Err(_) => PkgType::Extension,
                        };
                        ManifestDependency::RegistryDependency {
                            pkg_type,
                            name: dep.name,
                            // Default version requirement.
                            version_req: VersionReq::parse("*").unwrap(),
                        }
                    })
                    .collect()
            });

        let type_and_name = PkgTypeAndName {
            pkg_type: PkgType::from_str(&locked_item.pkg_type)
                .unwrap_or(PkgType::Extension),
            name: locked_item.name.clone(),
        };

        // Create a manifest with the dependencies.
        let manifest = Manifest {
            type_and_name: type_and_name.clone(),
            version: locked_item.version.clone(),
            dependencies: dependencies_option.clone(),
            supports: locked_item.supports.clone(),
            api: None,
            package: None,
            scripts: None,
            all_fields: {
                let mut map = serde_json::Map::new();

                // Add type and name.
                map.insert(
                    "type".to_string(),
                    serde_json::Value::String(
                        type_and_name.pkg_type.to_string(),
                    ),
                );
                map.insert(
                    "name".to_string(),
                    serde_json::Value::String(type_and_name.name.clone()),
                );

                // Add version.
                map.insert(
                    "version".to_string(),
                    serde_json::Value::String(locked_item.version.to_string()),
                );

                // Add dependencies if present.
                if let Some(deps) = &dependencies_option {
                    let deps_json = serde_json::to_value(deps)
                        .unwrap_or(serde_json::Value::Array(vec![]));
                    map.insert("dependencies".to_string(), deps_json);
                }

                // Add supports if present.
                if let Some(supports) = &locked_item.supports {
                    let supports_json = serde_json::to_value(supports)
                        .unwrap_or(serde_json::Value::Array(vec![]));
                    map.insert("supports".to_string(), supports_json);
                }

                map
            },
        };

        PkgInfo {
            compatible_score: 0, // TODO(xilin): default value.
            is_installed: false,
            url: "".to_string(), // TODO(xilin): default value.
            hash: locked_item.hash.clone(),
            manifest: Some(manifest),
            property: None,
            schema_store: None,
            is_local_dependency: locked_item.path.is_some(),
            local_dependency_path: locked_item.path.clone(),
            local_dependency_base_dir: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ManifestLockItemDependencyItem {
    #[serde(rename = "type")]
    pub pkg_type: String,

    pub name: String,
}
