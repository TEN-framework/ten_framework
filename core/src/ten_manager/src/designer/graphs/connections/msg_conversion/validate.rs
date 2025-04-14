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
    graph::{graph_info::GraphInfo, msg_conversion::MsgAndResultConversion},
    pkg_info::{
        get_pkg_info_for_extension_addon,
        message::{MsgDirection, MsgType},
    },
    schema::store::{
        are_msg_schemas_compatible, create_msg_schema_from_manifest,
        find_msg_schema_from_all_pkgs_info,
    },
};

use crate::graph::msg_conversion::msg_conversion_get_final_target_schema;

pub struct MsgConversionValidateInfo<'a> {
    pub src_app: &'a Option<String>,
    pub src_extension: &'a String,
    pub msg_type: &'a MsgType,
    pub msg_name: &'a String,
    pub dest_app: &'a Option<String>,
    pub dest_extension: &'a String,

    pub msg_conversion: &'a Option<MsgAndResultConversion>,
}

pub fn validate_msg_conversion_schema(
    graph_info: &mut GraphInfo,
    msg_conversion_validate_info: &MsgConversionValidateInfo,
    uri_to_pkg_info: &HashMap<Option<String>, PkgsInfoInAppWithBaseDir>,
    pkgs_cache: &HashMap<String, PkgsInfoInApp>,
) -> Result<()> {
    if let Some(msg_conversion) = &msg_conversion_validate_info.msg_conversion {
        let src_extension_addon =
            graph_info.graph.get_addon_name_of_extension(
                msg_conversion_validate_info.src_app,
                msg_conversion_validate_info.src_extension,
            )?;

        let converted_schema = msg_conversion_get_final_target_schema(
            uri_to_pkg_info,
            graph_info.app_base_dir.as_ref(),
            pkgs_cache,
            msg_conversion_validate_info.src_app,
            src_extension_addon,
            msg_conversion_validate_info.msg_type,
            msg_conversion_validate_info.msg_name,
            msg_conversion,
        )
        .unwrap();

        eprintln!(
            "msg_conversion converted_schema: {}",
            serde_json::to_string_pretty(&converted_schema).unwrap()
        );

        if let Ok(Some(src_ten_msg_schema)) =
            create_msg_schema_from_manifest(&converted_schema)
        {
            let dest_extension_addon =
                graph_info.graph.get_addon_name_of_extension(
                    msg_conversion_validate_info.dest_app,
                    msg_conversion_validate_info.dest_extension,
                )?;

            if let Some(dest_extension_pkg_info) =
                get_pkg_info_for_extension_addon(
                    msg_conversion_validate_info.dest_app,
                    dest_extension_addon,
                    uri_to_pkg_info,
                    graph_info.app_base_dir.as_ref(),
                    pkgs_cache,
                )
            {
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
    }

    Ok(())
}
