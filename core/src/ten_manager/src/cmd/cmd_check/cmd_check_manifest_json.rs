//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use anyhow::Result;
use clap::{Arg, ArgMatches, Command};
use console::Emoji;
use ten_rust::json_schema::ten_validate_manifest_json_string;

use crate::{config::TmanConfig, fs::read_file_to_string};

#[derive(Debug)]
pub struct CheckManifestJsonCommand {
    pub path: String,
}

pub fn create_sub_cmd(_args_cfg: &crate::cmd_line::ArgsCfg) -> Command {
    Command::new("manifest-json")
        .about(
            "Check if the manifest.json is valid according to the json schema",
        )
        .arg(
            Arg::new("PATH")
                .long("path")
                .help("The file path of manifest.json to be checked")
                .required(true)
                .num_args(1),
        )
}

pub fn parse_sub_cmd(
    sub_cmd_args: &ArgMatches,
) -> Result<CheckManifestJsonCommand> {
    let cmd = CheckManifestJsonCommand {
        path: sub_cmd_args.get_one::<String>("PATH").cloned().unwrap(),
    };

    Ok(cmd)
}

pub async fn execute_cmd(
    _tman_config: &TmanConfig,
    command_data: CheckManifestJsonCommand,
) -> Result<()> {
    let content = read_file_to_string(&command_data.path)?;
    match ten_validate_manifest_json_string(&content) {
        Ok(_) => {
            println!("{}  Conforms to JSON schema.", Emoji("👍", "Passed"));
            Ok(())
        }
        Err(e) => Err(e),
    }
}
