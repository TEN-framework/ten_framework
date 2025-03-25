//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use anyhow::Result;
use serde::{Deserialize, Serialize};

use ten_rust::pkg_info::manifest::dependency::ManifestDependency;
use ten_rust::pkg_info::manifest::Manifest;
use ten_rust::pkg_info::pkg_basic_info::PkgBasicInfo;
use ten_rust::pkg_info::PkgInfo;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PkgRegistryInfo {
    #[serde(flatten)]
    pub basic_info: PkgBasicInfo,

    #[serde(with = "dependencies_conversion")]
    pub dependencies: Vec<ManifestDependency>,

    pub hash: String,

    #[serde(rename = "downloadUrl")]
    pub download_url: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "contentFormat")]
    pub content_format: Option<String>,
}

mod dependencies_conversion {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use ten_rust::pkg_info::manifest::dependency::ManifestDependency;

    pub fn serialize<S>(
        deps: &[ManifestDependency],
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let manifest_deps: Vec<ManifestDependency> = deps.to_vec();
        manifest_deps.serialize(serializer)
    }

    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<Vec<ManifestDependency>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let manifest_deps: Vec<ManifestDependency> =
            Deserialize::deserialize(deserializer)?;
        Ok(manifest_deps)
    }
}

impl TryFrom<&Manifest> for PkgRegistryInfo {
    type Error = anyhow::Error;

    fn try_from(manifest: &Manifest) -> Result<Self> {
        let pkg_info = PkgInfo::from_metadata(manifest, &None)?;
        Ok((&pkg_info).into())
    }
}

impl From<&PkgInfo> for PkgRegistryInfo {
    fn from(pkg_info: &PkgInfo) -> Self {
        let dependencies = match &pkg_info.manifest {
            Some(manifest) => match &manifest.dependencies {
                Some(deps) => deps.clone(),
                None => vec![],
            },
            None => vec![],
        };

        PkgRegistryInfo {
            basic_info: PkgBasicInfo::from(pkg_info),
            dependencies,
            hash: pkg_info.hash.clone(),
            download_url: String::new(),
            content_format: None,
        }
    }
}

impl From<&PkgRegistryInfo> for PkgInfo {
    fn from(pkg_registry_info: &PkgRegistryInfo) -> Self {
        let mut pkg_info = PkgInfo {
            basic_info: pkg_registry_info.basic_info.clone(),
            compatible_score: -1,
            is_installed: false,
            url: pkg_registry_info.download_url.clone(),
            hash: pkg_registry_info.hash.clone(),
            manifest: None,
            property: None,
            schema_store: None,
            is_local_dependency: false,
            local_dependency_path: None,
            local_dependency_base_dir: None,
        };

        // Create a manifest with the dependencies
        let manifest = Manifest {
            type_and_name: pkg_registry_info.basic_info.type_and_name.clone(),
            version: pkg_registry_info.basic_info.version.clone(),
            dependencies: Some(pkg_registry_info.dependencies.clone()),
            supports: None,
            api: None,
            package: None,
            scripts: None,
        };

        pkg_info.manifest = Some(manifest);

        pkg_info
    }
}
