//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::collections::HashMap;

use anyhow::Result;

use crate::{
    base_dir_pkg_info::{PkgsInfoInApp, PkgsInfoInAppWithBaseDir},
    graph::{
        connection::{GraphConnection, GraphDestination},
        Graph,
    },
    pkg_info::{
        get_pkg_info_for_extension_addon,
        message::{MsgDirection, MsgType},
        pkg_type::PkgType,
    },
    schema::store::{
        are_msg_schemas_compatible, find_msg_schema_from_all_pkgs_info,
        TenMsgSchema,
    },
};

impl Graph {
    fn get_addon_name_of_extension(
        &self,
        app: Option<&str>,
        extension: &str,
    ) -> &str {
        self.nodes
            .iter()
            .find_map(|node| {
                if node.type_and_name.pkg_type == PkgType::Extension
                    && node.type_and_name.name.as_str() == extension
                    && node.get_app_uri() == app
                {
                    Some(node.addon.as_str())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| {
                panic!(
                    "Extension '{}' is not found in nodes, should not happen.",
                    extension
                )
            })
    }

    #[allow(clippy::too_many_arguments)]
    fn check_msg_flow_compatible(
        &self,
        graph_app_base_dir: &str,
        pkgs_cache: &HashMap<String, PkgsInfoInApp>,
        uri_to_pkg_info: &HashMap<Option<String>, PkgsInfoInAppWithBaseDir>,
        msg_name: &str,
        msg_type: &MsgType,
        src_msg_schema: Option<&TenMsgSchema>,
        dests: &[GraphDestination],
        ignore_missing_apps: bool,
    ) -> Result<()> {
        let mut errors: Vec<String> = Vec::new();

        for dest in dests {
            let dest_addon = self.get_addon_name_of_extension(
                dest.get_app_uri(),
                dest.extension.as_str(),
            );

            let extension_pkg_info = match get_pkg_info_for_extension_addon(
                &dest.app,
                &dest_addon.to_string(),
                uri_to_pkg_info,
                Some(&graph_app_base_dir.to_string()),
                pkgs_cache,
            ) {
                Some(pkg_info) => pkg_info,
                None if ignore_missing_apps => continue,
                None => {
                    return Err(anyhow::anyhow!(
                        "Extension addon [{}] is not found in the pkgs map, should not happen.",
                        dest_addon
                    ))
                }
            };

            let dest_msg_schema = find_msg_schema_from_all_pkgs_info(
                extension_pkg_info,
                msg_type,
                msg_name,
                MsgDirection::In,
            );

            if let Err(e) = are_msg_schemas_compatible(
                src_msg_schema,
                dest_msg_schema,
                true,
            ) {
                errors.push(format!(
                    "Schema incompatible to [extension: {}], {}",
                    dest.extension, e
                ));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(anyhow::anyhow!("{}", errors.join("\n")))
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn check_cmd_flow_compatible(
        &self,
        graph_app_base_dir: &str,
        pkgs_cache: &HashMap<String, PkgsInfoInApp>,
        uri_to_pkg_info: &HashMap<Option<String>, PkgsInfoInAppWithBaseDir>,
        cmd_name: &str,
        src_cmd_schema: Option<&TenMsgSchema>,
        dests: &[GraphDestination],
        ignore_missing_apps: bool,
    ) -> Result<()> {
        let mut errors: Vec<String> = Vec::new();

        for dest in dests {
            let dest_addon = self.get_addon_name_of_extension(
                dest.get_app_uri(),
                dest.extension.as_str(),
            );

            let extension_pkg_info = match get_pkg_info_for_extension_addon(
              &dest.app,
              &dest_addon.to_string(),
              uri_to_pkg_info,
              Some(&graph_app_base_dir.to_string()),
              pkgs_cache,
          ) {
              Some(pkg_info) => pkg_info,
              None if ignore_missing_apps => continue,
              None => {
                  return Err(anyhow::anyhow!(
                      "Extension addon [{}] is not found in the pkgs map, should not happen.",
                      dest_addon
                  ))
              }
          };

            let dest_cmd_schema = find_msg_schema_from_all_pkgs_info(
                extension_pkg_info,
                &MsgType::Cmd,
                cmd_name,
                MsgDirection::In,
            );

            if let Err(e) = are_msg_schemas_compatible(
                src_cmd_schema,
                dest_cmd_schema,
                true,
            ) {
                errors.push(format!(
                    "Schema incompatible to [extension: {}], {}",
                    dest.extension, e
                ));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(anyhow::anyhow!("{}", errors.join("\n")))
        }
    }

    /// Check the compatibility of a single connection with all its flow types.
    ///
    /// # Parameters
    /// * `installed_pkgs_of_all_apps` - Map of all packages across all apps
    /// * `connection` - The connection to validate
    /// * `ignore_missing_apps` - Whether to skip validation when an app is
    ///   missing
    ///
    /// # Returns
    /// * `Ok(())` if the connection is compatible
    /// * `Err` with compatibility error details otherwise
    fn check_connection_compatibility(
        &self,
        graph_app_base_dir: &str,
        pkgs_cache: &HashMap<String, PkgsInfoInApp>,
        uri_to_pkg_info: &HashMap<Option<String>, PkgsInfoInAppWithBaseDir>,
        connection: &GraphConnection,
        ignore_missing_apps: bool,
    ) -> Result<()> {
        let mut errors: Vec<String> = Vec::new();

        let src_app_uri = connection.get_app_uri();
        let src_addon = self.get_addon_name_of_extension(
            src_app_uri,
            connection.extension.as_str(),
        );

        let extension_pkg_info = match get_pkg_info_for_extension_addon(
          &src_app_uri.as_ref().map(|s| s.to_string()),
          &src_addon.to_string(),
          uri_to_pkg_info,
          Some(&graph_app_base_dir.to_string()),
          pkgs_cache,
      ) {
          Some(pkg_info) => pkg_info,
          None if ignore_missing_apps => return Ok(()),
          None => {
              return Err(anyhow::anyhow!(
                  "Extension addon [{}] is not found in the pkgs map, should not happen.",
                  src_addon
              ))
          }
      };

        // Check command flows.
        if let Some(cmd_flows) = &connection.cmd {
            for (flow_idx, flow) in cmd_flows.iter().enumerate() {
                // Get source command schema.
                let src_cmd_schema = find_msg_schema_from_all_pkgs_info(
                    extension_pkg_info,
                    &MsgType::Cmd,
                    flow.name.as_str(),
                    MsgDirection::Out,
                );

                // Check command flow compatibility.
                if let Err(e) = self.check_cmd_flow_compatible(
                    graph_app_base_dir,
                    pkgs_cache,
                    uri_to_pkg_info,
                    flow.name.as_str(),
                    src_cmd_schema,
                    &flow.dest,
                    ignore_missing_apps,
                ) {
                    errors.push(format!("- cmd[{}]:  {}", flow_idx, e));
                }
            }
        }

        // Check data flows.
        if let Some(data_flows) = &connection.data {
            for (flow_idx, flow) in data_flows.iter().enumerate() {
                // Get source message schema.
                let src_msg_schema = find_msg_schema_from_all_pkgs_info(
                    extension_pkg_info,
                    &MsgType::Data,
                    flow.name.as_str(),
                    MsgDirection::Out,
                );

                // Check message flow compatibility.
                if let Err(e) = self.check_msg_flow_compatible(
                    graph_app_base_dir,
                    pkgs_cache,
                    uri_to_pkg_info,
                    flow.name.as_str(),
                    &MsgType::Data,
                    src_msg_schema,
                    &flow.dest,
                    ignore_missing_apps,
                ) {
                    errors.push(format!("- data[{}]:  {}", flow_idx, e));
                }
            }
        }

        // Check video frame flows.
        if let Some(video_frame_flows) = &connection.video_frame {
            for (flow_idx, flow) in video_frame_flows.iter().enumerate() {
                // Get source message schema.
                let src_msg_schema = find_msg_schema_from_all_pkgs_info(
                    extension_pkg_info,
                    &MsgType::VideoFrame,
                    flow.name.as_str(),
                    MsgDirection::Out,
                );

                // Check message flow compatibility.
                if let Err(e) = self.check_msg_flow_compatible(
                    graph_app_base_dir,
                    pkgs_cache,
                    uri_to_pkg_info,
                    flow.name.as_str(),
                    &MsgType::VideoFrame,
                    src_msg_schema,
                    &flow.dest,
                    ignore_missing_apps,
                ) {
                    errors.push(format!("- video_frame[{}]:  {}", flow_idx, e));
                }
            }
        }

        // Check audio frame flows.
        if let Some(audio_frame_flows) = &connection.audio_frame {
            for (flow_idx, flow) in audio_frame_flows.iter().enumerate() {
                // Get source message schema.
                let src_msg_schema = find_msg_schema_from_all_pkgs_info(
                    extension_pkg_info,
                    &MsgType::AudioFrame,
                    flow.name.as_str(),
                    MsgDirection::Out,
                );

                // Check message flow compatibility.
                if let Err(e) = self.check_msg_flow_compatible(
                    graph_app_base_dir,
                    pkgs_cache,
                    uri_to_pkg_info,
                    flow.name.as_str(),
                    &MsgType::AudioFrame,
                    src_msg_schema,
                    &flow.dest,
                    ignore_missing_apps,
                ) {
                    errors.push(format!("- audio_frame[{}]:  {}", flow_idx, e));
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(anyhow::anyhow!("{}", errors.join("\n  ")))
        }
    }

    /// Check the compatibility of all connections in the graph.
    ///
    /// # Parameters
    /// * `installed_pkgs_of_all_apps` - Map of all packages across all apps
    /// * `ignore_missing_apps` - Whether to skip validation when an app is
    ///   missing
    ///
    /// # Returns
    /// * `Ok(())` if all connections are compatible
    /// * `Err` with detailed compatibility error messages otherwise
    pub fn check_connections_compatibility(
        &self,
        graph_app_base_dir: &str,
        pkgs_cache: &HashMap<String, PkgsInfoInApp>,
        uri_to_pkg_info: &HashMap<Option<String>, PkgsInfoInAppWithBaseDir>,
        ignore_missing_apps: bool,
    ) -> Result<()> {
        if self.connections.is_none() {
            return Ok(());
        }

        let mut errors: Vec<String> = Vec::new();

        // Check compatibility of all connections.
        let connections = self.connections.as_ref().unwrap();
        for (conn_idx, connection) in connections.iter().enumerate() {
            if let Err(e) = self.check_connection_compatibility(
                graph_app_base_dir,
                pkgs_cache,
                uri_to_pkg_info,
                connection,
                ignore_missing_apps,
            ) {
                errors.push(format!("- connections[{}]: \n  {}", conn_idx, e));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(anyhow::anyhow!("{}", errors.join("\n")))
        }
    }
}
