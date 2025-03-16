//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    thread,
    time::Duration,
};

use actix_web::{web, App, HttpServer};
use tokio::{self, sync::oneshot};

use ten_manager::{
    config::{read_config, TmanConfig},
    designer::{builtin_function::builtin_function_endpoint, DesignerState},
    output::TmanOutputCli,
};

use super::tman_config::find_config_json;

/// Starts a WebSocket server for the builtin function endpoint and returns the
/// server address. This is a common function used by multiple test cases to
/// avoid code duplication.
pub async fn start_builtin_function_server() -> std::net::SocketAddr {
    // Create channel for server address
    let (addr_tx, addr_rx) = oneshot::channel();

    // Start server in a separate thread
    let _ = thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let tman_config_file_path =
                find_config_json().map(|p| p.to_string_lossy().into_owned());

            let tman_config_file = read_config(&tman_config_file_path).unwrap();

            let tman_config = TmanConfig {
                config_file: tman_config_file_path,
                registry: if let Some(tman_config_file) = &tman_config_file {
                    tman_config_file.registry.clone()
                } else {
                    HashMap::new()
                },
                ..Default::default()
            };

            // Setup designer state
            let designer_state = DesignerState {
                tman_config: Arc::new(tman_config),
                out: Arc::new(Box::new(TmanOutputCli)),
                pkgs_cache: HashMap::new(),
            };

            let designer_state = Arc::new(RwLock::new(designer_state));

            // Start the server
            let server = HttpServer::new(move || {
                App::new()
                    .app_data(web::Data::new(designer_state.clone()))
                    .route(
                        "/ws/builtin-function",
                        web::get().to(builtin_function_endpoint),
                    )
            })
            // Using port 0 for automatic port selection
            .bind("127.0.0.1:0")
            .unwrap();

            // Get the actual server address
            let server_addr = server.addrs()[0];
            let _ = addr_tx.send(server_addr);

            // Run the server - this will block until the server is stopped
            // or the thread is terminated
            server.run().await.unwrap();
        });
    });

    // Wait for the server to start and get the address
    let server_addr = addr_rx.await.unwrap();

    // Small delay to ensure server is fully initialized
    tokio::time::sleep(Duration::from_millis(100)).await;

    server_addr
}
