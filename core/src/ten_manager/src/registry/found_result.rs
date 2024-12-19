//
// Copyright © 2024 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::path::PathBuf;

use anyhow::Result;
use semver::Version;
use serde::{Deserialize, Serialize};

use ten_rust::pkg_info::manifest::{
    dependency::ManifestDependency, support::ManifestSupport,
};
use ten_rust::pkg_info::pkg_basic_info::PkgBasicInfo;
use ten_rust::pkg_info::pkg_type_and_name::PkgTypeAndName;
use ten_rust::pkg_info::{
    dependencies::get_pkg_dependencies_from_manifest_dependencies,
    hash::gen_hash_hex, pkg_type::PkgType,
    supports::get_pkg_supports_from_manifest_supports,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct RegistryPackageData {
    #[serde(rename = "type")]
    pub pkg_type: PkgType,

    pub name: String,
    pub version: Version,
    pub dependencies: Vec<ManifestDependency>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports: Option<Vec<ManifestSupport>>,

    pub hash: String,
}

impl TryFrom<&RegistryPackageData> for PkgTypeAndName {
    type Error = anyhow::Error;

    fn try_from(package_data: &RegistryPackageData) -> Result<Self> {
        Ok(PkgTypeAndName {
            pkg_type: package_data.pkg_type,
            name: package_data.name.clone(),
        })
    }
}

impl TryFrom<&RegistryPackageData> for PkgBasicInfo {
    type Error = anyhow::Error;

    fn try_from(package_data: &RegistryPackageData) -> Result<Self> {
        Ok(PkgBasicInfo {
            type_and_name: PkgTypeAndName::try_from(package_data)?,
            version: package_data.version.clone(),
            supports: get_pkg_supports_from_manifest_supports(
                &package_data.supports,
            )?,
        })
    }
}

impl RegistryPackageData {
    pub fn gen_hash_hex(&self) -> Result<String> {
        let dependencies = get_pkg_dependencies_from_manifest_dependencies(
            &self.dependencies,
        )?;

        let supports = get_pkg_supports_from_manifest_supports(&self.supports)?;

        gen_hash_hex(
            &self.pkg_type,
            &self.name,
            &self.version,
            &dependencies,
            &supports,
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FoundResult {
    pub url: PathBuf,
    pub package_data: RegistryPackageData,
}
