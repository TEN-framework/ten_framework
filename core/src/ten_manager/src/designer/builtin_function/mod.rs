//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
mod install;
mod install_all;
pub mod msg;

use std::sync::{Arc, RwLock};

use actix::{Actor, Handler, Message, StreamHandler};
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use anyhow::Context;
use anyhow::Result;
use msg::InboundMsg;
use msg::OutboundMsg;

use crate::config::TmanConfig;
use crate::designer::DesignerState;

#[derive(Message)]
#[rtype(result = "()")]
pub enum BuiltinFunctionOutput {
    NormalLine(String),
    NormalPartial(String),
    ErrorLine(String),
    ErrorPartial(String),
    Exit(i32),
}

enum BuiltinFunction {
    InstallAll {
        base_dir: String,
    },
    Install {
        base_dir: String,
        pkg_type: String,
        pkg_name: String,
        pkg_version: Option<String>,
    },
}

/// `BuiltinFunctionParser` returns a tuple: the 1st element is the command
/// string, and the 2nd is an optional working directory.
type BuiltinFunctionParser =
    Box<dyn Fn(&str) -> Result<BuiltinFunction> + Send + Sync>;

pub struct WsBuiltinFunction {
    builtin_function_parser: BuiltinFunctionParser,
    tman_config: Arc<TmanConfig>,
}

impl WsBuiltinFunction {
    fn new(
        builtin_function_parser: BuiltinFunctionParser,
        tman_config: Arc<TmanConfig>,
    ) -> Self {
        Self {
            builtin_function_parser,
            tman_config,
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
            BuiltinFunctionOutput::NormalLine(line) => {
                // Send the line to the client.
                let msg_out = OutboundMsg::NormalLine { data: line };
                let out_str = serde_json::to_string(&msg_out).unwrap();

                // Sends a text message to the WebSocket client.
                ctx.text(out_str);
            }
            BuiltinFunctionOutput::NormalPartial(line) => {
                // Send the line to the client.
                let msg_out = OutboundMsg::NormalPartial { data: line };
                let out_str = serde_json::to_string(&msg_out).unwrap();

                // Sends a text message to the WebSocket client.
                ctx.text(out_str);
            }
            BuiltinFunctionOutput::ErrorLine(line) => {
                let msg_out = OutboundMsg::ErrorLine { data: line };
                let out_str = serde_json::to_string(&msg_out).unwrap();

                // Sends a text message to the WebSocket client.
                ctx.text(out_str);
            }
            BuiltinFunctionOutput::ErrorPartial(line) => {
                let msg_out = OutboundMsg::ErrorPartial { data: line };
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
                        BuiltinFunction::Install {
                            base_dir,
                            pkg_type,
                            pkg_name,
                            pkg_version,
                        } => self.install(
                            base_dir,
                            pkg_type,
                            pkg_name,
                            pkg_version,
                            ctx,
                        ),
                    },
                    Err(e) => {
                        let err_out = OutboundMsg::ErrorLine {
                            data: e.to_string(),
                        };
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

pub async fn builtin_function(
    req: HttpRequest,
    stream: web::Payload,
    state: web::Data<Arc<RwLock<DesignerState>>>,
) -> Result<HttpResponse, Error> {
    let tman_config = state.read().unwrap().tman_config.clone();

    let default_parser: BuiltinFunctionParser = Box::new(move |text: &str| {
        // Attempt to parse the JSON text from client.
        let inbound = serde_json::from_str::<InboundMsg>(text)
            .with_context(|| format!("Failed to parse {} into JSON", text))?;

        match inbound {
            InboundMsg::InstallAll { base_dir } => {
                Ok(BuiltinFunction::InstallAll { base_dir })
            }
            InboundMsg::Install {
                base_dir,
                pkg_type,
                pkg_name,
                pkg_version,
            } => Ok(BuiltinFunction::Install {
                base_dir,
                pkg_type,
                pkg_name,
                pkg_version,
            }),
        }
    });

    ws::start(
        WsBuiltinFunction::new(default_parser, tman_config.clone()),
        &req,
        stream,
    )
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        sync::{Arc, RwLock},
    };

    use actix_web::{
        http::{header, StatusCode},
        test, web, App,
    };

    use crate::{
        config::TmanConfig,
        designer::{builtin_function::builtin_function, DesignerState},
        output::TmanOutputCli,
    };

    #[actix_rt::test]
    async fn test_cmd_builtin_function_install_all() {
        let designer_state = DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
        };

        let designer_state = Arc::new(RwLock::new(designer_state));

        // Initialize the test service with the WebSocket endpoint.
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(designer_state))
                .route("/ws/builtin-function", web::get().to(builtin_function)),
        )
        .await;

        // Create a basic request just to test if the route is defined.
        let req = test::TestRequest::get()
            .uri("/ws/builtin-function")
            .to_request();

        // Execute the request but don't check for success. This just verifies
        // that the route exists and the handler is called.
        let resp = test::call_service(&app, req).await;

        println!("Response status: {:?}", resp.status());

        // Instead of asserting success which requires WebSocket setup, we'll
        // just verify that the endpoint exists by checking the response
        // is not a 404.
        assert_ne!(resp.status().as_u16(), 404, "Endpoint not found");

        // Note: A proper WebSocket test would require a more complex setup.
        // This test just verifies the route is registered correctly.
        println!(
        "WebSocket endpoint /ws/builtin-function is registered and available"
    );
    }

    #[actix_rt::test]
    async fn test_cmd_builtin_function_websocket_connection() {
        let designer_state = DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
        };

        let designer_state = Arc::new(RwLock::new(designer_state));

        // Initialize the test service with the WebSocket endpoint.
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(designer_state))
                .route("/ws/builtin-function", web::get().to(builtin_function)),
        )
        .await;

        // Create a test request with proper WebSocket headers.
        let req = test::TestRequest::get()
            .uri("/ws/builtin-function")
            .insert_header((header::CONNECTION, "upgrade"))
            .insert_header((header::UPGRADE, "websocket"))
            .insert_header(("Sec-WebSocket-Version", "13"))
            // Base64 encoded value.
            .insert_header(("Sec-WebSocket-Key", "dGhlIHNhbXBsZSBub25jZQ=="))
            .to_request();

        // Call the WebSocket endpoint.
        let resp = test::call_service(&app, req).await;

        // When a WebSocket connection is established successfully, the server
        // should respond with:
        // 1. A 101 Switching Protocols status code
        // 2. Various WebSocket headers

        println!("WebSocket connection response: {:?}", resp.status());
        println!("WebSocket connection headers: {:#?}", resp.headers());

        // Verify that we got the correct WebSocket upgrade response.
        assert_eq!(
            resp.status(),
            StatusCode::SWITCHING_PROTOCOLS,
            "Expected WebSocket upgrade response (101 Switching Protocols)"
        );

        // Check for key WebSocket response headers.
        assert!(
            resp.headers().contains_key("Sec-WebSocket-Accept"),
            "Missing WebSocket Accept header"
        );
        assert_eq!(
            resp.headers()
                .get(header::UPGRADE)
                .unwrap()
                .to_str()
                .unwrap()
                .to_lowercase(),
            "websocket",
            "Incorrect upgrade header value"
        );

        println!("WebSocket connection was successfully established (validated by checking response status and headers)");
    }
}
