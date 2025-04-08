//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::pkg_info::pkg_type::PkgType;

use super::Graph;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GraphInfo {
    pub name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_start: Option<bool>,

    #[serde(flatten)]
    pub graph: Graph,

    #[serde(skip)]
    pub app_base_dir: Option<String>,
    #[serde(skip)]
    pub belonging_pkg_type: PkgType,
    #[serde(skip)]
    pub belonging_pkg_name: Option<String>,
}

impl GraphInfo {
    pub fn get_id(&self) -> String {
        let mut id = String::new();

        // Add app_base_dir prefix if it exists.
        if let Some(base_dir) = &self.app_base_dir {
            id.push_str(base_dir);
            id.push('-');
        }

        // Add belonging_pkg_type.
        id.push_str(&format!("{}-", self.belonging_pkg_type));

        // Add belonging_pkg_name if it exists.
        if let Some(pkg_name) = &self.belonging_pkg_name {
            id.push_str(pkg_name);
            id.push('-');
        }

        // Add graph_name.
        id.push_str(&self.name);

        id
    }

    pub fn validate_and_complete(&mut self) -> Result<()> {
        self.graph.validate_and_complete()
    }
}
