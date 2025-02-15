//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use anyhow::Result;

use super::pkg_predefined_graphs_find;
use crate::{
    pkg_info::{
        graph::GraphNode,
        message::{MsgDirection, MsgType},
        pkg_type::PkgType,
        PkgInfo,
    },
    schema::{
        store::{
            are_cmd_schemas_compatible, are_ten_schemas_compatible, CmdSchema,
        },
        TenSchema,
    },
};

/// Retrieves all extension nodes from a specified graph.
///
/// # Arguments
/// - `graph_name`: The name of the graph to retrieve extension nodes from.
/// - `all_pkgs`: A slice of all package information.
///
/// # Returns
/// A vector of extension nodes from the specified graph.
pub fn get_extension_nodes_in_graph(
    graph_name: &String,
    all_pkgs: &[PkgInfo],
) -> Result<Vec<GraphNode>> {
    // Find the application package within the `all_pkgs`.
    if let Some(app_pkg) = all_pkgs
        .iter()
        .find(|pkg| pkg.basic_info.type_and_name.pkg_type == PkgType::App)
    {
        if app_pkg.property.is_none() {
            return Err(anyhow::anyhow!(
                "Property information is missing".to_string(),
            ));
        }

        // Look for the graph by name in the predefined_graphs of the app
        // package.
        if let Some(predefined_graph) = pkg_predefined_graphs_find(
            app_pkg.get_predefined_graphs(),
            |graph| graph.name == *graph_name,
        ) {
            // Collect all extension nodes from the graph.
            let extension_nodes: Vec<_> = predefined_graph
                .graph
                .nodes
                .iter()
                .filter(|node| {
                    node.type_and_name.pkg_type == PkgType::Extension
                })
                .cloned()
                .collect();

            Ok(extension_nodes)
        } else {
            Err(anyhow::anyhow!(
                "Graph '{}' not found in predefined graphs of the app",
                graph_name
            ))
        }
    } else {
        Err(anyhow::anyhow!("Package information is missing"))
    }
}

/// Searches through `all_pkgs` to find a package that matches the extension
/// specified in the GraphNode.
///
/// # Arguments
/// - `extension`: The extension node to search for.
/// - `all_pkgs`: A slice of all package information.
///
/// # Returns
/// A reference to the package information of the extension.
pub fn get_pkg_info_for_extension<'a>(
    extension: &'a GraphNode,
    all_pkgs: &'a [PkgInfo],
) -> Option<&'a PkgInfo> {
    all_pkgs.iter().find(|pkg| {
        pkg.basic_info.type_and_name.pkg_type == PkgType::Extension
            && pkg.basic_info.type_and_name.name == extension.addon
    })
}

pub fn get_extension<'a>(
    extensions: &'a [GraphNode],
    app: &String,
    extension_group: &String,
    extension: &String,
) -> Result<&'a GraphNode> {
    extensions
        .iter()
        .find(|ext| {
            ext.type_and_name.pkg_type == PkgType::Extension
                && ext.get_app_uri() == app
                && ext.extension_group.clone().unwrap_or("".to_string())
                    == *extension_group
                && &ext.type_and_name.name == extension
        })
        .ok_or_else(|| anyhow::anyhow!("Extension not found"))
}

pub struct CompatibleExtensionAndMsg<'a> {
    pub extension: &'a GraphNode,
    pub msg_type: MsgType,
    pub msg_direction: MsgDirection,
    pub msg_name: String,
}

pub fn get_compatible_cmd_extension<'a>(
    extensions: &'a [GraphNode],
    all_pkgs: &[PkgInfo],
    desired_msg_dir: &MsgDirection,
    pivot: Option<&CmdSchema>,
    cmd_name: &str,
) -> Result<Vec<CompatibleExtensionAndMsg<'a>>> {
    let mut result = Vec::new();

    for ext in extensions {
        let pkg_info = get_pkg_info_for_extension(ext, all_pkgs)
            .ok_or_else(|| anyhow::anyhow!("Extension not found"))?;

        let target_cmd_schema =
            pkg_info.schema_store.as_ref().and_then(|schema_store| {
                match desired_msg_dir {
                    MsgDirection::In => schema_store.cmd_in.get(cmd_name),
                    MsgDirection::Out => schema_store.cmd_out.get(cmd_name),
                }
            });

        let compatible = match desired_msg_dir {
            MsgDirection::In => {
                are_cmd_schemas_compatible(pivot, target_cmd_schema)
            }
            MsgDirection::Out => {
                are_cmd_schemas_compatible(target_cmd_schema, pivot)
            }
        };

        if compatible.is_ok() {
            result.push(CompatibleExtensionAndMsg {
                extension: ext,
                msg_type: MsgType::Cmd,
                msg_name: cmd_name.to_string(),
                msg_direction: desired_msg_dir.clone(),
            });
        }
    }

    Ok(result)
}

pub fn get_compatible_data_like_msg_extension<'a>(
    extensions: &'a [GraphNode],
    all_pkgs: &'a [PkgInfo],
    desired_msg_dir: &MsgDirection,
    pivot: Option<&TenSchema>,
    msg_type: &MsgType,
    msg_name: String,
) -> Result<Vec<CompatibleExtensionAndMsg<'a>>> {
    let mut result = Vec::new();

    for ext in extensions {
        let pkg_info = get_pkg_info_for_extension(ext, all_pkgs)
            .ok_or_else(|| anyhow::anyhow!("Extension not found"))?;

        let target_msg_schema =
            pkg_info.schema_store.as_ref().and_then(|schema_store| {
                let msg_name = msg_name.as_str();
                match msg_type {
                    MsgType::Data => match desired_msg_dir {
                        MsgDirection::In => schema_store.data_in.get(msg_name),
                        MsgDirection::Out => {
                            schema_store.data_out.get(msg_name)
                        }
                    },
                    MsgType::AudioFrame => match desired_msg_dir {
                        MsgDirection::In => {
                            schema_store.audio_frame_in.get(msg_name)
                        }
                        MsgDirection::Out => {
                            schema_store.audio_frame_out.get(msg_name)
                        }
                    },
                    MsgType::VideoFrame => match desired_msg_dir {
                        MsgDirection::In => {
                            schema_store.video_frame_in.get(msg_name)
                        }
                        MsgDirection::Out => {
                            schema_store.video_frame_out.get(msg_name)
                        }
                    },
                    _ => {
                        panic!("Unsupported message type: {}", msg_type)
                    }
                }
            });

        let compatible = match desired_msg_dir {
            MsgDirection::In => {
                are_ten_schemas_compatible(pivot, target_msg_schema)
            }
            MsgDirection::Out => {
                are_ten_schemas_compatible(target_msg_schema, pivot)
            }
        };

        if compatible.is_ok() {
            result.push(CompatibleExtensionAndMsg {
                extension: ext,
                msg_type: msg_type.clone(),
                msg_name: msg_name.to_string(),
                msg_direction: desired_msg_dir.clone(),
            });
        }
    }

    Ok(result)
}
