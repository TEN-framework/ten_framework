//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::collections::HashMap;

use anyhow::Result;

use ten_rust::{
    base_dir_pkg_info::PkgsInfoInAppWithBaseDir,
    graph::{
        connection::{GraphConnection, GraphDestination, GraphMessageFlow},
        msg_conversion::MsgAndResultConversion,
        Graph,
    },
    pkg_info::{message::MsgType, pkg_type::PkgType, PkgInfo},
    schema::store::are_msg_schemas_compatible,
};

type PkgInfoTuple<'a> = (Option<&'a PkgInfo>, Option<&'a PkgInfo>);

/// Finds package info for source and destination apps and extensions.
fn find_pkg_infos<'a>(
    installed_pkgs_of_all_apps: &'a HashMap<
        Option<String>,
        PkgsInfoInAppWithBaseDir,
    >,
    src_app: &Option<String>,
    src_extension: &str,
    dest_app: &Option<String>,
    dest_extension: &str,
) -> Result<PkgInfoTuple<'a>> {
    // Helper function to find extension package info.
    let find_extension_pkg = |app_uri: &Option<String>,
                              extension_name: &str,
                              is_source: bool|
     -> Result<Option<&'a PkgInfo>> {
        let entity_type = if is_source { "Source" } else { "Destination" };

        if let Some(base_dir_pkg_info) = installed_pkgs_of_all_apps.get(app_uri)
        {
            // Check if app exists.
            if base_dir_pkg_info.pkgs_info_in_app.app_pkg_info.is_none() {
                return Err(anyhow::anyhow!(
                    "{} app '{:?}' found in map but app_pkg_info is None",
                    entity_type,
                    app_uri
                ));
            }

            // Find extension in extension_pkg_info.
            if let Some(extensions) =
                &base_dir_pkg_info.pkgs_info_in_app.extension_pkgs_info
            {
                let found_pkg = extensions.iter().find(|pkg| {
                    pkg.manifest.type_and_name.pkg_type == PkgType::Extension
                        && pkg.manifest.type_and_name.name == extension_name
                });

                if found_pkg.is_none() {
                    return Err(anyhow::anyhow!(
                              "{} extension '{}' not found in the installed packages for app '{:?}'",
                              entity_type, extension_name, app_uri
                          ));
                }

                return Ok(found_pkg);
            }
        } else {
            return Err(anyhow::anyhow!(
                "{} app '{:?}' not found in the installed packages",
                entity_type,
                app_uri
            ));
        }

        // If we reach here, no package was found.
        Err(anyhow::anyhow!(
              "{} extension '{}' not found in the installed packages for app '{:?}'",
              entity_type, extension_name, app_uri
          ))
    };

    // Find both source and destination package info.
    let src_extension_pkg_info =
        find_extension_pkg(src_app, src_extension, true)?;
    let dest_extension_pkg_info =
        find_extension_pkg(dest_app, dest_extension, false)?;

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
                are_msg_schemas_compatible(src_schema, dest_schema, true)
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
                are_msg_schemas_compatible(src_schema, dest_schema, true)
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
                are_msg_schemas_compatible(src_schema, dest_schema, true)
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
                are_msg_schemas_compatible(src_schema, dest_schema, true)
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

/// Adds a message flow to a connection based on message type.
fn add_message_flow_to_connection(
    connection: &mut GraphConnection,
    msg_type: &MsgType,
    message_flow: GraphMessageFlow,
) -> Result<()> {
    // Add the message flow to the appropriate vector based on message type.
    match msg_type {
        MsgType::Cmd => add_to_flow(&mut connection.cmd, message_flow),
        MsgType::Data => add_to_flow(&mut connection.data, message_flow),
        MsgType::AudioFrame => {
            add_to_flow(&mut connection.audio_frame, message_flow)
        }
        MsgType::VideoFrame => {
            add_to_flow(&mut connection.video_frame, message_flow)
        }
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

/// Adds a new connection between two extension nodes in the graph.
#[allow(clippy::too_many_arguments)]
pub fn graph_add_connection(
    graph: &mut Graph,
    src_app: Option<String>,
    src_extension: String,
    msg_type: MsgType,
    msg_name: String,
    dest_app: Option<String>,
    dest_extension: String,
    installed_pkgs_of_all_apps: &HashMap<
        Option<String>,
        PkgsInfoInAppWithBaseDir,
    >,
    msg_conversion: Option<MsgAndResultConversion>,
) -> Result<()> {
    // Store the original state in case validation fails.
    let original_graph = graph.clone();

    // Check if nodes exist.
    check_nodes_exist(
        graph,
        &src_app,
        &src_extension,
        &dest_app,
        &dest_extension,
    )?;

    // Check if connection already exists.
    check_connection_exists(
        graph,
        &src_app,
        &src_extension,
        &msg_type,
        &msg_name,
        &dest_app,
        &dest_extension,
    )?;

    // Find source and destination package info.
    let (src_extension_pkg_info, dest_extension_pkg_info) = find_pkg_infos(
        installed_pkgs_of_all_apps,
        &src_app,
        &src_extension,
        &dest_app,
        &dest_extension,
    )?;

    // Check schema compatibility.
    check_schema_compatibility(
        &msg_type,
        &msg_name,
        &src_extension_pkg_info,
        &dest_extension_pkg_info,
    )?;

    // Create destination object.
    let destination = GraphDestination {
        app: dest_app,
        extension: dest_extension,
        msg_conversion,
    };

    // Initialize connections if None.
    if graph.connections.is_none() {
        graph.connections = Some(Vec::new());
    }

    // Create a message flow.
    let message_flow = GraphMessageFlow {
        name: msg_name,
        dest: vec![destination],
    };

    // Get or create a connection for the source node and add the message
    // flow.
    {
        let connections = graph.connections.as_mut().unwrap();

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
        add_message_flow_to_connection(connection, &msg_type, message_flow)?;
    }

    // Validate the updated graph.
    match graph.validate_and_complete() {
        Ok(_) => Ok(()),
        Err(e) => {
            // Restore the original graph if validation fails.
            *graph = original_graph;
            Err(e)
        }
    }
}
