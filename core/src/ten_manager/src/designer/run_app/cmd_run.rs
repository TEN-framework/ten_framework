//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::{
    process::{Command, Stdio},
    thread,
};

use actix::AsyncContext;
use actix_web_actors::ws::WebsocketContext;

use ten_rust::pkg_info::{pkg_type::PkgType, PkgInfo};

use crate::designer::{get_all_pkgs::get_all_pkgs, run_app::RunAppOutput};

use super::{msg_out::OutboundMsg, WsRunApp};

impl WsRunApp {
    pub fn cmd_run(
        &mut self,
        base_dir: &String,
        name: &String,
        ctx: &mut WebsocketContext<WsRunApp>,
    ) {
        // 1) Check if base_dir matches the state.
        let mut guard = self.state.write().unwrap();
        if guard.base_dir.as_deref() != Some(base_dir) {
            let err_msg = OutboundMsg::Error {
                msg: format!("Base directory [{}] is not opened.", base_dir),
            };

            ctx.text(serde_json::to_string(&err_msg).unwrap());
            ctx.close(None);

            return;
        }

        // 2) Get all packages in the base_dir.
        if let Err(err) = get_all_pkgs(&mut guard) {
            let err_msg = OutboundMsg::Error {
                msg: format!("Error fetching packages: {}", err),
            };

            ctx.text(serde_json::to_string(&err_msg).unwrap());
            ctx.close(None);

            return;
        }

        // 3) find the package of type == app.
        let app_pkgs: Vec<&PkgInfo> = guard
            .all_pkgs
            .as_ref()
            .map(|v| {
                v.iter()
                    .filter(|p| {
                        p.basic_info.type_and_name.pkg_type == PkgType::App
                    })
                    .collect()
            })
            .unwrap_or_default();

        if app_pkgs.len() != 1 {
            let err_msg = OutboundMsg::Error {
                msg: "There should be exactly one app package, found 0 or more"
                    .to_string(),
            };

            ctx.text(serde_json::to_string(&err_msg).unwrap());
            ctx.close(None);

            return;
        }

        let app_pkg = app_pkgs[0];

        // 4) Read the `scripts` field from the `manifest.json`.
        let scripts = match &app_pkg.manifest {
            Some(m) => m.scripts.clone().unwrap_or_default(),
            None => {
                let err_msg = OutboundMsg::Error {
                    msg: "No manifest found in the app package".to_string(),
                };

                ctx.text(serde_json::to_string(&err_msg).unwrap());
                ctx.close(None);

                return;
            }
        };

        // 5) Find script that matches `name`.
        let script_cmd = match scripts.get(name) {
            Some(cmd) => cmd.clone(),
            None => {
                let err_msg = OutboundMsg::Error {
                    msg: format!("Script '{}' not found in app manifest", name),
                };

                ctx.text(serde_json::to_string(&err_msg).unwrap());
                ctx.close(None);

                return;
            }
        };

        // 6) Run the command line, capture stdout/stderr.
        let child = match Command::new("sh")
            .arg("-c")
            .arg(&script_cmd)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(c) => c,
            Err(e) => {
                let err_msg = OutboundMsg::Error {
                    msg: format!("Failed to spawn command: {}", e),
                };

                ctx.text(serde_json::to_string(&err_msg).unwrap());
                ctx.close(None);

                return;
            }
        };

        self.child = Some(child);

        // Spawn threads to read stdout & stderr.
        let stdout_child = self.child.as_mut().unwrap().stdout.take();
        let stderr_child = self.child.as_mut().unwrap().stderr.take();

        // Returns the address of the current actor. This address serves as a
        // reference that can be used in other threads or tasks to send messages
        // to the actor.
        let addr = ctx.address();

        // Read stdout.
        if let Some(mut out) = stdout_child {
            let addr_stdout = addr.clone();

            thread::spawn(move || {
                use std::io::{BufRead, BufReader};

                let reader = BufReader::new(&mut out);
                for line_res in reader.lines() {
                    match line_res {
                        Ok(line) => {
                            // `do_send` is used to asynchronously send messages
                            // to an actor. This method does not wait for the
                            // message to be processed, making it suitable for
                            // messages that do not require a response.
                            addr_stdout.do_send(RunAppOutput::StdOut(line));
                        }
                        Err(_) => break,
                    }
                }
                // After reading is finished.
            });
        }

        // Read stderr.
        if let Some(mut err) = stderr_child {
            let addr_stderr = addr.clone();

            thread::spawn(move || {
                use std::io::{BufRead, BufReader};

                let reader = BufReader::new(&mut err);
                for line_res in reader.lines() {
                    match line_res {
                        Ok(line) => {
                            addr_stderr.do_send(RunAppOutput::StdErr(line));
                        }
                        Err(_) => break,
                    }
                }
                // After reading is finished.
            });
        }

        // Wait for child exit in another thread.
        let addr2 = ctx.address();
        if let Some(mut child) = self.child.take() {
            thread::spawn(move || {
                if let Ok(status) = child.wait() {
                    addr2.do_send(RunAppOutput::Exit(
                        status.code().unwrap_or(-1),
                    ));
                } else {
                    addr2.do_send(RunAppOutput::Exit(-1));
                }
            });
        }
    }
}
