//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::process;
use std::sync::Arc;

use anyhow::Result;
use console::Emoji;
use ten_manager::cmd::execute_cmd;
use ten_manager::config::metadata::TmanMetadata;
use tokio::runtime::Runtime;

use ten_manager::cmd_line;
use ten_manager::constants::GITHUB_RELEASE_PAGE;
use ten_manager::output::{TmanOutput, TmanOutputCli};
use ten_manager::version::VERSION;
use ten_manager::version_utils::check_update;

fn check_update_from_cmdline(out: Arc<Box<dyn TmanOutput>>) -> Result<()> {
    out.normal_line("Checking for new version...");

    let rt = Runtime::new()?;

    match rt.block_on(check_update()) {
        Ok((true, latest)) => {
            out.normal_line(&format!(
                "New version found: {}. Please go to {} to download the update.",
                latest, GITHUB_RELEASE_PAGE
            ));
        }
        Ok((false, _)) => {
            out.normal_line("Already up to date.");
        }
        Err(err_msg) => {
            out.normal_line(&err_msg.to_string());
        }
    }

    Ok(())
}

fn main() {
    let out: Arc<Box<dyn TmanOutput>> = Arc::new(Box::new(TmanOutputCli));

    let parsed_cmd = match cmd_line::parse_cmd() {
        Ok(parsed_cmd) => parsed_cmd,
        Err(e) => {
            out.error_line(&format!("{}  Error: {}", Emoji("🔴", ":-("), e));
            process::exit(1);
        }
    };

    if parsed_cmd.show_version {
        out.normal_line(&format!("TEN Framework version: {}", VERSION));

        // Call the update check function
        match check_update_from_cmdline(out.clone()) {
            Ok(_) => {
                // If `--version` is passed, ignore other parameters and exit
                // directly.
                process::exit(0);
            }
            Err(e) => {
                out.error_line(&format!(
                    "{}  Error: {}",
                    Emoji("🔴", ":-("),
                    e
                ));
                process::exit(1);
            }
        }
    }

    let rt = match Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            out.error_line(&format!("{}  Error: {}", Emoji("🔴", ":-("), e));
            process::exit(1);
        }
    };

    let result = rt.block_on(execute_cmd(
        parsed_cmd.tman_config,
        Arc::new(tokio::sync::RwLock::new(TmanMetadata::default())),
        parsed_cmd.command_data.unwrap(),
        out.clone(),
    ));

    if let Err(e) = result {
        out.error_line(&format!("{}  Error: {:?}", Emoji("🔴", ":-("), e));
        process::exit(1);
    }

    process::exit(0);
}
