//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use anyhow::Result;
use std::collections::HashMap;

use crate::graph::{
    connection::{GraphConnection, GraphDestination, GraphMessageFlow},
    Graph,
};
use crate::pkg_info::pkg_type::PkgType;
use crate::pkg_info::{message::MsgType, PkgInfo};
use crate::schema::store::{
    are_cmd_schemas_compatible, are_ten_schemas_compatible,
};

/// Type alias for package info tuple returned by find_pkg_infos.
type PkgInfoTuple<'a> = (Option<&'a PkgInfo>, Option<&'a PkgInfo>);

impl Graph {
    /// Adds a new connection between two extension nodes in the graph.
    #[allow(clippy::too_many_arguments)]
    pub fn add_connection(
        &mut self,
        src_app: Option<String>,
        src_extension: String,
        msg_type: MsgType,
        msg_name: String,
        dest_app: Option<String>,
        dest_extension: String,
        installed_pkgs_of_all_apps: &HashMap<String, Vec<PkgInfo>>,
    ) -> Result<()> {
        // Store the original state in case validation fails.
        let original_graph = self.clone();

        // Check if nodes exist.
        Self::check_nodes_exist(
            self,
            &src_app,
            &src_extension,
            &dest_app,
            &dest_extension,
        )?;

        // Check if connection already exists.
        Self::check_connection_exists(
            self,
            &src_app,
            &src_extension,
            &msg_type,
            &msg_name,
            &dest_app,
            &dest_extension,
        )?;

        // Find source and destination package info.
        let (src_extension_pkg_info, dest_extension_pkg_info) =
            Self::find_pkg_infos(
                installed_pkgs_of_all_apps,
                &src_app,
                &src_extension,
                &dest_app,
                &dest_extension,
            )?;

        // Check schema compatibility.
        Self::check_schema_compatibility(
            &msg_type,
            &msg_name,
            &src_extension_pkg_info,
            &dest_extension_pkg_info,
        )?;

        // Create destination object.
        let destination = GraphDestination {
            app: dest_app,
            extension: dest_extension,
            msg_conversion: None,
        };

        // Initialize connections if None.
        if self.connections.is_none() {
            self.connections = Some(Vec::new());
        }

        // Create a message flow.
        let message_flow = GraphMessageFlow {
            name: msg_name,
            dest: vec![destination],
        };

        // Get or create a connection for the source node and add the message
        // flow.
        {
            let connections = self.connections.as_mut().unwrap();

            // Find or create connection.
            let connection_idx = if let Some((idx, _)) =
                connections.iter().enumerate().find(|(_, conn)| {
                    conn.extension == src_extension && conn.app == src_app
                }) {
                idx
            } else {
                // Create a new connection for the source node.
                connections.push(GraphConnection {
                    app: src_app.clone(),
                    extension: src_extension,
                    cmd: None,
                    data: None,
                    audio_frame: None,
                    video_frame: None,
                });
                connections.len() - 1
            };

            // Add the message flow to the appropriate collection.
            let connection = &mut connections[connection_idx];
            Self::add_message_flow_to_connection(
                connection,
                &msg_type,
                message_flow,
            )?;
        }

        // Validate the updated graph.
        match self.validate_and_complete() {
            Ok(_) => Ok(()),
            Err(e) => {
                // Restore the original graph if validation fails.
                *self = original_graph;
                Err(e)
            }
        }
    }

    /// Checks if source and destination nodes exist in the graph.
    fn check_nodes_exist(
        graph: &Graph,
        src_app: &Option<String>,
        src_extension: &str,
        dest_app: &Option<String>,
        dest_extension: &str,
    ) -> Result<()> {
        // Validate that source node exists.
        let src_node_exists = graph.nodes.iter().any(|node| {
            node.type_and_name.name == src_extension && node.app == *src_app
        });

        if !src_node_exists {
            return Err(anyhow::anyhow!(
                "Source node with extension '{}' and app '{:?}' not found in the graph",
                src_extension,
                src_app
            ));
        }

        // Validate that destination node exists.
        let dest_node_exists = graph.nodes.iter().any(|node| {
            node.type_and_name.name == dest_extension && node.app == *dest_app
        });

        if !dest_node_exists {
            return Err(anyhow::anyhow!(
                "Destination node with extension '{}' and app '{:?}' not found in the graph",
                dest_extension,
                dest_app
            ));
        }

        Ok(())
    }

