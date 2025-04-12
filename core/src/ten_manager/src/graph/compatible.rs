//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::collections::HashMap;

use anyhow::Result;

use ten_rust::{
    base_dir_pkg_info::{PkgsInfoInApp, PkgsInfoInAppWithBaseDir},
    graph::node::GraphNode,
    pkg_info::{
        message::{MsgDirection, MsgType},
        pkg_type::PkgType,
        PkgInfo,
    },
    schema::{
        runtime_interface::TenSchema,
        store::{
            are_cmd_schemas_compatible, are_ten_schemas_compatible, CmdSchema,
        },
    },
};

pub struct CompatibleExtensionAndMsg<'a> {
    pub extension: &'a GraphNode,
    pub msg_type: MsgType,
    pub msg_direction: MsgDirection,
    pub msg_name: String,
}

pub fn get_pkg_info_for_extension_graph_node<'a>(
    app: &Option<String>,
    extension_addon: &String,
    uri_to_pkg_info: &'a HashMap<Option<String>, PkgsInfoInAppWithBaseDir>,
    app_base_dir: Option<&String>,
    pkgs_cache: &'a HashMap<String, PkgsInfoInApp>,
) -> Option<&'a PkgInfo> {
    let result =
        uri_to_pkg_info
            .get(app)
            .and_then(|pkgs_info_in_app_with_base_dir| {
                pkgs_info_in_app_with_base_dir
                    .pkgs_info_in_app
                    .get_extensions()
                    .iter()
                    .find(|pkg_info| {
                        pkg_info.manifest.type_and_name.pkg_type
                            == PkgType::Extension
                            && pkg_info.manifest.type_and_name.name
                                == *extension_addon
                    })
            });

    if let Some(pkg_info) = result {
        Some(pkg_info)
    } else if let Some(app_base_dir) = app_base_dir {
        pkgs_cache.get(app_base_dir).and_then(|pkgs_info_in_app| {
            pkgs_info_in_app.get_extensions().iter().find(|pkg_info| {
                pkg_info.manifest.type_and_name.pkg_type == PkgType::Extension
                    && pkg_info.manifest.type_and_name.name == *extension_addon
            })
        })
    } else {
        None
    }
}

pub fn get_compatible_cmd_extension<'a>(
    extension_graph_nodes: &'a [GraphNode],
    uri_to_pkg_info: &'a HashMap<Option<String>, PkgsInfoInAppWithBaseDir>,
    app_base_dir: Option<&String>,
    pkgs_cache: &'a HashMap<String, PkgsInfoInApp>,
    desired_msg_dir: &MsgDirection,
    pivot: Option<&CmdSchema>,
    cmd_name: &str,
) -> Result<Vec<CompatibleExtensionAndMsg<'a>>> {
    let mut result = Vec::new();

    for target_extension_graph_node in extension_graph_nodes {
        let target_extension_pkg_info = get_pkg_info_for_extension_graph_node(
            &target_extension_graph_node.app,
            &target_extension_graph_node.addon,
            uri_to_pkg_info,
            app_base_dir,
            pkgs_cache,
        );

        match target_extension_pkg_info {
            Some(target_extension_pkg_info) => {
                let target_cmd_schema = target_extension_pkg_info
                    .schema_store
                    .as_ref()
                    .and_then(|schema_store| match desired_msg_dir {
                        MsgDirection::In => schema_store.cmd_in.get(cmd_name),
                        MsgDirection::Out => schema_store.cmd_out.get(cmd_name),
                    });

                let compatible = match desired_msg_dir {
                    MsgDirection::In => are_cmd_schemas_compatible(
                        pivot,
                        target_cmd_schema,
                        false,
                    ),
                    MsgDirection::Out => are_cmd_schemas_compatible(
                        target_cmd_schema,
                        pivot,
                        false,
                    ),
                };

                if compatible.is_ok() {
                    result.push(CompatibleExtensionAndMsg {
                        extension: target_extension_graph_node,
                        msg_type: MsgType::Cmd,
                        msg_name: cmd_name.to_string(),
                        msg_direction: desired_msg_dir.clone(),
                    });
                }
            }
            None => continue,
        };
    }

    Ok(result)
}

#[allow(clippy::too_many_arguments)]
pub fn get_compatible_data_like_msg_extension<'a>(
    extension_graph_nodes: &'a [GraphNode],
    uri_to_pkg_info: &'a HashMap<Option<String>, PkgsInfoInAppWithBaseDir>,
    app_base_dir: Option<&String>,
    pkgs_cache: &'a HashMap<String, PkgsInfoInApp>,
    desired_msg_dir: &MsgDirection,
    pivot: Option<&TenSchema>,
    msg_type: &MsgType,
    msg_name: String,
) -> Result<Vec<CompatibleExtensionAndMsg<'a>>> {
    let mut result = Vec::new();

    for target_extension_graph_node in extension_graph_nodes {
        let target_extension_pkg_info = get_pkg_info_for_extension_graph_node(
            &target_extension_graph_node.app,
            &target_extension_graph_node.addon,
            uri_to_pkg_info,
            app_base_dir,
            pkgs_cache,
        );

        match target_extension_pkg_info {
            Some(target_extension_pkg_info) => {
                let target_msg_schema = target_extension_pkg_info
                    .schema_store
                    .as_ref()
                    .and_then(|schema_store| {
                        let msg_name = msg_name.as_str();
                        match msg_type {
                            MsgType::Data => match desired_msg_dir {
                                MsgDirection::In => {
                                    schema_store.data_in.get(msg_name)
                                }
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
                    MsgDirection::In => are_ten_schemas_compatible(
                        pivot,
                        target_msg_schema,
                        false,
                    ),
                    MsgDirection::Out => are_ten_schemas_compatible(
                        target_msg_schema,
                        pivot,
                        false,
                    ),
                };

                if compatible.is_ok() {
                    result.push(CompatibleExtensionAndMsg {
                        extension: target_extension_graph_node,
                        msg_type: msg_type.clone(),
                        msg_name: msg_name.to_string(),
                        msg_direction: desired_msg_dir.clone(),
                    });
                }
            }
            None => continue,
        }
    }

    Ok(result)
}
