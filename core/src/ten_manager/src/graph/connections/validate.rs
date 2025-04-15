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
    graph::{msg_conversion::MsgAndResultConversion, Graph},
    pkg_info::{
        get_pkg_info_for_extension_addon,
        message::{MsgDirection, MsgType},
        pkg_type::PkgType,
        PkgInfo,
    },
    schema::store::{
        are_msg_schemas_compatible, create_msg_schema_from_manifest,
        find_msg_schema_from_all_pkgs_info,
    },
};

use crate::graph::msg_conversion::msg_conversion_get_final_target_schema;

type PkgInfoTuple<'a> = (Option<&'a PkgInfo>, Option<&'a PkgInfo>);

pub struct MsgConversionValidateInfo<'a> {
    pub src_app: &'a Option<String>,
    pub src_extension: &'a String,
    pub msg_type: &'a MsgType,
    pub msg_name: &'a String,
    pub dest_app: &'a Option<String>,
    pub dest_extension: &'a String,

    pub msg_conversion: &'a Option<MsgAndResultConversion>,
}

fn validate_msg_conversion_schema(
    graph: &mut Graph,
    graph_app_base_dir: &Option<String>,
    msg_conversion_validate_info: &MsgConversionValidateInfo,
    uri_to_pkg_info: &HashMap<Option<String>, PkgsInfoInAppWithBaseDir>,
    pkgs_cache: &HashMap<String, PkgsInfoInApp>,
) -> Result<()> {
    assert!(msg_conversion_validate_info.msg_conversion.is_some());

    let src_extension_addon = graph.get_addon_name_of_extension(
        msg_conversion_validate_info.src_app,
        msg_conversion_validate_info.src_extension,
    )?;

    let converted_schema = msg_conversion_get_final_target_schema(
        uri_to_pkg_info,
        graph_app_base_dir,
        pkgs_cache,
        msg_conversion_validate_info.src_app,
        src_extension_addon,
        msg_conversion_validate_info.msg_type,
        msg_conversion_validate_info.msg_name,
        msg_conversion_validate_info
            .msg_conversion
            .as_ref()
            .unwrap(),
    )?;

    eprintln!(
        "msg_conversion converted_schema: {}",
        serde_json::to_string_pretty(&converted_schema).unwrap()
    );

    if let Ok(Some(src_ten_msg_schema)) =
        create_msg_schema_from_manifest(&converted_schema)
    {
        let dest_extension_addon = graph.get_addon_name_of_extension(
            msg_conversion_validate_info.dest_app,
            msg_conversion_validate_info.dest_extension,
        )?;

        if let Some(dest_extension_pkg_info) = get_pkg_info_for_extension_addon(
            msg_conversion_validate_info.dest_app,
            dest_extension_addon,
            uri_to_pkg_info,
            graph_app_base_dir,
            pkgs_cache,
        ) {
            if let Some(dest_ten_msg_schema) =
                find_msg_schema_from_all_pkgs_info(
                    dest_extension_pkg_info,
                    msg_conversion_validate_info.msg_type,
                    &converted_schema.name,
                    MsgDirection::In,
                )
            {
                are_msg_schemas_compatible(
                    Some(&src_ten_msg_schema),
                    Some(dest_ten_msg_schema),
                    false,
                )?;
            }
        }
    }

    Ok(())
}

/// Finds package info for source and destination apps and extensions.
fn find_pkg_infos<'a>(
    uri_to_pkg_info: &'a HashMap<Option<String>, PkgsInfoInAppWithBaseDir>,
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

        if let Some(base_dir_pkg_info) = uri_to_pkg_info.get(app_uri) {
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
    msg_conversion_validate_info: &MsgConversionValidateInfo,
    uri_to_pkg_info: &HashMap<Option<String>, PkgsInfoInAppWithBaseDir>,
) -> Result<()> {
    let (src_extension_pkg_info, dest_extension_pkg_info) = find_pkg_infos(
        uri_to_pkg_info,
        &msg_conversion_validate_info.src_app,
        &msg_conversion_validate_info.src_extension,
        &msg_conversion_validate_info.dest_app,
        &msg_conversion_validate_info.dest_extension,
    )?;

    if src_extension_pkg_info.is_none() || dest_extension_pkg_info.is_none() {
        return Ok(());
    }

    let src_extension_pkg_info = src_extension_pkg_info.unwrap();
    let dest_extension_pkg_info = dest_extension_pkg_info.unwrap();

    match &msg_conversion_validate_info.msg_type {
        MsgType::Cmd => {
            let src_schema = src_extension_pkg_info
                .schema_store
                .as_ref()
                .and_then(|store| {
                    store.cmd_out.get(msg_conversion_validate_info.msg_name)
                });

            let dest_schema = dest_extension_pkg_info
                .schema_store
                .as_ref()
                .and_then(|store| {
                    store.cmd_in.get(msg_conversion_validate_info.msg_name)
                });

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
            let src_schema = src_extension_pkg_info
                .schema_store
                .as_ref()
                .and_then(|store| {
                    store.data_out.get(msg_conversion_validate_info.msg_name)
                });

            let dest_schema = dest_extension_pkg_info
                .schema_store
                .as_ref()
                .and_then(|store| {
                    store.data_in.get(msg_conversion_validate_info.msg_name)
                });

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
            let src_schema = src_extension_pkg_info
                .schema_store
                .as_ref()
                .and_then(|store| {
                    store
                        .audio_frame_out
                        .get(msg_conversion_validate_info.msg_name)
                });

            let dest_schema = dest_extension_pkg_info
                .schema_store
                .as_ref()
                .and_then(|store| {
                    store
                        .audio_frame_in
                        .get(msg_conversion_validate_info.msg_name)
                });

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
            let src_schema = src_extension_pkg_info
                .schema_store
                .as_ref()
                .and_then(|store| {
                    store
                        .video_frame_out
                        .get(msg_conversion_validate_info.msg_name)
                });

            let dest_schema = dest_extension_pkg_info
                .schema_store
                .as_ref()
                .and_then(|store| {
                    store
                        .video_frame_in
                        .get(msg_conversion_validate_info.msg_name)
                });

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

pub fn validate_connection_schema(
    graph: &mut Graph,
    graph_app_base_dir: &Option<String>,
    msg_conversion_validate_info: &MsgConversionValidateInfo,
    uri_to_pkg_info: &HashMap<Option<String>, PkgsInfoInAppWithBaseDir>,
    pkgs_cache: &HashMap<String, PkgsInfoInApp>,
) -> Result<()> {
    if msg_conversion_validate_info.msg_conversion.is_some() {
        validate_msg_conversion_schema(
            graph,
            graph_app_base_dir,
            msg_conversion_validate_info,
            uri_to_pkg_info,
            pkgs_cache,
        )?;
    } else {
        check_schema_compatibility(
            msg_conversion_validate_info,
            uri_to_pkg_info,
        )?;
    }
    Ok(())
}
