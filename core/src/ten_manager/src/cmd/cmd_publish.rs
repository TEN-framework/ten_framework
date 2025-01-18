//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::time::Instant;

use anyhow::Result;
use clap::{ArgMatches, Command};
use console::Emoji;
use indicatif::HumanDuration;

use ten_rust::pkg_info::get_pkg_info_from_path;

use crate::config::TmanConfig;
use crate::constants::DOT_TEN_DIR;
use crate::constants::PACKAGE_DIR_IN_DOT_TEN_DIR;
use crate::log::tman_verbose_println;
use crate::package_file::create_package_tar_gz_file;
use crate::package_file::get_tpkg_file_name;
use crate::registry::upload_package;

#[derive(Debug)]
pub struct PublishCommand {}

pub fn create_sub_cmd(_args_cfg: &crate::cmd_line::ArgsCfg) -> Command {
    Command::new("publish")
        .about("Publish a package")
        .after_help(
            "Switch to the base directory of the TEN package you want to \
        publish, then simply run 'tman publish' directly in that directory.",
        )
}

pub fn parse_sub_cmd(
    _sub_cmd_args: &ArgMatches,
) -> Result<crate::cmd::cmd_publish::PublishCommand> {
    Ok(crate::cmd::cmd_publish::PublishCommand {})
}

pub async fn execute_cmd(
    tman_config: &TmanConfig,
    command_data: PublishCommand,
) -> Result<()> {
    tman_verbose_println!(tman_config, "Executing publish command");
    tman_verbose_println!(tman_config, "{:?}", command_data);

    let started = Instant::now();

    let cwd = crate::fs::get_cwd()?;

    let pkg_info = get_pkg_info_from_path(&cwd, true)?;
    let output_pkg_file_name = get_tpkg_file_name(&pkg_info)?;

    // Use the default output path, which is located in the `.ten/`
    // directory, ensuring that under normal circumstances, it will not be
    // uploaded to the git repository.

    // Create the output directory.
    let output_dir = cwd.join(DOT_TEN_DIR).join(PACKAGE_DIR_IN_DOT_TEN_DIR);
    if !output_dir.exists() {
        std::fs::create_dir_all(&output_dir)?;
    }

    let output_path = output_dir.join(output_pkg_file_name);
    if output_path.exists() {
        std::fs::remove_file(&output_path)?;
    }

    // Generate the package file.
    let output_path_str =
        create_package_tar_gz_file(tman_config, &output_path, &cwd)?;

    upload_package(tman_config, &output_path_str, &pkg_info).await?;

    if tman_config.mi_mode {
        println!("Publish to {:?}", output_path_str);
    } else {
        println!(
            "{}  Publish successfully in {}",
            Emoji("🏆", ":-)"),
            HumanDuration(started.elapsed())
        );
    }

    // Remove the package file.
    std::fs::remove_file(&output_path_str)?;

    Ok(())
}
