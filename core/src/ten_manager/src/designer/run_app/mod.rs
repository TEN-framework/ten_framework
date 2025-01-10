//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::{
    collections::VecDeque,
    process::{Child, Command, Stdio},
    sync::{Arc, Mutex, RwLock},
    thread,
};

use actix::{Actor, AsyncContext, Handler, Message, StreamHandler};
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use serde::Deserialize;
use serde_json::Value;

use ten_rust::pkg_info::{pkg_type::PkgType, PkgInfo};

use crate::designer::response::{ErrorResponse, Status};
use crate::designer::{get_all_pkgs::get_all_pkgs, DesignerState};

#[derive(Deserialize)]
pub struct RunAppParams {
    pub base_dir: String,
    pub name: String,
}

// For partial or line-based read from child.
#[derive(Message)]
#[rtype(result = "()")]
pub enum RunAppOutput {
    StdOut(String),
    StdErr(String),
    Exit(i32),
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
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
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

    fn handle(
        &mut self,
        msg: RunAppOutput,
        ctx: &mut Self::Context,
    ) -> Self::Result {
        match msg {
            RunAppOutput::StdOut(line) => {
                // We can push line into buffer.
                {
                    let mut buf = self.buffer_stdout.lock().unwrap();
                    buf.push_back(line.clone());
                    // For example, keep only last 200 lines.
                    while buf.len() > 200 {
                        buf.pop_front();
                    }
                }
                // Then send it to client.
                let json_msg = serde_json::json!({
                    "type": "stdout",
                    "data": line
                });
                ctx.text(json_msg.to_string());
            }
            RunAppOutput::StdErr(line) => {
                {
                    let mut buf = self.buffer_stderr.lock().unwrap();
                    buf.push_back(line.clone());
                    // Keep only last 200 lines.
                    while buf.len() > 200 {
                        buf.pop_front();
                    }
                }
                let json_msg = serde_json::json!({
                    "type": "stderr",
                    "data": line
                });
                ctx.text(json_msg.to_string());
            }
            RunAppOutput::Exit(code) => {
                let json_msg = serde_json::json!({
                    "type": "exit",
                    "code": code
                });
                ctx.text(json_msg.to_string());
                // close the WebSocket.
                ctx.close(None);
            }
        }
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsRunApp {
    fn handle(
        &mut self,
        item: Result<ws::Message, ws::ProtocolError>,
        ctx: &mut Self::Context,
    ) {
        match item {
            Ok(ws::Message::Text(text)) => {
                // Attempt to parse the JSON text from client.
                if let Ok(json) = serde_json::from_str::<Value>(&text) {
                    // If this is the initial 'run' command we expect.
                    if let Some(cmd_type) =
                        json.get("type").and_then(|v| v.as_str())
                    {
                        if cmd_type == "run" {
                            // parse base_dir and name.
                            if let (Some(base_dir), Some(name)) = (
                                json.get("base_dir").and_then(|v| v.as_str()),
                                json.get("name").and_then(|v| v.as_str()),
                            ) {
                                // 1) Check if base_dir matches the _state.
                                let mut guard = self.state.write().unwrap();
                                if guard.base_dir.as_deref() != Some(base_dir) {
                                    let err_msg = ErrorResponse {
                                        status: Status::Fail,
                                        message: format!("Base directory [{}] is not opened in DesignerState", base_dir),
                                        error: None
                                    };
                                    ctx.text(
                                        serde_json::to_string(&err_msg)
                                            .unwrap(),
                                    );
                                    ctx.close(None);
                                    return;
                                }

                                // 2) get all packages.
                                if let Err(err) = get_all_pkgs(&mut guard) {
                                    let error_response =
                                        ErrorResponse::from_error(
                                            &err,
                                            "Error fetching packages:",
                                        );
                                    ctx.text(
                                        serde_json::to_string(&error_response)
                                            .unwrap(),
                                    );
                                    ctx.close(None);
                                    return;
                                }

                                // 3) find package of type == app.
                                let app_pkgs: Vec<&PkgInfo> = guard
                                    .all_pkgs
                                    .as_ref()
                                    .map(|v| {
                                        v.iter()
                                            .filter(|p| {
                                                p.basic_info
                                                    .type_and_name
                                                    .pkg_type
                                                    == PkgType::App
                                            })
                                            .collect()
                                    })
                                    .unwrap_or_default();

                                if app_pkgs.len() != 1 {
                                    let err_msg = ErrorResponse {
                                        status: Status::Fail,
                                        message: "There should be exactly one app package, found 0 or more".to_string(),
                                        error: None
                                    };
                                    ctx.text(
                                        serde_json::to_string(&err_msg)
                                            .unwrap(),
                                    );
                                    ctx.close(None);
                                    return;
                                }

                                let app_pkg = app_pkgs[0];

                                // 4) read the scripts from the manifest.
                                let scripts = match &app_pkg.manifest {
                                    Some(m) => {
                                        m.scripts.clone().unwrap_or_default()
                                    }
                                    None => {
                                        let err_msg = ErrorResponse {
                                            status: Status::Fail,
                                            message: "No manifest found in the app package".to_string(),
                                            error: None
                                        };
                                        ctx.text(
                                            serde_json::to_string(&err_msg)
                                                .unwrap(),
                                        );
                                        ctx.close(None);
                                        return;
                                    }
                                };

                                // 5) find script that matches 'name'.
                                let script_cmd = match scripts.get(name) {
                                    Some(cmd) => cmd.clone(),
                                    None => {
                                        let err_msg = ErrorResponse {
                                            status: Status::Fail,
                                            message: format!("Script '{}' not found in app manifest", name),
                                            error: None
                                        };
                                        ctx.text(
                                            serde_json::to_string(&err_msg)
                                                .unwrap(),
                                        );
                                        ctx.close(None);
                                        return;
                                    }
                                };

                                // 6) run the command line, capture
                                //    stdout/stderr.
                                let child = match Command::new("sh")
                                    .arg("-c")
                                    .arg(&script_cmd)
                                    .stdout(Stdio::piped())
                                    .stderr(Stdio::piped())
                                    .spawn()
                                {
                                    Ok(c) => c,
                                    Err(e) => {
                                        let err_msg = ErrorResponse {
                                            status: Status::Fail,
                                            message: format!(
                                                "Failed to spawn command: {}",
                                                e
                                            ),
                                            error: None,
                                        };
                                        ctx.text(
                                            serde_json::to_string(&err_msg)
                                                .unwrap(),
                                        );
                                        ctx.close(None);
                                        return;
                                    }
                                };

                                self.child = Some(child);

                                // spawn threads to read stdout & stderr.
                                let stdout_child =
                                    self.child.as_mut().unwrap().stdout.take();
                                let stderr_child =
                                    self.child.as_mut().unwrap().stderr.take();

                                let addr = ctx.address();

                                // read stdout.
                                if let Some(mut out) = stdout_child {
                                    let addr_stdout = addr.clone();

                                    thread::spawn(move || {
                                        use std::io::{BufRead, BufReader};
                                        let reader = BufReader::new(&mut out);
                                        for line_res in reader.lines() {
                                            match line_res {
                                                Ok(line) => {
                                                    addr_stdout.do_send(
                                                        RunAppOutput::StdOut(
                                                            line,
                                                        ),
                                                    );
                                                }
                                                Err(_) => break,
                                            }
                                        }
                                        // after reading is finished.
                                    });
                                }

                                // read stderr.
                                if let Some(mut err) = stderr_child {
                                    let addr_stderr = addr.clone();

                                    thread::spawn(move || {
                                        use std::io::{BufRead, BufReader};
                                        let reader = BufReader::new(&mut err);

                                        for line_res in reader.lines() {
                                            match line_res {
                                                Ok(line) => {
                                                    addr_stderr.do_send(
                                                        RunAppOutput::StdErr(
                                                            line,
                                                        ),
                                                    );
                                                }
                                                Err(_) => break,
                                            }
                                        }
                                        // after reading is finished.
                                    });
                                }

                                // wait for child exit in another thread.
                                let addr2 = ctx.address();
                                if let Some(mut child) = self.child.take() {
                                    thread::spawn(move || {
                                        if let Ok(status) = child.wait() {
                                            addr2.do_send(RunAppOutput::Exit(
                                                status.code().unwrap_or(-1),
                                            ));
                                        } else {
                                            addr2.do_send(RunAppOutput::Exit(
                                                -1,
                                            ));
                                        }
                                    });
                                }
                            }
                        } else if cmd_type == "ping" {
                            // Simple keep-alive handshake, for example.
                            ctx.text(r#"{"type":"pong"}"#);
                        }
                    }
                }
            }
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Close(_)) => {
                ctx.close(None);
            }
            // ignore other message types.
            _ => {}
        }
    }
}

pub async fn run_app(
    req: HttpRequest,
    stream: web::Payload,
    state: web::Data<Arc<RwLock<DesignerState>>>,
) -> Result<HttpResponse, Error> {
    ws::start(WsRunApp::new(state.get_ref().clone()), &req, stream)
}
