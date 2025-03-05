//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//

// Enable this when development.
// #![allow(dead_code)]

use std::process;

use console::Emoji;
use ten_manager::constants::GITHUB_RELEASE_PAGE;
use ten_manager::output::{TmanOutput, TmanOutputCli};
use ten_manager::runner::run_tman_command;
use ten_manager::version::VERSION;
use ten_manager::version_utils::check_update;
use tokio::runtime::Runtime;

use ten_manager::cmd_line;
use ten_manager::config::TmanConfig;

fn merge(cmd_line: TmanConfig, config_file: TmanConfig) -> TmanConfig {
    TmanConfig {
        registry: config_file.registry,
        config_file: cmd_line.config_file,
        admin_token: cmd_line.admin_token.or(config_file.admin_token),
        user_token: cmd_line.user_token.or(config_file.user_token),
        verbose: cmd_line.verbose,
        assume_yes: cmd_line.assume_yes,
        enable_package_cache: config_file.enable_package_cache,
    }
}

fn check_update_from_cmdline(out: &TmanOutput) {
    out.output_line("Checking for new version...");

    let rt = Runtime::new().unwrap();
    match rt.block_on(check_update()) {
        Ok((true, latest)) => {
            out.output_line(&format!(
                "New version found: {}. Please go to {} to download the update.",
                latest, GITHUB_RELEASE_PAGE
            ));
        }
        Ok((false, _)) => {
            out.output_line("Already up to date.");
        }
        Err(err_msg) => {
            out.output_line(&err_msg.to_string());
        }
    }
}

fn main() {
    let out = TmanOutput::Cli(TmanOutputCli);

    let parsed_cmd = match cmd_line::parse_cmd() {
        Ok(parsed_cmd) => parsed_cmd,
        Err(e) => {
            out.output_err_line(&format!(
                "{}  Error: {}",
                Emoji("🔴", ":-("),
                e
            ));
            process::exit(1);
        }
    };

    let tman_config_from_cmd_line = parsed_cmd.tman_config;
    let tman_config_from_config_file = ten_manager::config::read_config(
        &tman_config_from_cmd_line.config_file,
    );

    let tman_config =
        merge(tman_config_from_cmd_line, tman_config_from_config_file);

    if parsed_cmd.show_version {
        out.output_line(&format!("TEN Framework version: {}", VERSION));

        // Call the update check function
        check_update_from_cmdline(&out);

        // If `--version` is passed, ignore other parameters and exit directly.
        std::process::exit(0);
    }

    let result =
        run_tman_command(tman_config, parsed_cmd.command_data.unwrap(), &out);

    if let Err(e) = result {
        out.output_err_line(&format!("{}  Error: {:?}", Emoji("🔴", ":-("), e));
        process::exit(1);
    }

    process::exit(0);
}
