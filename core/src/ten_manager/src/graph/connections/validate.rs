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
        manifest::api::ManifestApiPropertyAttributes,
        message::{MsgDirection, MsgType},
        pkg_type::PkgType,
        PkgInfo,
    },
    schema::store::{
        are_msg_schemas_compatible, are_ten_schemas_compatible,
        create_c_schema_from_properties_and_required,
        find_msg_schema_from_all_pkgs_info, TenMsgSchema,
    },
};

use crate::graph::msg_conversion::{
    msg_conversion_get_dest_msg_name, msg_conversion_get_final_target_schema,
};

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

#[allow(clippy::too_many_arguments)]
fn create_and_compare_c_schema(
    graph_app_base_dir: &Option<String>,
    msg_conversion_validate_info: &MsgConversionValidateInfo,
    uri_to_pkg_info: &HashMap<Option<String>, PkgsInfoInAppWithBaseDir>,
    pkgs_cache: &HashMap<String, PkgsInfoInApp>,
    compared_schema_properties: &Option<
        HashMap<String, ManifestApiPropertyAttributes>,
    >,
    compared_schema_required: &Option<Vec<String>>,
    target_app: &Option<String>,
    target_extension_addon: &String,
    target_msg_name: &str,
    msg_direction: MsgDirection,
) -> Result<()> {
    if let Ok(compared_c_schema) = create_c_schema_from_properties_and_required(
        compared_schema_properties,
        compared_schema_required,
    ) {
        if let Some(target_extension_pkg_info) =
            get_pkg_info_for_extension_addon(
                pkgs_cache,
                uri_to_pkg_info,
                graph_app_base_dir,
                target_app,
                target_extension_addon,
            )
        {
            if let Some(target_ten_msg_schema) =
                find_msg_schema_from_all_pkgs_info(
                    target_extension_pkg_info,
                    msg_conversion_validate_info.msg_type,
                    target_msg_name,
                    &msg_direction,
                )
            {
                let c_schema = if msg_direction == MsgDirection::In {
                    target_ten_msg_schema.msg.as_ref()
                } else {
                    target_ten_msg_schema.result.as_ref()
                };

                are_ten_schemas_compatible(
                    compared_c_schema.as_ref(),
                    c_schema,
                    true,
                    true,
                )?;
            }
        }
    }

    Ok(())
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

    let dest_extension_addon = graph.get_addon_name_of_extension(
        msg_conversion_validate_info.dest_app,
        msg_conversion_validate_info.dest_extension,
    )?;

    // Default to using `src_msg_name` as the `dest_msg_name`, but check if
    // there's a special rule for `_ten.name` to determine the `dest_msg_name`.
    let (dest_msg_name, ten_name_rule_index) =
        msg_conversion_get_dest_msg_name(
            msg_conversion_validate_info.msg_name,
            msg_conversion_validate_info
                .msg_conversion
                .as_ref()
                .unwrap(),
        )?;

    let (converted_schema, converted_result_schema) =
        msg_conversion_get_final_target_schema(
            uri_to_pkg_info,
            graph_app_base_dir,
            pkgs_cache,
            msg_conversion_validate_info.src_app,
            src_extension_addon,
            msg_conversion_validate_info.dest_app,
            dest_extension_addon,
            msg_conversion_validate_info.msg_type,
            msg_conversion_validate_info.msg_name,
            &dest_msg_name,
            ten_name_rule_index,
            msg_conversion_validate_info
                .msg_conversion
                .as_ref()
                .unwrap(),
        )?;

    eprintln!(
        "msg_conversion converted_schema: {}",
        serde_json::to_string_pretty(&converted_schema).unwrap()
    );

    eprintln!(
        "msg_conversion converted_result_schema: {}",
        serde_json::to_string_pretty(&converted_result_schema).unwrap()
    );

    create_and_compare_c_schema(
        graph_app_base_dir,
        msg_conversion_validate_info,
        uri_to_pkg_info,
        pkgs_cache,
        &converted_schema.property,
        &converted_schema.required,
        msg_conversion_validate_info.dest_app,
        dest_extension_addon,
        &converted_schema.name,
        MsgDirection::In,
    )?;

    if let Some(converted_result_schema) = converted_result_schema {
        create_and_compare_c_schema(
            graph_app_base_dir,
            msg_conversion_validate_info,
            uri_to_pkg_info,
            pkgs_cache,
            &converted_result_schema.property,
            &converted_result_schema.required,
            msg_conversion_validate_info.src_app,
            src_extension_addon,
            msg_conversion_validate_info.msg_name,
            MsgDirection::Out,
        )?;
    } else {
        // No result conversion, so directly check if the original source result
        // schema and destination result schema are compatible.
        let (src_schema, dest_schema, error_message) = get_src_and_dest_schema(
            graph,
            msg_conversion_validate_info,
            uri_to_pkg_info,
        )?;

        if src_schema.is_none() || dest_schema.is_none() {
            return Ok(());
        }

        let src_schema = src_schema.unwrap();
        let dest_schema = dest_schema.unwrap();

        if let Err(err) = are_ten_schemas_compatible(
            src_schema.result.as_ref(),
            dest_schema.result.as_ref(),
            true,
            true,
        ) {
            assert!(error_message.is_some());
            return Err(anyhow::anyhow!("{}: {}", error_message.unwrap(), err));
        }
    }

    Ok(())
}

