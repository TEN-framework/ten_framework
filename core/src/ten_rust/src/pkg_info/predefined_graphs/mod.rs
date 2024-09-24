//
// Copyright © 2024 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
pub mod extension;

use anyhow::Result;

use crate::pkg_info::{
    graph::Graph,
    property::{predefined_graph::PropertyPredefinedGraph, Property},
};

use super::graph::{GraphConnection, GraphNode};

pub fn get_pkg_predefined_graphs_from_property(
    property: &Property,
) -> Result<Vec<PropertyPredefinedGraph>> {
    let mut graphs = Vec::new();

    if let Some(ten) = &property._ten {
        if let Some(predefined_graphs) = &ten.predefined_graphs {
            for property_predefined_graph in predefined_graphs {
                graphs.push(property_predefined_graph.clone());
            }
        }
    }

    Ok(graphs)
}

pub fn pkg_predefined_graphs_find<F>(
    pkg_predefined_graphs: &[PropertyPredefinedGraph],
    predicate: F,
) -> Option<&PropertyPredefinedGraph>
where
    F: Fn(&&PropertyPredefinedGraph) -> bool,
{
    pkg_predefined_graphs.iter().find(predicate)
}

pub fn get_pkg_predefined_graph_from_nodes_and_connections(
    graph_name: &str,
    auto_start: bool,
    nodes: &Vec<GraphNode>,
    connections: &Vec<GraphConnection>,
) -> Result<PropertyPredefinedGraph> {
    Ok(PropertyPredefinedGraph {
        name: graph_name.to_string(),
        auto_start: Some(auto_start),
        graph: Graph {
            nodes: nodes.to_vec(),
            connections: Some(connections.to_vec()),
        },
    })
}