    /// Checks if the connection already exists.
    fn check_connection_exists(
        graph: &Graph,
        src_app: &Option<String>,
        src_extension: &str,
        msg_type: &MsgType,
        msg_name: &str,
        dest_app: &Option<String>,
        dest_extension: &str,
    ) -> Result<()> {
        if let Some(connections) = &graph.connections {
            for conn in connections.iter() {
                // Check if source matches.
                if conn.extension == src_extension && conn.app == *src_app {
                    // Check for duplicate message flows based on message type.
                    let msg_flows = match msg_type {
                        MsgType::Cmd => conn.cmd.as_ref(),
                        MsgType::Data => conn.data.as_ref(),
                        MsgType::AudioFrame => conn.audio_frame.as_ref(),
                        MsgType::VideoFrame => conn.video_frame.as_ref(),
                    };

                    if let Some(flows) = msg_flows {
                        for flow in flows {
                            // Check if message name matches.
                            if flow.name == msg_name {
                                // Check if destination already exists.
                                for dest in &flow.dest {
                                    if dest.extension == dest_extension
                                        && dest.app == *dest_app
                                    {
                                        return Err(anyhow::anyhow!(
                                            "Connection already exists: src:({:?}, {}), msg_type:{:?}, msg_name:{}, dest:({:?}, {})",
                                            src_app, src_extension, msg_type, msg_name, dest_app, dest_extension
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Finds package info for source and destination apps and extensions.
    fn find_pkg_infos<'a>(
        installed_pkgs_of_all_apps: &'a HashMap<String, Vec<PkgInfo>>,
        src_app: &Option<String>,
        src_extension: &str,
        dest_app: &Option<String>,
        dest_extension: &str,
    ) -> Result<PkgInfoTuple<'a>> {
        // Find source app/extension PkgInfo.
        let mut src_app_pkg_info = None;
        let mut src_extension_pkg_info = None;

        for pkgs in installed_pkgs_of_all_apps.values() {
            // Find source app PkgInfo.
            let app_pkg = pkgs.iter().find(|pkg| {
                if let Some(app_uri) = src_app.as_ref() {
                    if let Some(property) = &pkg.property {
                        if let Some(ten) = &property._ten {
                            if let Some(uri) = &ten.uri {
                                return pkg.manifest.as_ref().is_some_and(
                                    |m| {
                                        m.type_and_name.pkg_type == PkgType::App
                                    },
                                ) && uri == app_uri;
                            }
                        }
                    }
                }
                false
            });

            if let Some(app_pkg) = app_pkg {
                src_app_pkg_info = Some(app_pkg);

                // Find source extension PkgInfo in the same app.
                src_extension_pkg_info = pkgs.iter().find(|pkg| {
                    pkg.manifest.as_ref().is_some_and(|m| {
                        m.type_and_name.pkg_type == PkgType::Extension
                            && m.type_and_name.name == src_extension
                    })
                });

                if src_extension_pkg_info.is_some() {
                    break;
                }
            }
        }

        if src_app_pkg_info.is_none() {
            return Err(anyhow::anyhow!(
                "Source app '{:?}' not found in the installed packages",
                src_app
            ));
        }

        if src_extension_pkg_info.is_none() {
            return Err(anyhow::anyhow!(
                "Source extension '{}' not found in the installed packages for app '{:?}'",
                src_extension, src_app
            ));
        }

        // Find dest app/extension PkgInfo.
        let mut dest_app_pkg_info = None;
        let mut dest_extension_pkg_info = None;

        for pkgs in installed_pkgs_of_all_apps.values() {
            // Find destination app PkgInfo.
            let app_pkg = pkgs.iter().find(|pkg| {
                if let Some(app_uri) = dest_app.as_ref() {
                    if let Some(property) = &pkg.property {
                        if let Some(ten) = &property._ten {
                            if let Some(uri) = &ten.uri {
                                return pkg.manifest.as_ref().is_some_and(
                                    |m| {
                                        m.type_and_name.pkg_type == PkgType::App
                                    },
                                ) && uri == app_uri;
                            }
                        }
                    }
                }
                false
            });

            if let Some(app_pkg) = app_pkg {
                dest_app_pkg_info = Some(app_pkg);

                // Find destination extension PkgInfo in the same app.
                dest_extension_pkg_info = pkgs.iter().find(|pkg| {
                    pkg.manifest.as_ref().is_some_and(|m| {
                        m.type_and_name.pkg_type == PkgType::Extension
                            && m.type_and_name.name == dest_extension
                    })
                });

                if dest_extension_pkg_info.is_some() {
                    break;
                }
            }
        }

        if dest_app_pkg_info.is_none() {
            return Err(anyhow::anyhow!(
                "Destination app '{:?}' not found in the installed packages",
                dest_app
            ));
        }

        if dest_extension_pkg_info.is_none() {
            return Err(anyhow::anyhow!(
                "Destination extension '{}' not found in the installed packages for app '{:?}'",
                dest_extension, dest_app
            ));
        }

        Ok((src_extension_pkg_info, dest_extension_pkg_info))
    }

    /// Checks schema compatibility between source and destination based on
    /// message type.
    fn check_schema_compatibility(
        msg_type: &MsgType,
        msg_name: &str,
        src_extension_pkg: &Option<&PkgInfo>,
        dest_extension_pkg: &Option<&PkgInfo>,
    ) -> Result<()> {
        let src_extension_pkg = src_extension_pkg.unwrap();
        let dest_extension_pkg = dest_extension_pkg.unwrap();

        match msg_type {
            MsgType::Cmd => {
                let src_schema = src_extension_pkg
                    .schema_store
                    .as_ref()
                    .and_then(|store| store.cmd_out.get(msg_name));

                let dest_schema = dest_extension_pkg
                    .schema_store
                    .as_ref()
                    .and_then(|store| store.cmd_in.get(msg_name));

                if let Err(err) =
                    are_cmd_schemas_compatible(src_schema, dest_schema)
                {
                    return Err(anyhow::anyhow!(
                        "Command schema incompatibility between source and destination: {}",
                        err
                    ));
                }
            }
            MsgType::Data => {
                let src_schema = src_extension_pkg
                    .schema_store
                    .as_ref()
                    .and_then(|store| store.data_out.get(msg_name));

                let dest_schema = dest_extension_pkg
                    .schema_store
                    .as_ref()
                    .and_then(|store| store.data_in.get(msg_name));

                if let Err(err) =
                    are_ten_schemas_compatible(src_schema, dest_schema)
                {
                    return Err(anyhow::anyhow!(
                        "Data schema incompatibility between source and destination: {}",
                        err
                    ));
                }
            }
            MsgType::AudioFrame => {
                let src_schema = src_extension_pkg
                    .schema_store
                    .as_ref()
                    .and_then(|store| store.audio_frame_out.get(msg_name));

                let dest_schema = dest_extension_pkg
                    .schema_store
                    .as_ref()
                    .and_then(|store| store.audio_frame_in.get(msg_name));

                if let Err(err) =
                    are_ten_schemas_compatible(src_schema, dest_schema)
                {
                    return Err(anyhow::anyhow!(
                        "Audio frame schema incompatibility between source and destination: {}",
                        err
                    ));
                }
            }
            MsgType::VideoFrame => {
                let src_schema = src_extension_pkg
                    .schema_store
                    .as_ref()
                    .and_then(|store| store.video_frame_out.get(msg_name));

                let dest_schema = dest_extension_pkg
                    .schema_store
                    .as_ref()
                    .and_then(|store| store.video_frame_in.get(msg_name));

                if let Err(err) =
                    are_ten_schemas_compatible(src_schema, dest_schema)
                {
                    return Err(anyhow::anyhow!(
                        "Video frame schema incompatibility between source and destination: {}",
                        err
                    ));
                }
            }
        }
        Ok(())
    }

    /// Adds a message flow to a connection based on message type.
    fn add_message_flow_to_connection(
        connection: &mut GraphConnection,
        msg_type: &MsgType,
        message_flow: GraphMessageFlow,
    ) -> Result<()> {
        // Add the message flow to the appropriate vector based on message type.
        match msg_type {
            MsgType::Cmd => {
                Self::add_to_flow(&mut connection.cmd, message_flow)
            }
            MsgType::Data => {
                Self::add_to_flow(&mut connection.data, message_flow)
            }
            MsgType::AudioFrame => {
                Self::add_to_flow(&mut connection.audio_frame, message_flow)
            }
            MsgType::VideoFrame => {
                Self::add_to_flow(&mut connection.video_frame, message_flow)
            }
        }
        Ok(())
    }

    /// Helper function to add a message flow to a specific flow collection.
    fn add_to_flow(
        flow_collection: &mut Option<Vec<GraphMessageFlow>>,
        message_flow: GraphMessageFlow,
    ) {
        if flow_collection.is_none() {
            *flow_collection = Some(Vec::new());
        }

        // Check if a message flow with the same name already exists.
        let flows = flow_collection.as_mut().unwrap();
        if let Some(existing_flow) =
            flows.iter_mut().find(|flow| flow.name == message_flow.name)
        {
            // Add the destination to the existing flow if it doesn't already
            // exist.
            if !existing_flow.dest.iter().any(|dest| {
                dest.extension == message_flow.dest[0].extension
                    && dest.app == message_flow.dest[0].app
            }) {
                existing_flow.dest.push(message_flow.dest[0].clone());
            }
        } else {
            // Add the new message flow.
            flows.push(message_flow);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::node::GraphNode;
    use crate::pkg_info::manifest::Manifest;
    use crate::pkg_info::message::MsgType;
    use crate::pkg_info::pkg_type::PkgType;
    use crate::pkg_info::pkg_type_and_name::PkgTypeAndName;
    use crate::pkg_info::property::Property;
    use crate::schema::store::SchemaStore;
    use std::collections::HashMap;
    use std::str::FromStr;

    fn create_test_node(
        name: &str,
        addon: &str,
        app: Option<&str>,
    ) -> GraphNode {
        GraphNode {
            type_and_name: PkgTypeAndName {
                pkg_type: PkgType::Extension,
                name: name.to_string(),
            },
            addon: addon.to_string(),
            extension_group: None,
            app: app.map(|s| s.to_string()),
            property: None,
        }
    }

    fn create_test_pkg_info_map() -> HashMap<String, Vec<PkgInfo>> {
        let mut map = HashMap::new();

        // Create app PkgInfo.
        let app_manifest_str = r#"
        {
            "type": "app",
            "name": "app1",
            "version": "1.0.0"
        }
        "#;
        let app_manifest = Manifest::from_str(app_manifest_str).unwrap();

        // Create property with URI for the app.
        let prop_str = r#"
        {
            "_ten": {
                "uri": "app1"
            }
        }
        "#;
        let app_property = Property::from_str(prop_str).unwrap();

        // Create ext1 PkgInfo with valid API schema for message communication
        let ext1_manifest_str = r#"
        {
            "type": "extension",
            "name": "ext1",
            "version": "1.0.0",
            "api": {
                "property": {
                    "app_id": {
                        "type": "string"
                    },
                    "app_version": {
                        "type": "uint8"
                    }
                },
                "cmd_out": [
                    {
                        "name": "cmd1",
                        "property": {
                            "param1": {
                                "type": "int8"
                            }
                        },
                        "result": {
                            "property": {
                                "detail": {
                                    "type": "bool"
                                }
                            },
                            "required": [
                                "detail"
                            ]
                        }
                    },
                    {
                        "name": "cmd_incompatible",
                        "property": {
                            "param1": {
                                "type": "string"
                            }
                        }
                    }
                ],
                "data_out": [
                    {
                        "name": "data1",
                        "property": {
                            "text_data": {
                                "type": "buf"
                            }
                        }
                    },
                    {
                        "name": "data_incompatible",
                        "property": {
                            "value": {
                                "type": "float32"
                            }
                        }
                    }
                ],
                "video_frame_out": [
                    {
                        "name": "video1",
                        "property": {
                            "width": {
                                "type": "uint64"
                            }
                        }
                    }
                ],
                "audio_frame_out": [
                    {
                        "name": "audio1",
                        "property": {
                            "format": {
                                "type": "uint8"
                            }
                        }
                    }
                ]
            }
        }
        "#;
        let ext1_manifest = Manifest::from_str(ext1_manifest_str).unwrap();

        // Create ext2 PkgInfo with compatible API schemas
        let ext2_manifest_str = r#"
        {
            "type": "extension",
            "name": "ext2",
            "version": "1.0.0",
            "api": {
                "cmd_in": [
                    {
                        "name": "cmd1",
                        "property": {
                            "param1": {
                                "type": "int8"
                            }
                        },
                        "result": {
                            "property": {
                                "detail": {
                                    "type": "bool"
                                }
                            },
                            "required": [
                                "detail"
                            ]
                        }
                    }
                ],
                "data_in": [
                    {
                        "name": "data1",
                        "property": {
                            "text_data": {
                                "type": "buf"
                            }
                        }
                    }
                ],
                "video_frame_in": [
                    {
                        "name": "video1",
                        "property": {
                            "width": {
                                "type": "uint64"
                            }
                        }
                    }
                ],
                "audio_frame_in": [
                    {
                        "name": "audio1",
                        "property": {
                            "format": {
                                "type": "uint8"
                            }
                        }
                    }
                ]
            }
        }
        "#;
        let ext2_manifest = Manifest::from_str(ext2_manifest_str).unwrap();

        // Create ext3 PkgInfo with incompatible API schemas
        let ext3_manifest_str = r#"
        {
            "type": "extension",
            "name": "ext3",
            "version": "1.0.0",
            "api": {
                "cmd_in": [
                    {
                        "name": "cmd_incompatible",
                        "property": {
                            "param1": {
                                "type": "int32"
                            }
                        }
                    },
                    {
                        "name": "cmd1",
                        "property": {
                            "param1": {
                                "type": "int8"
                            }
                        },
                        "result": {
                            "property": {
                                "detail": {
                                    "type": "bool"
                                }
                            },
                            "required": [
                                "detail"
                            ]
                        }
                    }
                ],
                "data_in": [
                    {
                        "name": "data_incompatible",
                        "property": {
                            "value": {
                                "type": "int64"
                            }
                        }
                    },
                    {
                        "name": "data1",
                        "property": {
                            "text_data": {
                                "type": "buf"
                            }
                        }
                    }
                ]
            }
        }
        "#;
        let ext3_manifest = Manifest::from_str(ext3_manifest_str).unwrap();

        // Create app PkgInfo.
        let app_pkg_info = PkgInfo {
            manifest: Some(app_manifest),
            property: Some(app_property),
            compatible_score: 0,
            is_installed: true,
            url: String::new(),
            hash: String::new(),
            schema_store: None,
            is_local_dependency: false,
            local_dependency_path: None,
            local_dependency_base_dir: None,
        };

        // Create schema stores for extensions.
        let ext1_schema_store =
            SchemaStore::from_manifest(&ext1_manifest).unwrap().unwrap();
        let ext2_schema_store =
            SchemaStore::from_manifest(&ext2_manifest).unwrap().unwrap();
        let ext3_schema_store =
            SchemaStore::from_manifest(&ext3_manifest).unwrap().unwrap();

        // Create extension PkgInfos.
        let ext1_pkg_info = PkgInfo {
            manifest: Some(ext1_manifest),
            property: None,
            compatible_score: 0,
            is_installed: true,
            url: String::new(),
            hash: String::new(),
            schema_store: Some(ext1_schema_store),
            is_local_dependency: false,
            local_dependency_path: None,
            local_dependency_base_dir: None,
        };

        let ext2_pkg_info = PkgInfo {
            manifest: Some(ext2_manifest),
            property: None,
            compatible_score: 0,
            is_installed: true,
            url: String::new(),
            hash: String::new(),
            schema_store: Some(ext2_schema_store),
            is_local_dependency: false,
            local_dependency_path: None,
            local_dependency_base_dir: None,
        };

        let ext3_pkg_info = PkgInfo {
            manifest: Some(ext3_manifest),
            property: None,
            compatible_score: 0,
            is_installed: true,
            url: String::new(),
            hash: String::new(),
            schema_store: Some(ext3_schema_store),
            is_local_dependency: false,
            local_dependency_path: None,
            local_dependency_base_dir: None,
        };

        // Add to map.
        map.insert(
            "/path/to/app1".to_string(),
            vec![app_pkg_info, ext1_pkg_info, ext2_pkg_info, ext3_pkg_info],
        );

        map
    }

    #[test]
    fn test_add_connection() {
        // Create a graph with two nodes.
        let mut graph = Graph {
            nodes: vec![
                create_test_node("ext1", "addon1", Some("app1")),
                create_test_node("ext2", "addon2", Some("app1")),
            ],
            connections: None,
        };

        // Test adding a connection.
        let result = graph.add_connection(
            Some("app1".to_string()),
            "ext1".to_string(),
            MsgType::Cmd,
            "test_cmd".to_string(),
            Some("app1".to_string()),
            "ext2".to_string(),
            &create_test_pkg_info_map(),
        );

        assert!(result.is_ok());
        assert!(graph.connections.is_some());

        let connections = graph.connections.as_ref().unwrap();
        assert_eq!(connections.len(), 1);

        let connection = &connections[0];
        assert_eq!(connection.app, Some("app1".to_string()));
        assert_eq!(connection.extension, "ext1");

        let cmd_flows = connection.cmd.as_ref().unwrap();
        assert_eq!(cmd_flows.len(), 1);

        let flow = &cmd_flows[0];
        assert_eq!(flow.name, "test_cmd");
        assert_eq!(flow.dest.len(), 1);

        let dest = &flow.dest[0];
        assert_eq!(dest.app, Some("app1".to_string()));
        assert_eq!(dest.extension, "ext2");
    }

    #[test]
    fn test_add_connection_nonexistent_source() {
        // Create a graph with only one node.
        let mut graph = Graph {
            nodes: vec![create_test_node("ext2", "addon2", Some("app1"))],
            connections: None,
        };

        // Test adding a connection with nonexistent source.
        let result = graph.add_connection(
            Some("app1".to_string()),
            "ext1".to_string(), // This node doesn't exist.
            MsgType::Cmd,
            "test_cmd".to_string(),
            Some("app1".to_string()),
            "ext2".to_string(),
            &create_test_pkg_info_map(),
        );

        assert!(result.is_err());
        assert!(graph.connections.is_none()); // Graph should remain unchanged.
    }

    #[test]
    fn test_add_connection_nonexistent_destination() {
        // Create a graph with only one node.
        let mut graph = Graph {
            nodes: vec![create_test_node("ext1", "addon1", Some("app1"))],
            connections: None,
        };

        // Test adding a connection with nonexistent destination.
        let result = graph.add_connection(
            Some("app1".to_string()),
            "ext1".to_string(),
            MsgType::Cmd,
            "test_cmd".to_string(),
            Some("app1".to_string()),
            "ext2".to_string(), // This node doesn't exist.
            &create_test_pkg_info_map(),
        );

        assert!(result.is_err());
        assert!(graph.connections.is_none()); // Graph should remain unchanged.
    }

    #[test]
    fn test_add_connection_to_existing_flow() {
        // Create a graph with three nodes.
        let mut graph = Graph {
            nodes: vec![
                create_test_node("ext1", "addon1", Some("app1")),
                create_test_node("ext2", "addon2", Some("app1")),
                create_test_node("ext3", "addon3", Some("app1")),
            ],
            connections: None,
        };

        let pkg_info_map = create_test_pkg_info_map();

        // Add first connection.
        let result = graph.add_connection(
            Some("app1".to_string()),
            "ext1".to_string(),
            MsgType::Cmd,
            "test_cmd".to_string(),
            Some("app1".to_string()),
            "ext2".to_string(),
            &pkg_info_map,
        );
        assert!(result.is_ok());

        // Add second connection with same source and message name but different
        // destination.
        let result = graph.add_connection(
            Some("app1".to_string()),
            "ext1".to_string(),
            MsgType::Cmd,
            "test_cmd".to_string(),
            Some("app1".to_string()),
            "ext3".to_string(),
            &pkg_info_map,
        );
        assert!(result.is_ok());

        // Verify that we have one connection with one message flow that has two
        // destinations.
        let connections = graph.connections.as_ref().unwrap();
        assert_eq!(connections.len(), 1);

        let connection = &connections[0];
        let cmd_flows = connection.cmd.as_ref().unwrap();
        assert_eq!(cmd_flows.len(), 1);

        let flow = &cmd_flows[0];
        assert_eq!(flow.name, "test_cmd");
        assert_eq!(flow.dest.len(), 2);

        // Verify destinations.
        assert_eq!(flow.dest[0].extension, "ext2");
        assert_eq!(flow.dest[1].extension, "ext3");
    }

    #[test]
    fn test_add_different_message_types() {
        // Create a graph with two nodes.
        let mut graph = Graph {
            nodes: vec![
                create_test_node("ext1", "addon1", Some("app1")),
                create_test_node("ext2", "addon2", Some("app1")),
            ],
            connections: None,
        };

        let pkg_info_map = create_test_pkg_info_map();

        // Add different message types.
        let result = graph.add_connection(
            Some("app1".to_string()),
            "ext1".to_string(),
            MsgType::Cmd,
            "cmd1".to_string(),
            Some("app1".to_string()),
            "ext2".to_string(),
            &pkg_info_map,
        );
        assert!(result.is_ok());

        let result = graph.add_connection(
            Some("app1".to_string()),
            "ext1".to_string(),
            MsgType::Data,
            "data1".to_string(),
            Some("app1".to_string()),
            "ext2".to_string(),
            &pkg_info_map,
        );
        assert!(result.is_ok());

        let result = graph.add_connection(
            Some("app1".to_string()),
            "ext1".to_string(),
            MsgType::AudioFrame,
            "audio1".to_string(),
            Some("app1".to_string()),
            "ext2".to_string(),
            &pkg_info_map,
        );
        assert!(result.is_ok());

        let result = graph.add_connection(
            Some("app1".to_string()),
            "ext1".to_string(),
            MsgType::VideoFrame,
            "video1".to_string(),
            Some("app1".to_string()),
            "ext2".to_string(),
            &pkg_info_map,
        );
        assert!(result.is_ok());

        // Verify that we have one connection with different message flows.
        let connection = &graph.connections.as_ref().unwrap()[0];

        assert!(connection.cmd.is_some());
        assert!(connection.data.is_some());
        assert!(connection.audio_frame.is_some());
        assert!(connection.video_frame.is_some());

        assert_eq!(connection.cmd.as_ref().unwrap()[0].name, "cmd1");
        assert_eq!(connection.data.as_ref().unwrap()[0].name, "data1");
        assert_eq!(connection.audio_frame.as_ref().unwrap()[0].name, "audio1");
        assert_eq!(connection.video_frame.as_ref().unwrap()[0].name, "video1");
    }

    #[test]
    fn test_add_duplicate_connection() {
        // Create a graph with two nodes.
        let mut graph = Graph {
            nodes: vec![
                create_test_node("ext1", "addon1", Some("app1")),
                create_test_node("ext2", "addon2", Some("app1")),
            ],
            connections: None,
        };

        let pkg_info_map = create_test_pkg_info_map();

        // Add a connection.
        let result = graph.add_connection(
            Some("app1".to_string()),
            "ext1".to_string(),
            MsgType::Cmd,
            "test_cmd".to_string(),
            Some("app1".to_string()),
            "ext2".to_string(),
            &pkg_info_map,
        );
        assert!(result.is_ok());

        // Try to add the same connection again.
        let result = graph.add_connection(
            Some("app1".to_string()),
            "ext1".to_string(),
            MsgType::Cmd,
            "test_cmd".to_string(),
            Some("app1".to_string()),
            "ext2".to_string(),
            &pkg_info_map,
        );

        // This should fail because the connection already exists.
        assert!(result.is_err());

        // The error message should indicate that the connection already exists.
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Connection already exists"));

        // Verify that the graph wasn't changed by the second add attempt.
        let connections = graph.connections.as_ref().unwrap();
        assert_eq!(connections.len(), 1);

        let connection = &connections[0];
        let cmd_flows = connection.cmd.as_ref().unwrap();
        assert_eq!(cmd_flows.len(), 1);

        let flow = &cmd_flows[0];
        assert_eq!(flow.dest.len(), 1);
    }

    #[test]
    fn test_schema_compatibility_check() {
        // Create a graph with three nodes.
        let mut graph = Graph {
            nodes: vec![
                create_test_node("ext1", "addon1", Some("app1")),
                create_test_node("ext2", "addon2", Some("app1")),
                create_test_node("ext3", "addon3", Some("app1")),
            ],
            connections: None,
        };

        let pkg_info_map = create_test_pkg_info_map();

        // Test connecting ext1 to ext2 with compatible schema - should succeed.
        let result = graph.add_connection(
            Some("app1".to_string()),
            "ext1".to_string(),
            MsgType::Cmd,
            "cmd1".to_string(),
            Some("app1".to_string()),
            "ext2".to_string(),
            &pkg_info_map,
        );
        assert!(result.is_ok());

        // Test connecting ext1 to ext3 with compatible schema - should succeed.
        let result = graph.add_connection(
            Some("app1".to_string()),
            "ext1".to_string(),
            MsgType::Data,
            "data1".to_string(),
            Some("app1".to_string()),
            "ext3".to_string(),
            &pkg_info_map,
        );
        assert!(result.is_ok());

        // Test connecting ext1 to ext3 with incompatible schema - should fail.
        let result = graph.add_connection(
            Some("app1".to_string()),
            "ext1".to_string(),
            MsgType::Cmd,
            "cmd_incompatible".to_string(),
            Some("app1".to_string()),
            "ext3".to_string(),
            &pkg_info_map,
        );
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("schema incompatibility"));

        // Test connecting ext1 to ext3 with incompatible schema for data -
        // should fail.
        let result = graph.add_connection(
            Some("app1".to_string()),
            "ext1".to_string(),
            MsgType::Data,
            "data_incompatible".to_string(),
            Some("app1".to_string()),
            "ext3".to_string(),
            &pkg_info_map,
        );
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("schema incompatibility"));
    }
}
