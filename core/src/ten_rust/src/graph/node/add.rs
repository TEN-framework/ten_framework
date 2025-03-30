//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use anyhow::Result;

use crate::{
    graph::Graph,
    pkg_info::{pkg_type::PkgType, pkg_type_and_name::PkgTypeAndName},
};

use super::GraphNode;

impl Graph {
    /// Adds a new extension node to the graph and validates the graph after
    /// adding it.
    pub fn add_extension_node(
        &mut self,
        pkg_name: String,
        addon: String,
        app: Option<String>,
        extension_group: Option<String>,
        property: Option<serde_json::Value>,
    ) -> Result<()> {
        // Store the original state in case validation fails.
        let original_graph = self.clone();

        // Create new GraphNode.
        let node = GraphNode {
            type_and_name: PkgTypeAndName {
                pkg_type: PkgType::Extension,
                name: pkg_name,
            },
            addon,
            extension_group,
            app,
            property,
        };

        // Add the node to the graph.
        self.nodes.push(node);

        // Validate the graph.
        match self.validate_and_complete() {
            Ok(_) => Ok(()),
            Err(e) => {
                // Restore the original graph if validation fails.
                *self = original_graph;
                Err(e)
            }
        }
    }
}
