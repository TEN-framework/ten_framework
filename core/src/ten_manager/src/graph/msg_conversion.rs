//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use anyhow::Result;

use ten_rust::graph::msg_conversion::MsgConversion;

pub fn msg_conversion_get_final_target_schema(
    _src_app: Option<String>,
    _src_extension_addon: String,
    _dest_app: Option<String>,
    _dest_extension_addon: String,
    _msg_conversion: &MsgConversion,
) -> Result<()> {
    Ok(())
}
