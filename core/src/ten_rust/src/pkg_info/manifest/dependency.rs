//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use semver::VersionReq;
use serde::{Deserialize, Serialize};

use crate::pkg_info::{pkg_type::PkgType, PkgInfo};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum ManifestDependency {
    RegistryDependency {
        #[serde(rename = "type")]
        pkg_type: PkgType,

        name: String,

        #[serde(rename = "version")]
        version_req: VersionReq,
    },

    LocalDependency {
        path: String,

        // Used to record the folder path where the `manifest.json` containing
        // this dependency is located. It is primarily used to parse the `path`
        // field when it contains a relative path.
        #[serde(skip)]
        base_dir: String,
    },
}

impl From<&PkgInfo> for ManifestDependency {
    fn from(pkg_info: &PkgInfo) -> Self {
        if pkg_info.is_local_dependency {
            ManifestDependency::LocalDependency {
                path: {
                    assert!(
                        pkg_info.local_dependency_path.is_some(),
                        "Should not happen."
                    );

                    pkg_info.local_dependency_path.as_ref().unwrap().clone()
                },
                base_dir: {
                    assert!(
                        pkg_info.local_dependency_base_dir.is_some(),
                        "Should not happen."
                    );

                    pkg_info.local_dependency_base_dir.as_ref().unwrap().clone()
                },
            }
        } else {
            ManifestDependency::RegistryDependency {
                pkg_type: pkg_info.basic_info.type_and_name.pkg_type,
                name: pkg_info.basic_info.type_and_name.name.clone(),

                // Use the package's version as the declared dependency version.
                version_req: VersionReq::parse(
                    &pkg_info.basic_info.version.to_string(),
                )
                .unwrap(),
            }
        }
    }
}
