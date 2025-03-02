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

fn check_update_from_cmdline(tman_config: &TmanConfig) {
    println!("Checking for new version...");

    let rt = Runtime::new().unwrap();
    match rt.block_on(check_update()) {
        Ok((true, latest)) => {
            println!(
                "New version found: {}. Please download the update.",
                latest
            );
        }
        Ok((false, _)) => {
            if tman_config.verbose {
                println!("Already up to date.");
            }
        }
        Err(err_msg) => {
            if tman_config.verbose {
                println!("{}", err_msg);
            }
        }
    }
}

fn main() {
    let mut tman_config_from_cmd_line = TmanConfig::default();

    let command_data = match cmd_line::parse_cmd(&mut tman_config_from_cmd_line)
    {
        Ok(cmd_data) => cmd_data,
        Err(e) => {
            eprintln!("{}  Error: {}", Emoji("🔴", ":-("), e);
            process::exit(1);
        }
    };

    let tman_config_from_config_file = ten_manager::config::read_config(
        &tman_config_from_cmd_line.config_file,
    );

    let tman_config =
        merge(tman_config_from_cmd_line, tman_config_from_config_file);

    check_update_from_cmdline(&tman_config);

    let rt = Runtime::new().unwrap();
    let result = rt.block_on(cmd::execute_cmd(&tman_config, command_data));
    if let Err(e) = result {
        println!("{}  Error: {:?}", Emoji("🔴", ":-("), e);
        process::exit(1);
    }

    process::exit(0);
}
