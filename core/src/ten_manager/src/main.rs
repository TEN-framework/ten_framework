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
use ten_manager::version::VERSION;
use ten_manager::version_utils::check_update;
use tokio::runtime::Runtime;

use ten_manager::cmd;
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

fn check_update_from_cmdline() {
    println!("Checking for new version...");

    let rt = Runtime::new().unwrap();
    match rt.block_on(check_update()) {
        Ok((true, latest)) => {
            println!(
                "New version found: {}. Please go to {} to download the update.",
                latest, GITHUB_RELEASE_PAGE
            );
        }
        Ok((false, _)) => {
            println!("Already up to date.");
        }
        Err(err_msg) => {
            println!("{}", err_msg);
        }
    }
}

fn main() {
    let parsed_cmd = match cmd_line::parse_cmd() {
        Ok(parsed_cmd) => parsed_cmd,
        Err(e) => {
            eprintln!("{}  Error: {}", Emoji("🔴", ":-("), e);
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
        println!("TEN Framework version: {}", VERSION);

        // Call the update check function
        check_update_from_cmdline();

        // If --version is passed, ignore other parameters and exit directly.
        std::process::exit(0);
    }

    let rt = Runtime::new().unwrap();
    let result = rt.block_on(cmd::execute_cmd(
        &tman_config,
        parsed_cmd.command_data.unwrap(),
    ));
    if let Err(e) = result {
        println!("{}  Error: {:?}", Emoji("🔴", ":-("), e);
        process::exit(1);
    }

    process::exit(0);
}
