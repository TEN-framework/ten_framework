//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
mod cmd_run;
mod msg_out;

use std::{
    collections::VecDeque,
    process::Child,
    sync::{Arc, Mutex},
};

use actix::{Actor, Handler, Message, StreamHandler};
use actix_web_actors::ws;
use anyhow::Result;
use msg_out::OutboundMsg;
use serde::{Deserialize, Serialize};

use crate::constants::{
    PROCESS_STDERR_MAX_LINE_CNT, PROCESS_STDOUT_MAX_LINE_CNT,
};

// The output (stdout, stderr) and exit status from the child process.
#[derive(Message)]
#[rtype(result = "()")]
pub enum RunCmdOutput {
    StdOut(String),
    StdErr(String),
    Exit(i32),
}

/// `CmdParser` returns a tuple: the 1st element is the command string, and
/// the 2nd is an optional working directory.
pub type CmdParser =
    Box<dyn Fn(&str) -> Result<(String, Option<String>)> + Send + Sync>;

#[derive(Serialize, Deserialize)]
struct MsgFromFrontend {
    #[serde(rename = "type")]
    msg_type: String,
}

pub struct WsRunCmd {
    child: Option<Child>,
    buffer_stdout: Arc<Mutex<VecDeque<String>>>,
    buffer_stderr: Arc<Mutex<VecDeque<String>>>,
    cmd_parser: CmdParser,
    working_directory: Option<String>,
}

impl WsRunCmd {
    pub fn new(cmd_parser: CmdParser) -> Self {
        Self {
            child: None,
            buffer_stdout: Arc::new(Mutex::new(VecDeque::new())),
            buffer_stderr: Arc::new(Mutex::new(VecDeque::new())),
            cmd_parser,
            working_directory: None,
        }
    }
}

impl Actor for WsRunCmd {
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

impl Handler<RunCmdOutput> for WsRunCmd {
    type Result = ();

    // Handles the output (stderr, stdout) and exit status from the child
    // process.
    fn handle(
        &mut self,
        msg: RunCmdOutput,
        ctx: &mut Self::Context,
    ) -> Self::Result {
        match msg {
            RunCmdOutput::StdOut(line) => {
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
            RunCmdOutput::StdErr(line) => {
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
            RunCmdOutput::Exit(code) => {
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

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsRunCmd {
    // Handles messages from WebSocket clients, including text messages, Ping,
    // Close, and more.
    fn handle(
        &mut self,
        item: Result<ws::Message, ws::ProtocolError>,
        ctx: &mut Self::Context,
    ) {
        match item {
            Ok(ws::Message::Text(text)) => match (self.cmd_parser)(&text) {
                Ok((cmd, working_directory)) => {
                    if let Some(dir) = working_directory {
                        self.working_directory = Some(dir);
                    }

                    self.cmd_run(&cmd, ctx);
                }
                Err(e) => {
                    let err_out = OutboundMsg::Error { msg: e.to_string() };
                    let out_str = serde_json::to_string(&err_out).unwrap();
                    ctx.text(out_str);
                    ctx.close(None);
                }
            },
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Close(_)) => {
                ctx.close(None);
            }
            // Ignore other message types.
            _ => {}
        }
    }
}
