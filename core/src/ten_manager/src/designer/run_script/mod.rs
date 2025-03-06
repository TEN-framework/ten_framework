//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::sync::{Arc, RwLock};

use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use ten_rust::pkg_info::{pkg_type::PkgType, PkgInfo};

use crate::designer::{
    exec::{CmdParser, WsRunCmd},
    get_all_pkgs::get_all_pkgs,
    DesignerState,
};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
enum InboundMsg {
    #[serde(rename = "start")]
    Start { base_dir: String, name: String },
}

fn extract_command_from_manifest(
    base_dir: &String,
    name: &String,
    state: Arc<RwLock<DesignerState>>,
) -> Result<String> {
    {
        let mut state = state.write().unwrap();

        // 1) Check if base_dir matches the state.
        if state.base_dir.as_deref() != Some(base_dir) {
            return Err(anyhow::anyhow!(
                "Base directory [{}] is not opened.",
                base_dir
            ));
        }

        // 2) Get all packages in the base_dir.
        if let Err(err) = get_all_pkgs(&mut state) {
            return Err(anyhow::anyhow!("Error fetching packages: {}", err));
        }
    }

    let state = state.read().unwrap();

    // 3) find the package of type == app.
    let app_pkgs: Vec<&PkgInfo> = state
        .all_pkgs
        .as_ref()
        .map(|v| {
            v.iter()
                .filter(|p| p.basic_info.type_and_name.pkg_type == PkgType::App)
                .collect()
        })
        .unwrap_or_default();

    if app_pkgs.len() != 1 {
        return Err(anyhow::anyhow!(
            "There should be exactly one app package, found 0 or more"
        ));
    }

    let app_pkg = app_pkgs[0];

    // 4) Read the `scripts` field from the `manifest.json`.
    let scripts = match &app_pkg.manifest {
        Some(m) => m.scripts.clone().unwrap_or_default(),
        None => {
            return Err(anyhow::anyhow!(
                "No manifest found in the app package"
            ));
        }
    };

    // 5) Find script that matches `name`.
    let script_cmd = match scripts.get(name) {
        Some(cmd) => cmd.clone(),
        None => {
            return Err(anyhow::anyhow!(
                "Script '{}' not found in app manifest",
                name
            ));
        }
    };

    Ok(script_cmd)
}

pub async fn run_script(
    req: HttpRequest,
    stream: web::Payload,
    state: web::Data<Arc<RwLock<DesignerState>>>,
) -> Result<HttpResponse, Error> {
    let state_clone = state.get_ref().clone();

    // The client connects to the `run_app` route via WebSocket, creating an
    // instance of the `WsRunApp` actor.
    let default_parser: CmdParser = Box::new(move |text: &str| {
        // Attempt to parse the JSON text from client.
        let inbound = serde_json::from_str::<InboundMsg>(text)
            .with_context(|| format!("Failed to parse {} into JSON", text))?;

        match inbound {
            InboundMsg::Start { base_dir, name } => {
                let cmd = extract_command_from_manifest(
                    &base_dir,
                    &name,
                    state_clone.clone(),
                )?;
                Ok((cmd, Some(base_dir)))
            }
        }
    });

    ws::start(WsRunCmd::new(default_parser), &req, stream)
}
