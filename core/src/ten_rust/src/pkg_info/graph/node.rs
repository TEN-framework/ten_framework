//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::pkg_info::{
    localhost, pkg_type::PkgType, pkg_type_and_name::PkgTypeAndName,
};

use super::{constants, GraphNodeAppDeclaration};
use crate::pkg_info::graph::is_app_default_loc_or_none;

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
    pub fn validate_and_complete(
        &mut self,
        graph_node_app_declaration: &GraphNodeAppDeclaration,
    ) -> Result<()> {
        // Extension node must specify extension_group name.
        if self.type_and_name.pkg_type == PkgType::Extension
            && self.extension_group.is_none()
        {
            return Err(anyhow::anyhow!(
                "Node '{}' of type 'extension' must have an 'extension_group' defined.",
                self.type_and_name.name
            ));
        }

        if let Some(app) = &self.app {
            if app.as_str() == localhost() {
                let err_msg =
                    if graph_node_app_declaration.is_single_app_graph() {
                        constants::ERR_MSG_APP_LOCALHOST_DISALLOWED_SINGLE
                    } else {
                        constants::ERR_MSG_APP_LOCALHOST_DISALLOWED_MULTI
                    };

                return Err(anyhow::anyhow!(err_msg));
            }
        } else {
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
