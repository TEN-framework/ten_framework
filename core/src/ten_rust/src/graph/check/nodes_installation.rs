//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::collections::HashMap;

use anyhow::Result;

use crate::{
    base_dir_pkg_info::{PkgsInfoInApp, PkgsInfoInAppWithBaseDir},
    graph::Graph,
    pkg_info::pkg_type::PkgType,
};

impl Graph {
    /// Verifies that all nodes in the graph have corresponding installed
    /// packages.
    pub fn check_nodes_installation(
        &self,
        graph_app_base_dir: Option<&String>,
        pkgs_cache: &HashMap<String, PkgsInfoInApp>,
        uri_to_pkg_info: &HashMap<Option<String>, PkgsInfoInAppWithBaseDir>,
        ignore_missing_apps: bool,
    ) -> Result<()> {
        // Collection to store missing packages as tuples of (app_uri,
        // node_type, node_addon_name).
        let mut not_installed_pkgs: Vec<(String, PkgType, String)> = Vec::new();

        // Iterate through all nodes in the graph to verify their installation
        // status.
        for node in &self.nodes {
            let mut pkgs_info_in_app = uri_to_pkg_info
                .get(&node.app)
                .map(|pkgs_info_in_app| pkgs_info_in_app.pkgs_info_in_app);

            if pkgs_info_in_app.is_none() {
                if let Some(graph_app_base_dir) = graph_app_base_dir {
                    pkgs_info_in_app = pkgs_cache.get(graph_app_base_dir);
                }
            }

            if let Some(pkgs_info_in_app) = pkgs_info_in_app {
                // Search for the package using the helper method.
                let found = pkgs_info_in_app.find_pkg_by_type_and_name(
                    node.type_and_name.pkg_type,
                    &node.addon,
                );

                // If the node is not found, add it to the missing packages
                // list.
                if found.is_none() && !ignore_missing_apps {
                    not_installed_pkgs.push((
                        node.app
                            .as_ref()
                            .map(|s| s.to_string())
                            .unwrap_or("".to_string()),
                        node.type_and_name.pkg_type,
                        node.addon.clone(),
                    ));
                }
            } else if !ignore_missing_apps {
                not_installed_pkgs.push((
                    node.app
                        .as_ref()
                        .map(|s| s.to_string())
                        .unwrap_or("".to_string()),
                    node.type_and_name.pkg_type,
                    node.addon.clone(),
                ));
            }
        }

        type FilterFn = Box<dyn Fn(&(String, PkgType, String)) -> bool>;

        // Define filters for packages that don't need to be physically
        // installed Each filter returns true if the package should be checked
        // (not exempted).
        let filters: Vec<FilterFn> = vec![Box::new(|pkg| {
            !(pkg.1 == PkgType::Extension && pkg.2 == *"ten:test_extension")
        })];

        // Filter out those known addons that do not need to be installed in the
        // file system.
        not_installed_pkgs.retain(|pkg| filters.iter().all(|f| f(pkg)));

        // Return error if there are any non-exempted missing packages.
        if !not_installed_pkgs.is_empty() {
            return Err(anyhow::anyhow!(
                "The following packages are declared in nodes but not installed: {:?}.",
                not_installed_pkgs
            ));
        }

        Ok(())
    }
}
