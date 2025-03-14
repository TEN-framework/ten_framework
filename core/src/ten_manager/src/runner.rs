//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::sync::Arc;

use anyhow::Result;
use tokio::runtime::Runtime;

use crate::cmd::{execute_cmd, CommandData};
use crate::config::TmanConfig;
use crate::output::TmanOutput;

pub fn run_tman_command(
    tman_config: Arc<TmanConfig>,
    cmd_data: CommandData,
    out: Arc<Box<dyn TmanOutput>>,
) -> Result<()> {
    let rt = Runtime::new().unwrap();
    rt.block_on(execute_cmd(tman_config, cmd_data, out))?;

    Ok(())
}
