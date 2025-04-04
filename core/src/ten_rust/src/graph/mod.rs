//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
mod check;
pub mod connection;
pub mod constants;
pub mod msg_conversion;
pub mod node;

use std::{collections::HashMap, str::FromStr};

use anyhow::Result;
use connection::GraphConnection;
use serde::{Deserialize, Serialize};

use crate::{base_dir_pkg_info::BaseDirPkgInfo, pkg_info::localhost};

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
/// {
///   "nodes": [
///     {
///       "type": "extension",
///       "app": "http://localhost:8000",
///       "addon": "addon_1",
///       "name": "ext_1",
///       "extension_group": "some_group"
///     },
///     {
///       "type": "extension",
///       "app": "http://localhost:8000",
///       "addon": "addon_2",
///       "name": "ext_2",
///       "extension_group": "another_group"
///     }
///   ]
/// }
///
///   The state will be `UniformDeclared`.
///
/// - Case 3: all nodes have declared the 'app' field, but they have different
///   values.
///
/// {
///   "nodes": [
///     {
///       "type": "extension",
///       "app": "http://localhost:8000",
///       "addon": "addon_1",
///       "name": "ext_1",
///       "extension_group": "some_group"
///     },
///     {
///       "type": "extension",
///       "app": "msgpack://localhost:8001",
///       "addon": "addon_2",
///       "name": "ext_2",
///       "extension_group": "another_group"
///     }
///   ]
/// }
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
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum AppUriDeclarationState {
    /// No node has declared an app URI.
    NoneDeclared,
    /// All nodes have declared the same app URI.
    UniformDeclared,
    /// Nodes have declared different app URIs.
    MixedDeclared,
}

impl AppUriDeclarationState {
    /// Returns true if the graph can be considered a single-app graph.
    pub fn is_single_app_graph(&self) -> bool {
        matches!(
            self,
            AppUriDeclarationState::NoneDeclared
                | AppUriDeclarationState::UniformDeclared
        )
    }
}

/// Represents a connection graph that defines how extensions connect to each
/// other.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Graph {
    pub nodes: Vec<node::GraphNode>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub connections: Option<Vec<connection::GraphConnection>>,
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
    /// - A graph for a single app (NoneDeclared or UniformDeclared state)
    /// - A graph spanning multiple apps (MixedDeclared state)
    ///
    /// For a valid graph, either all nodes must have the app field defined or
    /// none of them should. If some nodes have the app field defined while
    /// others do not, it creates an invalid graph because TEN cannot
    /// determine which app the nodes without the defined field belong to.
    /// Therefore, the only valid case where nodes don't define the app
    /// field is when all nodes in the graph lack this field.
    ///
    /// For graphs spanning multiple apps, no node can have 'localhost' as its
    /// app field value, as other apps would not know how to connect to the
    /// app that node belongs to. For consistency, single app graphs also do
    /// not allow 'localhost' as an explicit app field value. Instead,
    /// 'localhost' is used as the internal default value when no app field is
    /// specified.
    fn analyze_app_uri_declaration_state(
        &self,
    ) -> Result<AppUriDeclarationState> {
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

        // Some nodes have 'app' declared and some don't - this is invalid.
        // Because TEN can not determine which app the nodes without the defined
        // field belong to.
        if nodes_have_declared_app != 0
            && nodes_have_declared_app != self.nodes.len()
        {
            return Err(anyhow::anyhow!(
                constants::ERR_MSG_GRAPH_MIXED_APP_DECLARATIONS
            ));
        }

        match app_uris.len() {
            // No nodes have 'app' declared.
            0 => Ok(AppUriDeclarationState::NoneDeclared),

            // All nodes have the same 'app' URI declared.
            1 => Ok(AppUriDeclarationState::UniformDeclared),

            // All nodes have 'app' declared but with different URIs.
            _ => Ok(AppUriDeclarationState::MixedDeclared),
        }
    }

    /// Validates and completes the graph by ensuring all nodes and connections
    /// follow the app declaration rules and other validation requirements.
    pub fn validate_and_complete(&mut self) -> Result<()> {
        // Determine the app URI declaration state by examining all nodes.
        let app_uri_declaration_state =
            self.analyze_app_uri_declaration_state()?;

        // Validate all nodes.
        for (idx, node) in self.nodes.iter_mut().enumerate() {
            node.validate_and_complete(&app_uri_declaration_state)
                .map_err(|e| anyhow::anyhow!("nodes[{}]: {}", idx, e))?;
        }

        // Validate all connections if they exist.
        if let Some(connections) = &mut self.connections {
            for (idx, connection) in connections.iter_mut().enumerate() {
                connection
                    .validate_and_complete(&app_uri_declaration_state)
                    .map_err(|e| {
                        anyhow::anyhow!("connections[{}]: {}", idx, e)
                    })?;
            }
        }

        Ok(())
    }

    pub fn check(
        &self,
        installed_pkgs_of_all_apps: &HashMap<String, BaseDirPkgInfo>,
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
        installed_pkgs_of_all_apps: &HashMap<String, BaseDirPkgInfo>,
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

    /// Helper function to convert Option<&str> to String for HashMap keys and
    /// string formatting.
    pub(crate) fn option_str_to_string(app_uri: Option<&str>) -> String {
        app_uri.map_or_else(String::new, |s| s.to_string())
    }
}

