//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use semver::Version;
use sha2::{Digest, Sha256};

use super::{
    manifest::dependency::ManifestDependency,
    manifest::support::ManifestSupport, pkg_type::PkgType, PkgInfo,
};

impl PkgInfo {
    pub fn gen_hash_hex(&self) -> String {
        let dependencies = match &self.manifest {
            Some(manifest) => match &manifest.dependencies {
                Some(deps) => deps,
                None => &Vec::new(),
            },
            None => &Vec::new(),
        };

        gen_hash_hex(
            &self.basic_info.type_and_name.pkg_type,
            &self.basic_info.type_and_name.name,
            &self.basic_info.version,
            dependencies,
            &self.basic_info.supports,
        )
    }
}

fn gen_hash_hex(
    pkg_type: &PkgType,
    name: &String,
    version: &Version,
    dependencies: &Vec<ManifestDependency>,
    supports: &Vec<ManifestSupport>,
) -> String {
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
        let dep_string = match dep {
            ManifestDependency::RegistryDependency {
                pkg_type,
                name,
                version_req,
            } => {
                format!("{}:{}@{}", pkg_type, name, version_req)
            }
            ManifestDependency::LocalDependency { path, base_dir } => {
                format!("local:{}@{}", path, base_dir)
            }
        };
        hasher.update(dep_string);
    }

    // Hash supports.
    for support in supports {
        let support_string = format!("{}", support);
        hasher.update(support_string);
    }

    let hash_result = hasher.finalize();
    let hash_hex = format!("{:x}", hash_result);

    hash_hex
}
