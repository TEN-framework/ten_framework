//
// Copyright © 2024 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to https://github.com/TEN-framework/ten_framework/LICENSE for more
// information.
//
use serde::{Deserialize, Serialize};

use crate::pkg_info::dependencies::PkgDependency;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ManifestDependency {
    #[serde(rename = "type")]
    pub pkg_type: String,
    pub name: String,
    pub version: String,
}

impl From<PkgDependency> for ManifestDependency {
    fn from(pkg_dependency: PkgDependency) -> Self {
        ManifestDependency {
            pkg_type: pkg_dependency.pkg_identity.pkg_type.to_string(),
            name: pkg_dependency.pkg_identity.name,
            version: pkg_dependency.version_req.to_string(),
        }
    }
}
