//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use ten_rust::{
    base_dir_pkg_info::BaseDirPkgInfo,
    pkg_info::{
        pkg_type::PkgType, predefined_graphs::pkg_predefined_graphs_find,
        property::predefined_graph::PredefinedGraph, PkgInfo,
    },
};

/// Finds the app package in the list of packages
pub fn find_app_package(pkgs: &mut [PkgInfo]) -> Option<&mut PkgInfo> {
    pkgs.iter_mut().find(|pkg| {
        pkg.manifest
            .as_ref()
            .is_some_and(|m| m.type_and_name.pkg_type == PkgType::App)
    })
}

/// Finds the app package directly from BaseDirPkgInfo.
pub fn find_app_package_from_base_dir(
    base_dir_pkg_info: &mut BaseDirPkgInfo,
) -> Option<&mut PkgInfo> {
    base_dir_pkg_info.app_pkg_info.as_mut()
}

/// Finds a predefined graph in the app package by name.
pub fn find_predefined_graph<'a>(
    app_pkg: &'a mut PkgInfo,
    graph_name: &str,
) -> Option<&'a PredefinedGraph> {
    pkg_predefined_graphs_find(app_pkg.get_predefined_graphs(), |g| {
        g.name == graph_name
    })
}
