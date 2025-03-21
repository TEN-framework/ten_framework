//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::str::FromStr;

use anyhow::{anyhow, Result};
use semver::VersionReq;
use serde::{Deserialize, Serialize};

use super::{
    constants::MANIFEST_JSON_FILENAME, pkg_type::PkgType,
    pkg_type_and_name::PkgTypeAndName,
};
use crate::pkg_info::manifest::{dependency::ManifestDependency, Manifest};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PkgDependency {
    #[serde(flatten)]
    pub type_and_name: PkgTypeAndName,

    // The version requirement of this dependency, ex: the `version`
    // field declared in the `dependencies` section in the manifest.json, or
    // the `pkg@version` parameter in the command line.
    //
    // The `version_req_str` is the original value in string form, while
    // `version_req` is the result after being converted to `VersionReq`.
    pub version_req: VersionReq,

    // If the `path` field is not `None`, it indicates that this is a local
    // dependency.
    pub path: Option<String>,

    // Refer to `base_dir` of ManifestDependency.
    pub base_dir: Option<String>,
}

impl PkgDependency {
    pub fn is_local(&self) -> bool {
        self.path.is_some()
    }
}

impl From<&PkgDependency> for PkgTypeAndName {
    fn from(dependency: &PkgDependency) -> Self {
        dependency.type_and_name.clone()
    }
}

impl TryFrom<&ManifestDependency> for PkgDependency {
    type Error = anyhow::Error;

    fn try_from(dep: &ManifestDependency) -> Result<Self> {
        match dep {
            ManifestDependency::RegistryDependency {
                pkg_type,
                name,
                version,
            } => {
                // Currently, tman uses the Rust semver crate, while the cloud
                // store uses the npm semver package. The semver requirement
                // specifications of these two packages are not completely
                // identical. For example:
                //
                // - The Rust semver crate uses "," to separate different
                //   ranges, whereas the npm semver package uses a space (" ")
                //   to separate different requirement ranges.
                // - The npm semver package uses "||" to unify different ranges,
                //   but the Rust semver crate does not support this feature.
                //
                // Since TEN is a cross-language system, it needs to define its
                // own semver requirement specification. This specification
                // could follow either the Rust or npm format or other spec, but
                // in either case, tman or the cloud store would need to make
                // adaptations.
                //
                // Therefore, the current approach is to simplify the
                // specification to only support a single-range semver
                // requirement, which is the common subset of both the npm
                // semver package and the Rust semver crate.
                if version.contains("||")
                    || version.chars().any(|c| c.is_whitespace())
                    || version.contains(",")
                {
                    return Err(anyhow!(
                        "Invalid version requirement '{}' in manifest: contains forbidden characters (||, whitespace, or ,)",
                        version
                    ));
                }

                Ok(PkgDependency {
                    type_and_name: PkgTypeAndName {
                        pkg_type: PkgType::from_str(pkg_type)?,
                        name: name.clone(),
                    },
                    version_req: VersionReq::parse(version)?,
                    path: None,
                    base_dir: None,
                })
            }

            ManifestDependency::LocalDependency { path, base_dir } => {
                // Check if there is a manifest.json file under the path.
                let dep_folder_path = std::path::Path::new(base_dir).join(path);

                if !dep_folder_path.exists() {
                    return Err(anyhow!(
                        "Local dependency path '{}' does not exist",
                        path
                    ));
                }

                let dep_manifest_path =
                    dep_folder_path.join(MANIFEST_JSON_FILENAME);

                if !dep_manifest_path.exists() {
                    return Err(anyhow!(
                        "Local dependency path '{}' does not contain \
                        manifest.json",
                        path
                    ));
                }

                // Parse the `manifest.json` file to retrieve the `type`,
                // `name`, and `version`.
                let local_manifest =
                    crate::pkg_info::manifest::parse_manifest_from_file(
                        &dep_manifest_path,
                    )?;

                // Currently, tman uses the Rust semver crate, while the cloud
                // store uses the npm semver package. The semver requirement
                // specifications of these two packages are not completely
                // identical. For example:
                //
                // - The Rust semver crate uses "," to separate different
                //   ranges, whereas the npm semver package uses a space (" ")
                //   to separate different requirement ranges.
                // - The npm semver package uses "||" to unify different ranges,
                //   but the Rust semver crate does not support this feature.
                //
                // Since TEN is a cross-language system, it needs to define its
                // own semver requirement specification. This specification
                // could follow either the Rust or npm format or other spec, but
                // in either case, tman or the cloud store would need to make
                // adaptations.
                //
                // Therefore, the current approach is to simplify the
                // specification to only support a single-range semver
                // requirement, which is the common subset of both the npm
                // semver package and the Rust semver crate.
                if local_manifest.version.contains("||")
                    || local_manifest.version.chars().any(|c| c.is_whitespace())
                    || local_manifest.version.contains(",")
                {
                    return Err(anyhow!(
                        "Invalid version requirement '{}' in local manifest: contains forbidden characters (||, whitespace, or ,)",
                        local_manifest.version
                    ));
                }

                Ok(PkgDependency {
                    type_and_name: local_manifest.type_and_name.clone(),
                    version_req: semver::VersionReq::parse(
                        &local_manifest.version,
                    )?,
                    path: Some(path.clone()),
                    base_dir: Some(base_dir.clone()),
                })
            }
        }
    }
}

pub fn get_pkg_dependencies_from_manifest(
    manifest: &Manifest,
) -> Result<Vec<PkgDependency>> {
    match &manifest.dependencies {
        Some(manifest_dependencies) => {
            Ok(get_pkg_dependencies_from_manifest_dependencies(
                manifest_dependencies,
            )?)
        }
        None => Ok(vec![]),
    }
}

fn get_pkg_dependencies_from_manifest_dependencies(
    manifest_dependencies: &[ManifestDependency],
) -> Result<Vec<PkgDependency>> {
    manifest_dependencies
        .iter()
        .map(PkgDependency::try_from)
        .collect()
}

pub fn get_manifest_dependencies_from_pkg(
    pkg_dependencies: Vec<PkgDependency>,
) -> Vec<ManifestDependency> {
    pkg_dependencies.into_iter().map(|v| (&v).into()).collect()
}
