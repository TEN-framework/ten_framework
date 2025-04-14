//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::process;
use std::sync::Arc;

use console::Emoji;
use ten_manager::config::internal::TmanInternalConfig;
use tokio::runtime::Runtime;

use ten_manager::cmd_line;
use ten_manager::constants::GITHUB_RELEASE_PAGE;
use ten_manager::output::{TmanOutput, TmanOutputCli};
use ten_manager::runner::run_tman_command;
use ten_manager::version::VERSION;
use ten_manager::version_utils::check_update;

fn check_update_from_cmdline(out: Arc<Box<dyn TmanOutput>>) {
    out.normal_line("Checking for new version...");

    let rt = Runtime::new().unwrap();
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
        check_update_from_cmdline(out.clone());

        // If `--version` is passed, ignore other parameters and exit directly.
        std::process::exit(0);
    }

    let result = run_tman_command(
        parsed_cmd.tman_config,
        Arc::new(TmanInternalConfig::default()),
        parsed_cmd.command_data.unwrap(),
        out.clone(),
    );

    if let Err(e) = result {
        out.error_line(&format!("{}  Error: {:?}", Emoji("🔴", ":-("), e));
        process::exit(1);
    }

    process::exit(0);
}
