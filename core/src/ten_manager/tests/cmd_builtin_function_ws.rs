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
use futures_util::{SinkExt, StreamExt};
use tokio::{self, sync::oneshot};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

use ten_manager::{
    config::TmanConfig,
    designer::{
        builtin_function::{builtin_function, msg::InboundMsg},
        DesignerState,
    },
    output::TmanOutputCli,
};

#[actix_rt::test]
async fn test_ws_builtin_function_install_all() {
    // Create channel for server address.
    let (addr_tx, addr_rx) = oneshot::channel();

    // Start server in a separate thread.
    let _ = thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let tman_config = TmanConfig {
                config_file: Some(
                    std::env::current_dir()
                        .unwrap()
                        .parent()
                        .and_then(|p| p.parent())
                        .map(|p| p.to_path_buf())
                        .unwrap()
                        .join("config.json")
                        .to_string_lossy()
                        .into_owned(),
                ),
                ..Default::default()
            };

            // Setup designer state.
            let designer_state = DesignerState {
                tman_config: Arc::new(tman_config),
                out: Arc::new(Box::new(TmanOutputCli)),
                pkgs_cache: HashMap::new(),
            };

            let designer_state = Arc::new(RwLock::new(designer_state));

            // Start the server.
            let server = HttpServer::new(move || {
                App::new()
                    .app_data(web::Data::new(designer_state.clone()))
                    .route(
                        "/ws/builtin-function",
                        web::get().to(builtin_function),
                    )
            })
            // Using port 0 for automatic port selection.
            .bind("127.0.0.1:0")
            .unwrap();

            // Get the actual server address.
            let server_addr = server.addrs()[0];
            let _ = addr_tx.send(server_addr);

            // Run the server - this will block until the server is stopped
            // or the thread is terminated.
            server.run().await.unwrap();
        });
    });

    // Wait for the server to start and get the address.
    let server_addr = addr_rx.await.unwrap();
    println!("Server started at: {}", server_addr);

    // Small delay to ensure server is fully initialized.
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Connect WebSocket client
    let ws_url = format!("ws://{}/ws/builtin-function", server_addr);
    let (ws_stream, _) = connect_async(ws_url).await.unwrap();
    println!("WebSocket connection established");

    // Split the WebSocket stream.
    let (mut write, mut read) = ws_stream.split();

    // Prepare InstallAll message.
    let install_all_msg = InboundMsg::InstallAll {
        base_dir: "tests/test_data/cmd_check_start_graph_cmd".to_string(),
    };
    let json_msg = serde_json::to_string(&install_all_msg).unwrap();

    // Send the message.
    write.send(Message::Text(json_msg)).await.unwrap();
    println!(
        "Sent InstallAll message: {}",
        serde_json::to_string(&install_all_msg).unwrap()
    );

    // Wait for first response message.
    if let Some(Ok(message)) = read.next().await {
        if let Message::Text(text) = message {
            println!("Received response: {}", text);
            // We just verify that we got any response, no need to validate
            // content yet.
            assert!(!text.is_empty(), "Response should not be empty");
        } else {
            panic!("Expected text message");
        }
    } else {
        panic!("Did not receive message from server");
    }

    // Close the connection.
    let _ = write.send(Message::Close(None)).await;

    // We don't gracefully stop the server, just let the thread continue running
    // and it will be cleaned up when the process exits.
    println!("Test completed successfully");
}