/// Finds package info for source and destination apps and extensions.
fn find_pkg_infos<'a>(
    uri_to_pkg_info: &'a HashMap<Option<String>, PkgsInfoInAppWithBaseDir>,
    src_app: &Option<String>,
    src_extension_addon: &str,
    dest_app: &Option<String>,
    dest_extension_addon: &str,
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
        find_extension_pkg(src_app, src_extension_addon, true)?;
    let dest_extension_pkg_info =
        find_extension_pkg(dest_app, dest_extension_addon, false)?;

    Ok((src_extension_pkg_info, dest_extension_pkg_info))
}

fn get_src_and_dest_schema<'a>(
    graph: &mut Graph,
    msg_conversion_validate_info: &MsgConversionValidateInfo,
    uri_to_pkg_info: &'a HashMap<Option<String>, PkgsInfoInAppWithBaseDir>,
) -> Result<(
    Option<&'a TenMsgSchema>,
    Option<&'a TenMsgSchema>,
    Option<String>,
)> {
    let src_extension_addon = graph.get_addon_name_of_extension(
        msg_conversion_validate_info.src_app,
        msg_conversion_validate_info.src_extension,
    )?;

    let dest_extension_addon = graph.get_addon_name_of_extension(
        msg_conversion_validate_info.dest_app,
        msg_conversion_validate_info.dest_extension,
    )?;

    let (src_extension_pkg_info, dest_extension_pkg_info) = find_pkg_infos(
        uri_to_pkg_info,
        msg_conversion_validate_info.src_app,
        src_extension_addon,
        msg_conversion_validate_info.dest_app,
        dest_extension_addon,
    )?;

    if src_extension_pkg_info.is_none() || dest_extension_pkg_info.is_none() {
        return Ok((None, None, None));
    }

    let src_extension_pkg_info = src_extension_pkg_info.unwrap();
    let dest_extension_pkg_info = dest_extension_pkg_info.unwrap();

    // Get source and destination schemas based on message type.
    let (src_schema, dest_schema, error_message) =
        match msg_conversion_validate_info.msg_type {
            MsgType::Cmd => {
                let src = src_extension_pkg_info
                    .schema_store
                    .as_ref()
                    .and_then(|store| {
                        store.cmd_out.get(msg_conversion_validate_info.msg_name)
                    });
                let dest = dest_extension_pkg_info
                    .schema_store
                    .as_ref()
                    .and_then(|store| {
                        store.cmd_in.get(msg_conversion_validate_info.msg_name)
                    });
                (src, dest, "Command schema incompatibility between source and destination")
            }
            MsgType::Data => {
                let src = src_extension_pkg_info
                    .schema_store
                    .as_ref()
                    .and_then(|store| {
                        store
                            .data_out
                            .get(msg_conversion_validate_info.msg_name)
                    });
                let dest = dest_extension_pkg_info
                    .schema_store
                    .as_ref()
                    .and_then(|store| {
                        store.data_in.get(msg_conversion_validate_info.msg_name)
                    });
                (src, dest, "Data schema incompatibility between source and destination")
            }
            MsgType::AudioFrame => {
                let src = src_extension_pkg_info
                    .schema_store
                    .as_ref()
                    .and_then(|store| {
                        store
                            .audio_frame_out
                            .get(msg_conversion_validate_info.msg_name)
                    });
                let dest = dest_extension_pkg_info
                    .schema_store
                    .as_ref()
                    .and_then(|store| {
                        store
                            .audio_frame_in
                            .get(msg_conversion_validate_info.msg_name)
                    });
                (src, dest, "Audio frame schema incompatibility between source and destination")
            }
            MsgType::VideoFrame => {
                let src = src_extension_pkg_info
                    .schema_store
                    .as_ref()
                    .and_then(|store| {
                        store
                            .video_frame_out
                            .get(msg_conversion_validate_info.msg_name)
                    });
                let dest = dest_extension_pkg_info
                    .schema_store
                    .as_ref()
                    .and_then(|store| {
                        store
                            .video_frame_in
                            .get(msg_conversion_validate_info.msg_name)
                    });
                (src, dest, "Video frame schema incompatibility between source and destination")
            }
        };

    Ok((src_schema, dest_schema, Some(error_message.to_string())))
}

/// Checks schema compatibility between source and destination based on
/// message type.
fn check_schema_compatibility(
    graph: &mut Graph,
    msg_conversion_validate_info: &MsgConversionValidateInfo,
    uri_to_pkg_info: &HashMap<Option<String>, PkgsInfoInAppWithBaseDir>,
) -> Result<()> {
    let (src_schema, dest_schema, error_message) = get_src_and_dest_schema(
        graph,
        msg_conversion_validate_info,
        uri_to_pkg_info,
    )?;

    if src_schema.is_none() || dest_schema.is_none() {
        return Ok(());
    }

    // Check schema compatibility.
    if let Err(err) =
        are_msg_schemas_compatible(src_schema, dest_schema, true, true)
    {
        assert!(error_message.is_some());
        return Err(anyhow::anyhow!("{}: {}", error_message.unwrap(), err));
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
            graph,
            msg_conversion_validate_info,
            uri_to_pkg_info,
        )?;
    }
    Ok(())
}
