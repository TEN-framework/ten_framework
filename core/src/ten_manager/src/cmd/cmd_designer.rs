//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::{
    collections::HashMap,
    path::Path,
    sync::{Arc, RwLock},
};

use actix_cors::Cors;
use actix_web::{http::header, web, App, HttpServer};
use anyhow::{Ok, Result};
use clap::{value_parser, Arg, ArgMatches, Command};
use console::Emoji;
// use webbrowser;

use crate::{
    config::{internal::TmanInternalConfig, TmanConfig},
    constants::DESIGNER_BACKEND_DEFAULT_PORT,
    designer::{configure_routes, frontend::get_frontend_asset, DesignerState},
    fs::{check_is_app_folder, get_cwd},
    output::{TmanOutput, TmanOutputCli},
    pkg_info::get_all_pkgs::get_all_pkgs_in_app,
};

#[derive(Clone, Debug)]
pub struct DesignerCommand {
    pub ip_address: String,
    pub port: u16,
    pub base_dir: Option<String>,
}

pub fn create_sub_cmd(_args_cfg: &crate::cmd_line::ArgsCfg) -> Command {
    Command::new("designer")
        .about("Launch designer")
        .arg(
            Arg::new("IP_ADDRESS")
                .long("ip")
                .help("Sets the IP address")
                .default_value("0.0.0.0"),
        )
        .arg(
            Arg::new("PORT")
                .long("port")
                .help("Sets the port number")
                .value_parser(value_parser!(u16).range(0..=65535))
                // Why we choose 49483?
                //
                // 'r' -> ascii value: 114
                // 't' -> ascii value: 116
                // 'e' -> ascii value: 101
                //
                // ascii_sum = 114 + 116 + 101 = 331
                //
                // The range of dynamic or private ports is 49152-65535.
                // range_size = 65535 - 49152 + 1 = 16384
                // port_number = (331 % 16384) + 49152
                //             = 338 + 49152
                //             = 49483
                .default_value(DESIGNER_BACKEND_DEFAULT_PORT),
        )
        .arg(
            Arg::new("BASE_DIR")
                .long("base-dir")
                .help("The base directory")
                .required(false),
        )
}

pub fn parse_sub_cmd(
    sub_cmd_args: &ArgMatches,
) -> Result<crate::cmd::cmd_designer::DesignerCommand> {
    let cmd = crate::cmd::cmd_designer::DesignerCommand {
        ip_address: sub_cmd_args
            .get_one::<String>("IP_ADDRESS")
            .unwrap()
            .to_string(),
        port: *sub_cmd_args.get_one::<u16>("PORT").unwrap(),
        base_dir: sub_cmd_args.get_one::<String>("BASE_DIR").cloned(),
    };

    Ok(cmd)
}

pub async fn execute_cmd(
    tman_config: Arc<TmanConfig>,
    tman_internal_config: Arc<TmanInternalConfig>,
    command_data: DesignerCommand,
    out: Arc<Box<dyn TmanOutput>>,
) -> Result<()> {
    if tman_config.verbose {
        out.normal_line("Executing designer command");
        out.normal_line(&format!("{:?}", command_data));
        out.normal_line(&format!("{:?}", tman_config));
    }

    let base_dir = match &command_data.base_dir {
        Some(base_dir) => {
            out.normal_line(&format!("Base directory: {}", base_dir));
            base_dir.clone()
        }
        None => {
            let cwd = get_cwd()?.to_str().unwrap_or_default().to_string();

            out.normal_line(&format!(
                "{}  Doesn't specify the base directory, use current working \
                directory instead: {}",
                Emoji("💡", "!"),
                &cwd
            ));

            cwd.clone()
        }
    };

    let state = Arc::new(RwLock::new(DesignerState {
        tman_config,
        tman_internal_config,
        out: Arc::new(Box::new(TmanOutputCli)),
        pkgs_cache: HashMap::new(),
        graphs_cache: HashMap::new(),
    }));

    let mut actual_base_dir_opt: Option<String> = Some(base_dir);

    // Check if the base_dir is an app folder.
    if let Some(actual_base_dir) = actual_base_dir_opt.as_ref() {
        if let Err(e) = check_is_app_folder(Path::new(actual_base_dir)) {
            out.normal_line(&format!(
                "{}  base_dir is not an app folder: {}",
                Emoji("🚨", ":-("),
                e
            ));
            actual_base_dir_opt = None;
        }
    }

    if let Some(actual_base_dir) = actual_base_dir_opt.as_ref() {
        let mut state_write = state.write().unwrap();

        // Destructure to avoid multiple mutable borrows.
        let DesignerState {
            pkgs_cache,
            graphs_cache,
            ..
        } = &mut *state_write;

        get_all_pkgs_in_app(pkgs_cache, graphs_cache, actual_base_dir)?;
    }

    let server = HttpServer::new(move || {
        let state = web::Data::new(state.clone());

        let cors = Cors::default()
            .allow_any_origin()
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
            .allowed_headers(vec![
                header::AUTHORIZATION,
                header::ACCEPT,
                header::CONTENT_TYPE,
            ])
            .max_age(3600);

        App::new()
            .app_data(state.clone())
            .wrap(cors)
            .configure(|cfg| configure_routes(cfg, state.clone()))
            .default_service(web::to(get_frontend_asset))
    });

    let bind_address =
        format!("{}:{}", command_data.ip_address, command_data.port);

    out.normal_line(&format!(
        "{}  Starting server at http://{}",
        Emoji("🏆", ":-)"),
        bind_address,
    ));

    server.bind(&bind_address)?.run().await?;

    Ok(())
}
