//
// Copyright © 2024 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use anyhow::Result;
use semver::Version;
use sha2::{Digest, Sha256};

use super::{
    dependencies::PkgDependency, pkg_type::PkgType, supports::PkgSupport,
    PkgInfo,
};

impl PkgInfo {
    pub fn gen_hash_hex(&self) -> Result<String> {
        gen_hash_hex(
            &self.pkg_type,
            &self.name,
            &self.version,
            &self.dependencies,
            &self.supports,
        )
    }
}

pub fn gen_hash_hex(
    pkg_type: &PkgType,
    name: &String,
    version: &Version,
    dependencies: &Vec<PkgDependency>,
    supports: &Vec<PkgSupport>,
) -> Result<String> {
    let mut hasher = Sha256::new();

    // Hash type.
    let type_string = format!("{}", pkg_type);
    hasher.update(type_string);

    // Hash name.
    hasher.update(name);

    // Hash version.
    let version_string = version.to_string();
    hasher.update(version_string);

    // Hash dependencies.
    for dep in dependencies {
        let dep_string =
            format!("{}:{}@{}", dep.pkg_type, dep.name, dep.version_req);
        hasher.update(dep_string);
    }

    // Hash supports.
    for support in supports {
        let support_string = format!("{}", support);
        hasher.update(support_string);
    }

    let hash_result = hasher.finalize();
    let hash_hex = format!("{:x}", hash_result);

    Ok(hash_hex)
}
