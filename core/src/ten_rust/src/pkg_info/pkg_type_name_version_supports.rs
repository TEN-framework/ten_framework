//
// Copyright © 2024 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::hash::{Hash, Hasher};

use semver::Version;

use super::{pkg_type::PkgType, supports::PkgSupport, PkgInfo};

#[derive(Clone, Debug, PartialOrd, Ord)]
pub struct PkgTypeNameVersionSupports {
    pub pkg_type: PkgType,
    pub name: String,
    pub version: Version,

    // Since the declaration 'does not support all environments' has no
    // practical meaning, not specifying the 'supports' field or specifying an
    // empty 'supports' field both represent support for all environments.
    // Therefore, the 'supports' field here is not an option. An empty
    // 'supports' field represents support for all combinations of
    // environments.
    pub supports: Vec<PkgSupport>,
}

impl Hash for PkgTypeNameVersionSupports {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.pkg_type.hash(state);
        self.name.hash(state);
    }
}

impl PartialEq for PkgTypeNameVersionSupports {
    fn eq(&self, other: &Self) -> bool {
        self.pkg_type == other.pkg_type && self.name == other.name
    }
}

impl Eq for PkgTypeNameVersionSupports {}

impl From<&PkgInfo> for PkgTypeNameVersionSupports {
    fn from(pkg_info: &PkgInfo) -> Self {
        PkgTypeNameVersionSupports {
            pkg_type: pkg_info.pkg_type,
            name: pkg_info.name.clone(),
            version: pkg_info.version.clone(),
            supports: pkg_info.supports.clone(),
        }
    }
}
