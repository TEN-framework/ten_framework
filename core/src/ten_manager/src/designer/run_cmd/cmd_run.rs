//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::{process::Command, thread};

use actix::AsyncContext;
use actix_web_actors::ws::WebsocketContext;

use crate::designer::run_cmd::RunCmdOutput;

use super::{msg_out::OutboundMsg, WsRunCmd};

impl WsRunCmd {
    pub fn cmd_run(
        &mut self,
        cmd: &String,
        ctx: &mut WebsocketContext<WsRunCmd>,
    ) {
        let mut command = Command::new("sh");
        command
            .arg("-c")
            .arg(cmd)
            .env("TEN_LOG_FORMATTER", "default")
            // Capture stdout/stderr.
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        if let Some(ref dir) = self.working_directory {
            command.current_dir(dir);
        }

        // Run the command.
        let child = match command.spawn() {
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
                            addr_stdout.do_send(RunCmdOutput::StdOut(line));
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
                            addr_stderr.do_send(RunCmdOutput::StdErr(line));
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
                    addr2.do_send(RunCmdOutput::Exit(
                        status.code().unwrap_or(-1),
                    ));
                } else {
                    addr2.do_send(RunCmdOutput::Exit(-1));
                }
            });
        }
    }
}
