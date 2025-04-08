//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use anyhow::Result;

use crate::graph::{
    connection::GraphConnection, graph_info::GraphInfo, node::GraphNode, Graph,
};

use super::pkg_type::PkgType;

pub fn pkg_predefined_graphs_find<F>(
    pkg_predefined_graphs: Option<&Vec<GraphInfo>>,
    predicate: F,
) -> Option<&GraphInfo>
where
    F: Fn(&&GraphInfo) -> bool,
{
    match pkg_predefined_graphs {
        None => None,
        Some(pkg_predefined_graphs) => {
            pkg_predefined_graphs.iter().find(predicate)
        }
    }
}

pub fn get_pkg_predefined_graph_from_nodes_and_connections(
    graph_name: &str,
    auto_start: bool,
    nodes: &[GraphNode],
    connections: &[GraphConnection],
) -> Result<GraphInfo> {
    Ok(GraphInfo {
        name: graph_name.to_string(),
        auto_start: Some(auto_start),
        graph: Graph {
            nodes: nodes.to_vec(),
            connections: Some(connections.to_vec()),
        },
        app_base_dir: None,
        belonging_pkg_type: PkgType::Invalid,
        belonging_pkg_name: None,
    })
}
