//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
pub mod connections;
pub mod nodes;

use std::collections::HashMap;
use uuid::Uuid;

pub use connections::update_graph_connections_all_fields;
pub use nodes::update_graph_node_all_fields;

use ten_rust::graph::graph_info::GraphInfo;

pub fn graphs_cache_find_by_name<'a>(
    graphs_cache: &'a HashMap<Uuid, GraphInfo>,
    graph_name: &str,
) -> Option<&'a GraphInfo> {
    graphs_cache.iter().find_map(|(_, graph_info)| {
        if graph_info
            .name
            .as_ref()
            .map(|name| name == graph_name)
            .unwrap_or(false)
        {
            Some(graph_info)
        } else {
            None
        }
    })
}

pub fn graphs_cache_find_by_id(
    graphs_cache: &HashMap<Uuid, GraphInfo>,
    graph_id: Uuid,
) -> Option<&GraphInfo> {
    graphs_cache.get(&graph_id)
}
