//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::sync::Arc;

use actix::AsyncContext;
use actix_web_actors::ws::WebsocketContext;
use tokio::task::LocalSet;

use ten_rust::pkg_info::manifest::support::ManifestSupport;

use super::{msg::TmanOutputWs, BuiltinFunctionOutput, WsBuiltinFunction};
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
            support: ManifestSupport {
                os: None,
                arch: None,
            },
            local_install_mode: crate::cmd::cmd_install::LocalInstallMode::Link,
            standalone: false,
            local_path: None,
            cwd: base_dir.clone(),
        };

        let addr = ctx.address();
        let output_ws: Arc<Box<dyn TmanOutput>> =
            Arc::new(Box::new(TmanOutputWs { addr: addr.clone() }));

        let addr = addr.clone();

        // Clone the config and metadata before the async block.
        let tman_config = self.tman_config.clone();
        let tman_metadata = self.tman_metadata.clone();

        // Create a LocalSet to ensure spawn_local runs on this thread.
        let local = LocalSet::new();

        // Spawn the task within the LocalSet context.
        local.spawn_local(async move {
            // Now perform the actual work.
            let result = crate::cmd::cmd_install::execute_cmd(
                tman_config,
                tman_metadata,
                install_command,
                output_ws,
            )
            .await;

            // Notify the WebSocket client that the task is complete.
            let exit_code = if result.is_ok() { 0 } else { -1 };
            let error_message = if let Err(err) = result {
                Some(err.to_string())
            } else {
                None
            };
            addr.do_send(BuiltinFunctionOutput::Exit {
                exit_code,
                error_message,
            });
        });

        // Use spawn to run the LocalSet in the background.
        actix_web::rt::spawn(async move {
            local.await;
        });
    }
}
