//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::{constants, GraphNodeAppDeclaration};

use crate::graph::is_app_default_loc_or_none;
use crate::pkg_info::{localhost, pkg_type_and_name::PkgTypeAndName};

/// Represents a node in a graph. This struct is completely equivalent to the
/// node element in the graph JSON.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GraphNode {
    #[serde(flatten)]
    pub type_and_name: PkgTypeAndName,

    pub addon: String,

    // extension group node does not contain extension_group field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extension_group: Option<String>,

    #[serde(skip_serializing_if = "is_app_default_loc_or_none")]
    pub app: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub property: Option<serde_json::Value>,
}

impl GraphNode {
    /// Validates and completes a graph node by ensuring it has all required
    /// fields and follows the app declaration rules of the graph.
    ///
    /// This method performs the following validations:
    /// 1. Ensures extension nodes have an extension_group defined.
    /// 2. Validates app URI according to graph's app declaration state.
    /// 3. Sets default app URI if none is provided.
    ///
    /// # Arguments
    /// * `graph_node_app_declaration` - The app declaration state of the entire
    ///   graph.
    ///
    /// # Returns
    /// * `Ok(())` if validation succeeds.
    /// * `Err` with a descriptive error message if validation fails.
    pub fn validate_and_complete(
        &mut self,
        graph_node_app_declaration: &GraphNodeAppDeclaration,
    ) -> Result<()> {
        // Check if app URI is provided and validate it.
        if let Some(app) = &self.app {
            // Disallow 'localhost' as an app URI in graph definitions.
            if app.as_str() == localhost() {
                let err_msg = if graph_node_app_declaration
                    .is_single_app_graph()
                {
                    constants::ERR_MSG_GRAPH_LOCALHOST_FORBIDDEN_IN_SINGLE_APP_MODE
                } else {
                    constants::ERR_MSG_GRAPH_LOCALHOST_FORBIDDEN_IN_MULTI_APP_MODE
                };

                return Err(anyhow::anyhow!(err_msg));
            }
        } else {
            // If app URI is not provided, set it to localhost.
            // Note: This might be problematic if the graph has mixed app
            // declarations but that case should be caught by
            // Graph::determine_graph_node_app_declaration.
            self.app = Some(localhost().to_string());
        }

        Ok(())
    }

    pub fn get_app_uri(&self) -> &str {
        // The 'app' should be assigned after 'validate_and_complete' is called,
        // so it should not be None.
        self.app.as_ref().unwrap().as_str()
    }
}
