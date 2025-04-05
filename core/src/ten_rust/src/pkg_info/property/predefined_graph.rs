//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::graph::Graph;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PredefinedGraph {
    pub name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_start: Option<bool>,

    #[serde(flatten)]
    pub graph: Graph,
}

impl PredefinedGraph {
    pub fn validate_and_complete(&mut self) -> Result<()> {
        self.graph.validate_and_complete()
    }
}
