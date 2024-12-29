//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::pkg_info::graph::Graph;

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

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::pkg_info::property::Property;

    #[test]
    fn test_property_predefined_graph_deserialize() {
        let property_str = include_str!("test_data_embed/property.json");
        let property = Property::from_str(property_str).unwrap();
        assert!(property._ten.is_some());

        let predefined_graphs = property
            ._ten
            .as_ref()
            .unwrap()
            .predefined_graphs
            .as_ref()
            .unwrap();
        assert!(predefined_graphs.len() == 1);

        let predefined_graph = predefined_graphs.first().unwrap().clone();
        let property_json_value =
            serde_json::to_value(&predefined_graph).unwrap();
        assert_eq!(
            property_json_value.as_object().unwrap()["connections"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
    }
}