/// Checks if the application URI is either not specified (None) or set to the
/// default localhost value.
///
/// This function is used to determine if an application's URI is using the
/// default location, which helps decide whether the URI field should be
/// included when serializing property data.
pub fn is_app_default_loc_or_none(app_uri: &Option<String>) -> bool {
    match app_uri {
        None => true,
        Some(uri) => uri == localhost(),
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::{
        graph::constants::{
            ERR_MSG_GRAPH_APP_FIELD_SHOULD_BE_DECLARED,
            ERR_MSG_GRAPH_APP_FIELD_SHOULD_NOT_BE_DECLARED,
        },
        pkg_info::property::Property,
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

    #[test]
    fn test_add_extension_node() {
        // Create an empty graph.
        let mut graph = Graph {
            nodes: Vec::new(),
            connections: None,
        };

        // Test case 1: Add a valid node.
        let result = graph.add_extension_node(
            "test_extension".to_string(),
            "test_addon".to_string(),
            Some("http://test-app-uri.com".to_string()),
            None,
            None,
        );
        assert!(result.is_ok());
        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.nodes[0].type_and_name.name, "test_extension");
        assert_eq!(graph.nodes[0].addon, "test_addon");
        assert_eq!(
            graph.nodes[0].app,
            Some("http://test-app-uri.com".to_string())
        );

        // Test case 2: Add a second node.
        let result = graph.add_extension_node(
            "test_extension2".to_string(),
            "test_addon2".to_string(),
            Some("http://test-app-uri.com".to_string()), // Same app URI.
            Some("custom_group".to_string()),
            None,
        );
        assert!(result.is_ok());
        assert_eq!(graph.nodes.len(), 2);
        assert_eq!(graph.nodes[1].type_and_name.name, "test_extension2");
        assert_eq!(
            graph.nodes[1].extension_group,
            Some("custom_group".to_string())
        );

        // Test case 3: Adding a node with localhost app URI should fail.
        let original_len = graph.nodes.len();
        let result = graph.add_extension_node(
            "test_extension3".to_string(),
            "test_addon3".to_string(),
            Some(localhost().to_string()), // This is not allowed.
            None,
            None,
        );
        assert!(result.is_err());
        // Verify rollback worked.
        assert_eq!(graph.nodes.len(), original_len);

        // Test case 4: Adding a node with different app URI.
        let original_len = graph.nodes.len();
        let result = graph.add_extension_node(
            "test_extension4".to_string(),
            "test_addon4".to_string(),
            Some("http://different-uri.com".to_string()), // Different URI.
            None,
            None,
        );
        // This should be ok as mixed URIs are valid.
        assert!(result.is_ok());
        // Node should be added.
        assert_eq!(graph.nodes.len(), original_len + 1);

        // Test case 5: Adding a node with no app URI to a graph with declared
        // app URIs. This should fail because we would have mixed app
        // declarations (some with URI, some without).
        let original_len = graph.nodes.len();
        let result = graph.add_extension_node(
            "test_extension5".to_string(),
            "test_addon5".to_string(),
            None, // No app URI.
            None,
            None,
        );
        assert!(result.is_err());
        // Verify rollback worked.
        assert_eq!(graph.nodes.len(), original_len);
    }
}
