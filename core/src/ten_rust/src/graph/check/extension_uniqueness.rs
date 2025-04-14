//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use anyhow::Result;

use crate::{graph::Graph, pkg_info::pkg_type::PkgType};

impl Graph {
    /// Validates that no extension nodes are duplicated within the graph.
    ///
    /// # Extension Uniqueness Rule
    /// Extensions are considered duplicated if they have the same name within
    /// the same application (identified by app URI). Each extension must
    /// have a unique `app_uri:extension_name` identifier.
    ///
    /// # Returns
    /// - `Ok(())` if validation passes (no duplicates found)
    /// - `Err` with descriptive message if duplicates are found
    pub fn check_extension_uniqueness(&self) -> Result<()> {
        // Vector to track all extensions by their unique identifier
        let mut all_extensions: Vec<String> = Vec::new();

        // Iterate through all nodes in the graph to find extensions
        for (node_idx, node) in self.nodes.iter().enumerate() {
            if node.type_and_name.pkg_type == PkgType::Extension {
                // Create a unique identifier by combining app URI and extension
                // name
                let unique_ext_name = format!(
                    "{}:{}",
                    node.get_app_uri().map_or("", |s| s.as_str()),
                    node.type_and_name.name
                );

                // Check if this extension already exists in our tracking list
                if all_extensions.contains(&unique_ext_name) {
                    return Err(anyhow::anyhow!(
                      "Duplicated extension was found in nodes[{}], addon: {}, name: {}.",
                      node_idx, node.addon, node.type_and_name.name
                  ));
                }

                // Add this extension to our tracking list
                all_extensions.push(unique_ext_name);
            }
        }

        Ok(())
    }
}
