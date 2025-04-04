//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::collections::HashMap;

use anyhow::Result;

use crate::{
    base_dir_pkg_info::BaseDirPkgInfo, graph::Graph,
    pkg_info::pkg_type::PkgType,
};

impl Graph {
    /// Verifies that all nodes in the graph have corresponding installed
    /// packages.
    pub fn check_nodes_installation(
        &self,
        installed_pkgs_of_all_apps: &HashMap<String, BaseDirPkgInfo>,
        ignore_missing_apps: bool,
    ) -> Result<()> {
        // Collection to store missing packages as tuples of (app_uri,
        // node_type, node_addon_name).
        let mut not_installed_pkgs: Vec<(String, PkgType, String)> = Vec::new();

        // Iterate through all nodes in the graph to verify their installation
        // status.
        for node in &self.nodes {
            // Get app URI or empty string if None.
            let node_app_uri = node.get_app_uri().unwrap_or("");

            // Verify if the node's app exists in our app mapping.
            if !installed_pkgs_of_all_apps.contains_key(node_app_uri) {
                // If app doesn't exist and we're not skipping such cases, add
                // it to missing packages.
                if !ignore_missing_apps {
                    let uri_for_error = node_app_uri.to_string();
                    not_installed_pkgs.push((
                        uri_for_error,
                        node.type_and_name.pkg_type,
                        node.addon.clone(),
                    ));
                }

                continue;
            }

            // Get the BaseDirPkgInfo for this app.
            let installed_pkgs_of_app =
                installed_pkgs_of_all_apps.get(node_app_uri).unwrap();

            // Search for the package using the helper method.
            let found = installed_pkgs_of_app.find_pkg_by_type_and_name(
                node.type_and_name.pkg_type,
                &node.addon,
            );

            // If the node is not found, add it to the missing packages list.
            if found.is_none() {
                let uri_for_error = node_app_uri.to_string();
                not_installed_pkgs.push((
                    uri_for_error,
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
