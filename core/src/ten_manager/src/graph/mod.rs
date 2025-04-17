//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
pub mod compatible;
pub mod connections;
pub mod msg_conversion;
pub mod nodes;

use std::collections::HashMap;
use uuid::Uuid;

pub use connections::update_graph_connections_all_fields;
pub use nodes::update_graph_node_all_fields;

use ten_rust::graph::graph_info::GraphInfo;
use ten_rust::graph::{connection::GraphConnection, node::GraphNode, Graph};

pub fn graphs_cache_find_by_name<'a>(
    graphs_cache: &'a HashMap<Uuid, GraphInfo>,
    graph_name: &str,
) -> Option<(&'a Uuid, &'a GraphInfo)> {
    graphs_cache.iter().find_map(|(uuid, graph_info)| {
        if graph_info
            .name
            .as_ref()
            .map(|name| name == graph_name)
            .unwrap_or(false)
        {
            Some((uuid, graph_info))
        } else {
            None
        }
    })
}

pub fn graphs_cache_find_by_id<'a>(
    graphs_cache: &'a HashMap<Uuid, GraphInfo>,
    graph_id: &Uuid,
) -> Option<&'a GraphInfo> {
    graphs_cache.get(graph_id)
}

pub fn graphs_cache_find_by_id_mut<'a>(
    graphs_cache: &'a mut HashMap<Uuid, GraphInfo>,
    graph_id: &Uuid,
) -> Option<&'a mut GraphInfo> {
    graphs_cache.get_mut(graph_id)
}

// Remove graphs associated with app from graphs_cache.
pub fn graphs_cache_remove_by_app_base_dir(
    graphs_cache: &mut HashMap<Uuid, GraphInfo>,
    base_dir: &str,
) {
    // Collect UUIDs of graphs to remove.
    let graph_uuids_to_remove: Vec<uuid::Uuid> = graphs_cache
        .iter()
        .filter_map(|(uuid, graph_info)| {
            if let Some(app_base_dir) = &graph_info.app_base_dir {
                if app_base_dir == base_dir {
                    Some(*uuid)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    // Remove the graphs
    for uuid in graph_uuids_to_remove {
        graphs_cache.remove(&uuid);
    }
}

/// Replace the nodes and connections in a graph with new nodes and connections.
///
/// If the connections vector is empty, it sets graph.connections to None.
fn replace_graph_nodes_and_connections(
    graph: &mut Graph,
    nodes: &[GraphNode],
    connections: &[GraphConnection],
) {
    // Replace the nodes with a copy of the provided nodes.
    graph.nodes = nodes.to_vec();

    // If the connections vector is empty, set graph.connections to None,
    // otherwise set it to Some with a copy of the provided connections.
    if connections.is_empty() {
        graph.connections = None;
    } else {
        graph.connections = Some(connections.to_owned());
    }
}

/// Update a graph with nodes and connections from the provided request payload.
///
/// This function takes a graph ID, nodes, and connections and updates the
/// corresponding graph in the graphs cache if it exists.
pub fn update_graph_endpoint(
    graphs_cache: &mut HashMap<Uuid, GraphInfo>,
    graph_id: &Uuid,
    nodes: &[GraphNode],
    connections: &[GraphConnection],
) -> Result<(), String> {
    // Find the graph info by ID
    if let Some(graph_info) =
        graphs_cache_find_by_id_mut(graphs_cache, graph_id)
    {
        // Access the graph and update it
        replace_graph_nodes_and_connections(
            &mut graph_info.graph,
            nodes,
            connections,
        );
        Ok(())
    } else {
        Err(format!("Graph with ID {} not found", graph_id))
    }
}
