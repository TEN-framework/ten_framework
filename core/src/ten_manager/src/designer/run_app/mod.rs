//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
mod cmd_run;
mod msg_in;
mod msg_out;

use std::{
    collections::VecDeque,
    process::Child,
    sync::{Arc, Mutex, RwLock},
};

use actix::{Actor, Handler, Message, StreamHandler};
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use msg_in::InboundMsg;
use msg_out::OutboundMsg;
use serde::{Deserialize, Serialize};

use crate::{
    constants::{PROCESS_STDERR_MAX_LINE_CNT, PROCESS_STDOUT_MAX_LINE_CNT},
    designer::DesignerState,
};

// The output (stdout, stderr) and exit status from the child process.
#[derive(Message)]
#[rtype(result = "()")]
pub enum RunAppOutput {
    StdOut(String),
    StdErr(String),
    Exit(i32),
}

#[derive(Serialize, Deserialize)]
struct MsgFromFrontend {
    #[serde(rename = "type")]
    msg_type: String,
}

pub struct WsRunApp {
    state: Arc<RwLock<DesignerState>>,
    child: Option<Child>,
    buffer_stdout: Arc<Mutex<VecDeque<String>>>,
    buffer_stderr: Arc<Mutex<VecDeque<String>>>,
}

impl WsRunApp {
    pub fn new(state: Arc<RwLock<DesignerState>>) -> Self {
        Self {
            state,
            child: None,
            buffer_stdout: Arc::new(Mutex::new(VecDeque::new())),
            buffer_stderr: Arc::new(Mutex::new(VecDeque::new())),
        }
    }
}

impl Actor for WsRunApp {
    // Each actor runs within its own context and can receive and process
    // messages. This context provides various methods for interacting with the
    // actor, such as sending messages, closing connections, and more.
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        // We don't yet spawn the child command. We'll wait for the first
        // message from client that includes `base_dir` and `name`.
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        // If the process is still running, try to kill it.
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
        }
    }
}

impl Handler<RunAppOutput> for WsRunApp {
    type Result = ();

    // Handles the output (stderr, stdout) and exit status from the child
    // process.
    fn handle(
        &mut self,
        msg: RunAppOutput,
        ctx: &mut Self::Context,
    ) -> Self::Result {
        match msg {
            RunAppOutput::StdOut(line) => {
                // Push the line into buffer.
                {
                    let mut buf = self.buffer_stdout.lock().unwrap();
                    buf.push_back(line.clone());

                    while buf.len() > PROCESS_STDOUT_MAX_LINE_CNT {
                        buf.pop_front();
                    }
                }

                // Send the line to the client.
                let msg_out = OutboundMsg::StdOut { data: line };
                let out_str = serde_json::to_string(&msg_out).unwrap();

                // Sends a text message to the WebSocket client.
                ctx.text(out_str);
            }
            RunAppOutput::StdErr(line) => {
                {
                    let mut buf = self.buffer_stderr.lock().unwrap();
                    buf.push_back(line.clone());
                    while buf.len() > PROCESS_STDERR_MAX_LINE_CNT {
                        buf.pop_front();
                    }
                }

                let msg_out = OutboundMsg::StdErr { data: line };
                let out_str = serde_json::to_string(&msg_out).unwrap();

                // Sends a text message to the WebSocket client.
                ctx.text(out_str);
            }
            RunAppOutput::Exit(code) => {
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

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsRunApp {
    // Handles messages from WebSocket clients, including text messages, Ping,
    // Close, and more.
    fn handle(
        &mut self,
        item: Result<ws::Message, ws::ProtocolError>,
        ctx: &mut Self::Context,
    ) {
        match item {
            Ok(ws::Message::Text(text)) => {
                // Attempt to parse the JSON text from client.
                match serde_json::from_str::<InboundMsg>(&text) {
                    Ok(inbound) => match inbound {
                        InboundMsg::Run { base_dir, name } => {
                            self.cmd_run(&base_dir, &name, ctx);
                        }
                    },
                    Err(e) => {
                        let err_msg = OutboundMsg::Error {
                            msg: format!(
                                "Failed to parse inbound message: {}",
                                e
                            ),
                        };
                        let out_str = serde_json::to_string(&err_msg).unwrap();
                        ctx.text(out_str);
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

pub async fn run_app(
    req: HttpRequest,
    stream: web::Payload,
    state: web::Data<Arc<RwLock<DesignerState>>>,
) -> Result<HttpResponse, Error> {
    // The client connects to the `run_app` route via WebSocket, creating an
    // instance of the `WsRunApp` actor.
    ws::start(WsRunApp::new(state.get_ref().clone()), &req, stream)
}
