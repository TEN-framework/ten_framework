//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::collections::HashMap;
use std::str::FromStr;

use anyhow::Result;

use ten_rust::base_dir_pkg_info::PkgsInfoInApp;
use ten_rust::graph::graph_info::GraphInfo;
use ten_rust::pkg_info::manifest::Manifest;
use ten_rust::pkg_info::pkg_type::PkgType;
use ten_rust::pkg_info::property::parse_property_from_str;
use ten_rust::pkg_info::PkgInfo;

pub fn inject_all_pkgs_for_mock(
    base_dir: &str,
    pkgs_cache: &mut HashMap<String, PkgsInfoInApp>,
    graphs_cache: &mut HashMap<String, GraphInfo>,
    all_pkgs_json: Vec<(String, String)>,
) -> Result<()> {
    if pkgs_cache.contains_key(base_dir) {
        return Err(anyhow::anyhow!("The all_pkgs field is already set"));
    }

    let mut app_pkg_info = None;
    let mut extension_pkg_info = Vec::new();
    let mut protocol_pkg_info = Vec::new();
    let mut addon_loader_pkg_info = Vec::new();
    let mut system_pkg_info = Vec::new();

    for metadata_json in all_pkgs_json {
        let manifest = Manifest::from_str(&metadata_json.0)?;
        let property = parse_property_from_str(&metadata_json.1, graphs_cache)?;

        let pkg_info = PkgInfo::from_metadata(&manifest, &Some(property))?;

        // Sort package by type.
        match manifest.type_and_name.pkg_type {
            PkgType::App => {
                app_pkg_info = Some(pkg_info);
            }
            PkgType::Extension => {
                extension_pkg_info.push(pkg_info);
            }
            PkgType::Protocol => {
                protocol_pkg_info.push(pkg_info);
            }
            PkgType::AddonLoader => {
                addon_loader_pkg_info.push(pkg_info);
            }
            PkgType::System => {
                system_pkg_info.push(pkg_info);
            }
            PkgType::Invalid => {}
        }
    }

    let base_dir_pkg_info = PkgsInfoInApp {
        app_pkg_info,
        extension_pkg_info: if extension_pkg_info.is_empty() {
            None
        } else {
            Some(extension_pkg_info)
        },
        protocol_pkg_info: if protocol_pkg_info.is_empty() {
            None
        } else {
            Some(protocol_pkg_info)
        },
        addon_loader_pkg_info: if addon_loader_pkg_info.is_empty() {
            None
        } else {
            Some(addon_loader_pkg_info)
        },
        system_pkg_info: if system_pkg_info.is_empty() {
            None
        } else {
            Some(system_pkg_info)
        },
    };

    pkgs_cache.insert(base_dir.to_string(), base_dir_pkg_info);

    Ok(())
}
