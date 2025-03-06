//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use actix::AsyncContext;
use actix_web_actors::ws::WebsocketContext;

use ten_rust::pkg_info::supports::PkgSupport;

use super::{BuiltinFunctionOutput, WsBuiltinFunction};
use crate::output::TmanOutput;

impl WsBuiltinFunction {
    pub fn install_all(
        &mut self,
        base_dir: String,
        ctx: &mut WebsocketContext<WsBuiltinFunction>,
    ) {
        let install_command = crate::cmd::cmd_install::InstallCommand {
            package_type: None,
            package_name: None,
            support: PkgSupport {
                os: None,
                arch: None,
            },
            local_install_mode: crate::cmd::cmd_install::LocalInstallMode::Link,
            standalone: false,
            local_path: Some(base_dir.clone()),
        };

        let tman_config = crate::config::TmanConfig::default();

        let output_ws = self.output_ws.clone();
        let addr = ctx.address();

        // Call `execute_cmd()` in an async task.
        tokio::spawn(async move {
            let output_ws = TmanOutput::Ws(output_ws);

            let result = crate::cmd::cmd_install::execute_cmd(
                &tman_config,
                install_command,
                &output_ws,
            )
            .await;

            // Notify the WebSocket client that the task is complete, and
            // determine the exit code based on the result.
            let exit_code = if result.is_ok() { 0 } else { -1 };
            addr.do_send(BuiltinFunctionOutput::Exit(exit_code));
        });

        // Send the messages collected by `TmanOutputWs` to the WebSocket client
        // periodically.
        ctx.run_interval(std::time::Duration::from_millis(100), |act, ctx| {
            let msgs = act.output_ws.take_messages();
            for msg in msgs {
                ctx.text(msg);
            }
        });
    }
}
