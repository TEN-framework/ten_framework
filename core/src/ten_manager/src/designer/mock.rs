//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::collections::HashMap;
use std::str::FromStr;

use anyhow::Result;

use ten_rust::pkg_info::PkgInfo;
use ten_rust::pkg_info::{manifest::Manifest, property::Property};

pub fn inject_all_pkgs_for_mock(
    base_dir: &str,
    pkgs_cache: &mut HashMap<String, Vec<PkgInfo>>,
    all_pkgs_json: Vec<(String, String)>,
) -> Result<()> {
    if pkgs_cache.contains_key(base_dir) {
        return Err(anyhow::anyhow!("The all_pkgs field is already set"));
    }

    let mut all_pkgs: Vec<PkgInfo> = Vec::new();
    for metadata_json in all_pkgs_json {
        let manifest = Manifest::from_str(&metadata_json.0)?;
        let property = Property::from_str(&metadata_json.1)?;

        let pkg_info = PkgInfo::from_metadata(&manifest, &Some(property))?;
        all_pkgs.push(pkg_info);
    }

    pkgs_cache.insert(base_dir.to_string(), all_pkgs);

    Ok(())
}
