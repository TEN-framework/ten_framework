//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::collections::HashMap;

use anyhow::Result;
use uuid::Uuid;

use crate::{
    graph::node::GraphNode,
    pkg_info::{
        message::{MsgDirection, MsgType},
        pkg_type::PkgType,
        predefined_graphs::pkg_predefined_graphs_find,
        PkgInfo,
    },
    schema::{
        store::{
            are_cmd_schemas_compatible, are_ten_schemas_compatible, CmdSchema,
        },
        TenSchema,
    },
};

use super::graph_info::GraphInfo;

/// Retrieves all extension nodes from a specified graph.
pub fn get_extension_nodes_in_graph(
    base_dir: &String,
    graph_name: &String,
    graphs_cache: &HashMap<Uuid, GraphInfo>,
) -> Result<Vec<GraphNode>> {
    // Look for the graph by name in the graphs_cache.
    if let Some(graph_info) =
        pkg_predefined_graphs_find(graphs_cache, |graph| {
            graph
                .name
                .as_ref()
                .map(|name| name == graph_name)
                .unwrap_or(false)
                && (graph.app_base_dir.is_some()
                    && graph.app_base_dir.as_ref().unwrap() == base_dir)
                && (graph.belonging_pkg_type.is_some()
                    && graph.belonging_pkg_type.unwrap() == PkgType::App)
        })
    {
        // Collect all extension nodes from the graph.
        let extension_nodes: Vec<_> = graph_info
            .graph
            .nodes
            .iter()
            .filter(|node| node.type_and_name.pkg_type == PkgType::Extension)
            .cloned()
            .collect();

        Ok(extension_nodes)
    } else {
        Err(anyhow::anyhow!(
            "Graph '{}' not found in predefined graphs of the app",
            graph_name
        ))
    }
}

/// Searches through `all_pkgs` to find a package that matches the extension
/// specified in the GraphNode.
pub fn get_pkg_info_for_extension<'a>(
    extension: &'a GraphNode,
    extension_pkgs_info: &'a [PkgInfo],
) -> Option<&'a PkgInfo> {
    extension_pkgs_info.iter().find(|pkg| {
        if let Some(manifest) = &pkg.manifest {
            manifest.type_and_name.pkg_type == PkgType::Extension
                && manifest.type_and_name.name == extension.addon
        } else {
            false
        }
    })
}

pub fn get_extension<'a>(
    extensions: &'a [GraphNode],
    app: &Option<String>,
    extension_group: &Option<String>,
    extension_name: &str,
) -> Result<&'a GraphNode> {
    for extension in extensions {
        if extension.type_and_name.name == extension_name
            && extension.extension_group.as_ref() == extension_group.as_ref()
            && extension.app == *app
        {
            return Ok(extension);
        }
    }

    Err(anyhow::anyhow!(
        "Failed to find extension '{}' in extension group '{:?}', app: '{:?}'",
        extension_name,
        extension_group,
        app
    ))
}

pub struct CompatibleExtensionAndMsg<'a> {
    pub extension: &'a GraphNode,
    pub msg_type: MsgType,
    pub msg_direction: MsgDirection,
    pub msg_name: String,
}

pub fn get_compatible_cmd_extension<'a>(
    extensions: &'a [GraphNode],
    extension_pkgs_info: &[PkgInfo],
    desired_msg_dir: &MsgDirection,
    pivot: Option<&CmdSchema>,
    cmd_name: &str,
) -> Result<Vec<CompatibleExtensionAndMsg<'a>>> {
    let mut result = Vec::new();

    for ext in extensions {
        let pkg_info = get_pkg_info_for_extension(ext, extension_pkgs_info)
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
    extension_pkgs_info: &'a [PkgInfo],
    desired_msg_dir: &MsgDirection,
    pivot: Option<&TenSchema>,
    msg_type: &MsgType,
    msg_name: String,
) -> Result<Vec<CompatibleExtensionAndMsg<'a>>> {
    let mut result = Vec::new();

    for ext in extensions {
        let pkg_info = get_pkg_info_for_extension(ext, extension_pkgs_info)
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
