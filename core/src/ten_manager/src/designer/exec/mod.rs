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

use crate::designer::{
    run_cmd::{CmdParser, WsRunCmd},
    DesignerState,
};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
enum InboundMsg {
    #[serde(rename = "exec")]
    Exec { base_dir: String, cmd: String },
}

pub async fn exec(
    req: HttpRequest,
    stream: web::Payload,
    _state: web::Data<Arc<RwLock<DesignerState>>>,
) -> Result<HttpResponse, Error> {
    let default_parser: CmdParser = Box::new(move |text: &str| {
        // Attempt to parse the JSON text from client.
        let inbound = serde_json::from_str::<InboundMsg>(text)
            .with_context(|| format!("Failed to parse {} into JSON", text))?;

        match inbound {
            InboundMsg::Exec { base_dir, cmd } => Ok((cmd, Some(base_dir))),
        }
    });

    ws::start(WsRunCmd::new(default_parser), &req, stream)
}
