//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
mod check;
pub mod connection;
mod constants;
pub mod msg_conversion;
pub mod node;

use std::{collections::HashMap, str::FromStr};

use anyhow::Result;
use connection::GraphConnection;
use node::GraphNode;
use serde::{Deserialize, Serialize};

use super::PkgInfo;
use crate::pkg_info::localhost;

/// The state of the 'app' field declaration in all nodes in the graph.
///
/// There might be the following cases for the 'app' field declaration:
///
/// - Case 1: neither of the nodes has declared the 'app' field. The state will
///   be `NoneDeclared`.
///
/// - Case 2: all nodes have declared the 'app' field, and all of them have the
///   same value. Ex:
///
///   {
///     "nodes": [
///       {
///         "type": "extension",
///         "app": "http://localhost:8000",
///         "addon": "addon_1",
///         "name": "ext_1",
///         "extension_group": "some_group"
///       },
///       {
///         "type": "extension",
///         "app": "http://localhost:8000",
///         "addon": "addon_2",
///         "name": "ext_2",
///         "extension_group": "another_group"
///       }
///     ]
///   }
///
///   The state will be `UniformDeclared`.
///
/// - Case 3: all nodes have declared the 'app' field, but they have different
///   values.
///
///   {
///     "nodes": [
///       {
///         "type": "extension",
///         "app": "http://localhost:8000",
///         "addon": "addon_1",
///         "name": "ext_1",
///         "extension_group": "some_group"
///       },
///       {
///         "type": "extension",
///         "app": "msgpack://localhost:8001",
///         "addon": "addon_2",
///         "name": "ext_2",
///         "extension_group": "another_group"
///       }
///     ]
///   }
///
///   The state will be `MixedDeclared`.
///
/// - Case 4: some nodes have declared the 'app' field, and some have not. It's
///   illegal.
///
/// In the view of the 'app' field declaration, there are two types of graphs:
///
/// * Single-app graph: the state is `NoneDeclared` or `UniformDeclared`.
/// * Multi-app graph: the state is `MixedDeclared`.

#[derive(Debug, Clone, PartialEq)]
pub enum GraphNodeAppDeclaration {
    NoneDeclared,
    UniformDeclared,
    MixedDeclared,
}

