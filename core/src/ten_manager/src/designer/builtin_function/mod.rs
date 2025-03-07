//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
mod install_all;
mod msg_out;

use std::sync::{Arc, RwLock};

use actix::{Actor, Handler, Message, StreamHandler};
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use anyhow::Context;
use anyhow::Result;
use msg_out::OutboundMsg;
use serde::{Deserialize, Serialize};

use crate::designer::DesignerState;

#[derive(Message)]
#[rtype(result = "()")]
pub enum BuiltinFunctionOutput {
    NormalOutput(String),
    ErrorOutput(String),
    Exit(i32),
}

enum BuiltinFunction {
    InstallAll { base_dir: String },
}

/// `BuiltinFunctionParser` returns a tuple: the 1st element is the command
/// string, and the 2nd is an optional working directory.
type BuiltinFunctionParser =
    Box<dyn Fn(&str) -> Result<BuiltinFunction> + Send + Sync>;

pub struct WsBuiltinFunction {
    builtin_function_parser: BuiltinFunctionParser,
}

impl WsBuiltinFunction {
    fn new(builtin_function_parser: BuiltinFunctionParser) -> Self {
        Self {
            builtin_function_parser,
        }
    }
}

impl Actor for WsBuiltinFunction {
    // Each actor runs within its own context and can receive and process
    // messages. This context provides various methods for interacting with the
    // actor, such as sending messages, closing connections, and more.
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {}

    fn stopped(&mut self, _ctx: &mut Self::Context) {}
}

impl Handler<BuiltinFunctionOutput> for WsBuiltinFunction {
    type Result = ();

    // Handles the output and exit status from the builtin function.
    fn handle(
        &mut self,
        msg: BuiltinFunctionOutput,
        ctx: &mut Self::Context,
    ) -> Self::Result {
        match msg {
            BuiltinFunctionOutput::NormalOutput(line) => {
                // Send the line to the client.
                let msg_out = OutboundMsg::NormalOutput { data: line };
                let out_str = serde_json::to_string(&msg_out).unwrap();

                // Sends a text message to the WebSocket client.
                ctx.text(out_str);
            }
            BuiltinFunctionOutput::ErrorOutput(line) => {
                let msg_out = OutboundMsg::ErrorOutput { data: line };
                let out_str = serde_json::to_string(&msg_out).unwrap();

                // Sends a text message to the WebSocket client.
                ctx.text(out_str);
            }
            BuiltinFunctionOutput::Exit(code) => {
                // Send it to the client.
                let msg_out = OutboundMsg::Exit { code };
                let out_str = serde_json::to_string(&msg_out).unwrap();

                // Sends a text message to the WebSocket client.
                ctx.text(out_str);

                // Close the WebSocket. Passing `None` as a parameter indicates
                // sending a default close frame (Close Frame) to the client and
                // closing the WebSocket connection.
                ctx.close(None);
            }
        }
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>>
    for WsBuiltinFunction
{
    // Handles messages from WebSocket clients, including text messages, Ping,
    // Close, and more.
    fn handle(
        &mut self,
        item: Result<ws::Message, ws::ProtocolError>,
        ctx: &mut Self::Context,
    ) {
        match item {
            Ok(ws::Message::Text(text)) => {
                match (self.builtin_function_parser)(&text) {
                    Ok(builtin_function) => match builtin_function {
                        BuiltinFunction::InstallAll { base_dir } => {
                            self.install_all(base_dir, ctx)
                        }
                    },
                    Err(e) => {
                        let err_out = OutboundMsg::Error { msg: e.to_string() };
                        let out_str = serde_json::to_string(&err_out).unwrap();
                        ctx.text(out_str);
                        ctx.close(None);
                    }
                }
            }
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Close(_)) => {
                ctx.close(None);
            }
            // Ignore other message types.
            _ => {}
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
enum InboundMsg {
    #[serde(rename = "install_all")]
    InstallAll { base_dir: String },
}

pub async fn builtin_function(
    req: HttpRequest,
    stream: web::Payload,
    _state: web::Data<Arc<RwLock<DesignerState>>>,
) -> Result<HttpResponse, Error> {
    let default_parser: BuiltinFunctionParser = Box::new(move |text: &str| {
        // Attempt to parse the JSON text from client.
        let inbound = serde_json::from_str::<InboundMsg>(text)
            .with_context(|| format!("Failed to parse {} into JSON", text))?;

        match inbound {
            InboundMsg::InstallAll { base_dir } => {
                Ok(BuiltinFunction::InstallAll { base_dir })
            }
        }
    });

    ws::start(WsBuiltinFunction::new(default_parser), &req, stream)
}