impl GraphNodeAppDeclaration {
    pub fn is_single_app_graph(&self) -> bool {
        match self {
            GraphNodeAppDeclaration::NoneDeclared => true,
            GraphNodeAppDeclaration::UniformDeclared => true,
            GraphNodeAppDeclaration::MixedDeclared => false,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Graph {
    // It's invalid that a graph does not contain any nodes.
    pub nodes: Vec<GraphNode>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub connections: Option<Vec<GraphConnection>>,
}

impl FromStr for Graph {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut graph: Graph = serde_json::from_str(s)?;

        graph.validate_and_complete()?;

        // Return the parsed data.
        Ok(graph)
    }
}

impl Graph {
    /// Determines how app URIs are declared across all nodes in the graph.
    ///
    /// This method analyzes all nodes in the graph to determine the app
    /// declaration state:
    /// - If no nodes have an 'app' field declared, returns `NoneDeclared`.
    /// - If all nodes have the same 'app' URI declared, returns
    ///   `UniformDeclared`.
    /// - If all nodes have 'app' fields but with different URIs, returns
    ///   `MixedDeclared`.
    /// - If some nodes have 'app' fields and others don't, returns an error as
    ///   this is invalid.
    ///
    /// Graphs can be categorized based on the number of apps:
    /// - A graph for a single app
    /// - A graph spanning multiple apps
    ///
    /// If none of the nodes have the app field defined, then it represents a
    /// graph for a single app. If some nodes have the app field defined while
    /// others do not, then it's not a valid graph, because TEN doesn't know
    /// which app the nodes without the defined field belong to.
    ///
    /// So, the only valid case where nodes don't define the app field is when
    /// all nodes in the graph lack the app field.
    ///
    /// # Returns
    /// * `Ok(GraphNodeAppDeclaration)` - The determined app declaration state.
    /// * `Err` - If any node has an empty app URI or if there's a mix of nodes
    ///   with and without app declarations.
    fn determine_graph_node_app_declaration(
        &self,
    ) -> Result<GraphNodeAppDeclaration> {
        let mut nodes_have_declared_app = 0;
        let mut app_uris = std::collections::HashSet::new();

        for (idx, node) in self.nodes.iter().enumerate() {
            if let Some(app_uri) = &node.app {
                if app_uri.is_empty() {
                    return Err(anyhow::anyhow!(
                        "nodes[{}]: {}",
                        idx,
                        constants::ERR_MSG_GRAPH_APP_FIELD_EMPTY
                    ));
                }

                app_uris.insert(app_uri);
                nodes_have_declared_app += 1;
            }
        }

        // Case 4: Some nodes have 'app' declared and some don't - this is
        // invalid.
        if nodes_have_declared_app != 0
            && nodes_have_declared_app != self.nodes.len()
        {
            return Err(anyhow::anyhow!(
                constants::ERR_MSG_GRAPH_MIXED_APP_DECLARATIONS
            ));
        }

        match app_uris.len() {
            // Case 1: No nodes have 'app' declared.
            0 => Ok(GraphNodeAppDeclaration::NoneDeclared),

            // Case 2: All nodes have the same 'app' URI declared.
            1 => Ok(GraphNodeAppDeclaration::UniformDeclared),

            // Case 3: All nodes have 'app' declared but with different URIs.
            _ => Ok(GraphNodeAppDeclaration::MixedDeclared),
        }
    }

    /// Validates and completes the graph structure by ensuring all nodes and
    /// connections are properly configured.
    ///
    /// This method performs the following steps:
    /// 1. Determines how app URIs are declared across nodes (none, uniform, or
    ///    mixed).
    /// 2. Validates and completes each node with the app declaration context.
    /// 3. Validates and completes each connection with the app declaration
    ///    context.
    ///
    /// # Returns
    /// * `Ok(())` if validation succeeds and the graph is properly completed.
    /// * `Err` with a descriptive error message if validation fails, including
    ///   the index of the problematic node or connection
    pub fn validate_and_complete(&mut self) -> Result<()> {
        // First determine how app URIs are declared across all nodes.
        let graph_node_app_declaration =
            self.determine_graph_node_app_declaration()?;

        // Validate and complete each node.
        for (idx, node) in self.nodes.iter_mut().enumerate() {
            node.validate_and_complete(&graph_node_app_declaration)
                .map_err(|e| anyhow::anyhow!("nodes[{}]: {}", idx, e))?;
        }

        // Validate and complete connections if they exist.
        if let Some(connections) = &mut self.connections {
            for (idx, connection) in connections.iter_mut().enumerate() {
                connection
                    .validate_and_complete(&graph_node_app_declaration)
                    .map_err(|e| {
                        anyhow::anyhow!("connections[{}]: {}", idx, e)
                    })?;
            }
        }

        Ok(())
    }

    pub fn check(
        &self,
        installed_pkgs_of_all_apps: &HashMap<String, Vec<PkgInfo>>,
    ) -> Result<()> {
        self.check_extension_uniqueness()?;
        self.check_extension_existence()?;
        self.check_connection_extensions_exist()?;

        self.check_nodes_installation(installed_pkgs_of_all_apps, false)?;
        self.check_connections_compatibility(
            installed_pkgs_of_all_apps,
            false,
        )?;

        self.check_extension_uniqueness_in_connections()?;
        self.check_message_names()?;

        Ok(())
    }

    pub fn check_for_single_app(
        &self,
        installed_pkgs_of_all_apps: &HashMap<String, Vec<PkgInfo>>,
    ) -> Result<()> {
        assert!(installed_pkgs_of_all_apps.len() == 1);

        self.check_extension_uniqueness()?;
        self.check_extension_existence()?;
        self.check_connection_extensions_exist()?;

        // In a single app, there is no information about pkg_info of other
        // apps, neither the message schemas.
        self.check_nodes_installation(installed_pkgs_of_all_apps, true)?;
        self.check_connections_compatibility(installed_pkgs_of_all_apps, true)?;

        self.check_extension_uniqueness_in_connections()?;
        self.check_message_names()?;

        Ok(())
    }
}

/// Checks if the application URI is either not specified (None) or set to the
/// default localhost value.
///
/// This function is used to determine if an application's URI is using the
/// default location, which helps decide whether the URI field should be
/// included when serializing property data.
///
/// # Arguments
/// * `app_uri` - An Option containing the application's URI string, or None if
///   not specified.
///
/// # Returns
/// * `true` if the URI is None or matches the default localhost value.
/// * `false` if the URI is specified and different from the default localhost.
pub fn is_app_default_loc_or_none(app_uri: &Option<String>) -> bool {
    match app_uri {
        None => true,
        Some(uri) => uri == &localhost(),
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::pkg_info::{
        graph::constants::{
            ERR_MSG_GRAPH_APP_FIELD_SHOULD_BE_DECLARED,
            ERR_MSG_GRAPH_APP_FIELD_SHOULD_NOT_BE_DECLARED,
        },
        property::Property,
    };

    use super::*;

    fn check_extension_existence_and_uniqueness(graph: &Graph) -> Result<()> {
        graph.check_extension_existence()?;
        graph.check_extension_uniqueness()?;
        Ok(())
    }

    #[test]
    fn test_predefined_graph_has_no_extensions() {
        let property_str =
            include_str!("test_data_embed/predefined_graph_no_extensions.json");
        let property: Property = Property::from_str(property_str).unwrap();
        let ten = property._ten.as_ref().unwrap();
        let predefined_graph =
            ten.predefined_graphs.as_ref().unwrap().first().unwrap();
        let graph = &predefined_graph.graph;
        let result = check_extension_existence_and_uniqueness(graph);
        assert!(result.is_err());
        println!("Error: {:?}", result.err().unwrap());
    }

    #[test]
    fn test_predefined_graph_has_extension_duplicated() {
        let property_str = include_str!(
            "test_data_embed/predefined_graph_has_duplicated_extension.json"
        );
        let property: Property = Property::from_str(property_str).unwrap();
        let ten = property._ten.as_ref().unwrap();
        let predefined_graph =
            ten.predefined_graphs.as_ref().unwrap().first().unwrap();
        let graph = &predefined_graph.graph;
        let result = check_extension_existence_and_uniqueness(graph);
        assert!(result.is_err());
        println!("Error: {:?}", result.err().unwrap());
    }

    #[test]
    fn test_start_graph_cmd_has_extension_duplicated() {
        let cmd_str = include_str!(
            "test_data_embed/start_graph_cmd_has_duplicated_extension.json"
        );

        let graph: Graph = Graph::from_str(cmd_str).unwrap();
        let result = check_extension_existence_and_uniqueness(&graph);
        assert!(result.is_err());
        println!("Error: {:?}", result.err().unwrap());
    }

    #[test]
    fn test_predefined_graph_connection_src_not_found() {
        let property_str = include_str!(
            "test_data_embed/predefined_graph_connection_src_not_found.json"
        );
        let property: Property = Property::from_str(property_str).unwrap();
        let ten = property._ten.as_ref().unwrap();
        let predefined_graph =
            ten.predefined_graphs.as_ref().unwrap().first().unwrap();

        let graph = &predefined_graph.graph;
        let result = graph.check_connection_extensions_exist();
        assert!(result.is_err());
        println!("Error: {:?}", result.err().unwrap());
    }

    #[test]
    fn test_predefined_graph_connection_dest_not_found() {
        let property_str = include_str!(
            "test_data_embed/predefined_graph_connection_dest_not_found.json"
        );
        let property: Property = Property::from_str(property_str).unwrap();
        let ten = property._ten.as_ref().unwrap();
        let predefined_graph =
            ten.predefined_graphs.as_ref().unwrap().first().unwrap();

        let graph = &predefined_graph.graph;
        let result = graph.check_connection_extensions_exist();
        assert!(result.is_err());
        println!("Error: {:?}", result.err().unwrap());
    }

    #[test]
    fn test_predefined_graph_node_app_localhost() {
        let property_str = include_str!(
            "test_data_embed/predefined_graph_connection_app_localhost.json"
        );
        let property = Property::from_str(property_str);

        // 'localhost' is not allowed in graph definition.
        assert!(property.is_err());
        println!("Error: {:?}", property);

        let msg = property.err().unwrap().to_string();
        assert!(msg.contains(
            constants::ERR_MSG_GRAPH_LOCALHOST_FORBIDDEN_IN_SINGLE_APP_MODE
        ));
    }

    #[test]
    fn test_start_graph_cmd_single_app_node_app_localhost() {
        let graph_str = include_str!(
            "test_data_embed/start_graph_cmd_single_app_node_app_localhost.json"
        );
        let graph = Graph::from_str(graph_str);

        // 'localhost' is not allowed in graph definition.
        assert!(graph.is_err());
        println!("Error: {:?}", graph);

        let msg = graph.err().unwrap().to_string();
        assert!(msg.contains(
            constants::ERR_MSG_GRAPH_LOCALHOST_FORBIDDEN_IN_SINGLE_APP_MODE
        ));
    }

    #[test]
    fn test_start_graph_cmd_multi_apps_node_app_localhost() {
        let graph_str = include_str!(
            "test_data_embed/start_graph_cmd_multi_apps_node_app_localhost.json"
        );
        let graph = Graph::from_str(graph_str);

        // 'localhost' is not allowed in graph definition.
        assert!(graph.is_err());
        println!("Error: {:?}", graph);

        let msg = graph.err().unwrap().to_string();
        assert!(msg.contains(
            constants::ERR_MSG_GRAPH_LOCALHOST_FORBIDDEN_IN_MULTI_APP_MODE
        ));
    }

    #[test]
    fn test_predefined_graph_connection_app_localhost() {
        let property_str = include_str!(
            "test_data_embed/predefined_graph_connection_app_localhost.json"
        );
        let property = Property::from_str(property_str);

        // 'localhost' is not allowed in graph definition.
        assert!(property.is_err());
        println!("Error: {:?}", property);

        let msg = property.err().unwrap().to_string();
        assert!(msg.contains(
            constants::ERR_MSG_GRAPH_LOCALHOST_FORBIDDEN_IN_SINGLE_APP_MODE
        ));
    }

    #[test]
    fn test_predefined_graph_app_in_nodes_not_all_declared() {
        let property_str = include_str!(
            "test_data_embed/predefined_graph_app_in_nodes_not_all_declared.json"
        );
        let property = Property::from_str(property_str);

        // Either all nodes should have 'app' declared, or none should, but not
        // a mix of both.
        assert!(property.is_err());
        println!("Error: {:?}", property);

        let msg = property.err().unwrap().to_string();
        assert!(msg.contains(
            "Either all nodes should have 'app' declared, or none should, but not a mix of both."
        ));
    }

    #[test]
    fn test_predefined_graph_app_in_connections_not_all_declared() {
        let property_str = include_str!(
            "test_data_embed/predefined_graph_app_in_connections_not_all_declared.json"
        );
        let property = Property::from_str(property_str);

        // The 'app' can not be none, as it has been declared in nodes.
        assert!(property.is_err());
        println!("Error: {:?}", property);

        let msg = property.err().unwrap().to_string();
        assert!(msg.contains(ERR_MSG_GRAPH_APP_FIELD_SHOULD_BE_DECLARED));
    }

    #[test]
    fn test_predefined_graph_app_in_connections_should_not_declared() {
        let property_str = include_str!(
            "test_data_embed/predefined_graph_app_in_connections_should_not_declared.json"
        );
        let property = Property::from_str(property_str);

        // The 'app' should not be declared, as not any node has declared it.
        assert!(property.is_err());
        println!("Error: {:?}", property);

        let msg = property.err().unwrap().to_string();
        assert!(msg.contains(ERR_MSG_GRAPH_APP_FIELD_SHOULD_NOT_BE_DECLARED));
    }

    #[test]
    fn test_predefined_graph_app_in_dest_not_all_declared() {
        let property_str = include_str!(
            "test_data_embed/predefined_graph_app_in_dest_not_all_declared.json"
        );
        let property = Property::from_str(property_str);

        // The 'app' can not be none, as it has been declared in nodes.
        assert!(property.is_err());
        println!("Error: {:?}", property);

        let msg = property.err().unwrap().to_string();
        assert!(msg.contains(ERR_MSG_GRAPH_APP_FIELD_SHOULD_BE_DECLARED));
    }

    #[test]
    fn test_predefined_graph_app_in_dest_should_not_declared() {
        let property_str = include_str!(
            "test_data_embed/predefined_graph_app_in_dest_should_not_declared.json"
        );
        let property = Property::from_str(property_str);

        // The 'app' should not be declared, as not any node has declared it.
        assert!(property.is_err());
        println!("Error: {:?}", property);

        let msg = property.err().unwrap().to_string();
        assert!(msg.contains(ERR_MSG_GRAPH_APP_FIELD_SHOULD_NOT_BE_DECLARED));
    }

    #[test]
    fn test_graph_same_extension_in_two_section_of_connections() {
        let graph_str = include_str!(
            "test_data_embed/graph_same_extension_in_two_section_of_connections.json"
        );

        let graph = Graph::from_str(graph_str).unwrap();

        let result = graph.check_extension_uniqueness_in_connections();

        assert!(result.is_err());
        println!("Error: {:?}", result);

        let msg = result.err().unwrap().to_string();
        assert!(msg.contains("extension 'some_extension' is defined in connection[0] and connection[1]"));
    }

    #[test]
    fn test_graph_duplicated_cmd_name_in_one_connection() {
        let graph_str = include_str!(
            "test_data_embed/graph_duplicated_cmd_name_in_one_connection.json"
        );

        let graph = Graph::from_str(graph_str).unwrap();
        let result = graph.check_message_names();
        assert!(result.is_err());
        println!("Error: {:?}", result);

        let msg = result.err().unwrap().to_string();
        assert!(msg.contains("'hello' is defined in flow[0] and flow[1]"));
    }

    #[test]
    fn test_graph_messages_same_name_in_different_type_are_ok() {
        let graph_str = include_str!(
            "test_data_embed/graph_messages_same_name_in_different_type_are_ok.json"
        );

        let graph = Graph::from_str(graph_str).unwrap();
        let result = graph.check_message_names();
        assert!(result.is_ok());
    }

    #[test]
    fn test_graph_app_can_not_be_empty_string() {
        let graph_str = include_str!(
            "test_data_embed/graph_app_can_not_be_empty_string.json"
        );
        let graph = Graph::from_str(graph_str);

        // The 'app' can not be empty string.
        assert!(graph.is_err());
        println!("Error: {:?}", graph);

        let msg = graph.err().unwrap().to_string();
        assert!(msg.contains(constants::ERR_MSG_GRAPH_APP_FIELD_EMPTY));
    }

    #[test]
    fn test_graph_message_conversion_fixed_value() {
        let graph_str = include_str!(
            "test_data_embed/graph_message_conversion_fixed_value.json"
        );
        let graph = Graph::from_str(graph_str).unwrap();

        let connections = graph.connections.unwrap();
        let cmd = connections
            .first()
            .unwrap()
            .cmd
            .as_ref()
            .unwrap()
            .first()
            .unwrap();
        let msg_conversion =
            cmd.dest.first().unwrap().msg_conversion.as_ref().unwrap();
        let rules = &msg_conversion.msg.rules.rules;
        assert_eq!(rules.len(), 4);
        assert_eq!(rules[1].value.as_ref().unwrap().as_str().unwrap(), "hello");
        assert!(rules[2].value.as_ref().unwrap().as_bool().unwrap());
    }
}
