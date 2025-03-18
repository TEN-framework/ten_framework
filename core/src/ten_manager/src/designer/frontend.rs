//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::{
    sync::{Arc, RwLock},
    time::Instant,
};

use actix_web::{web, HttpRequest, HttpResponse, Responder};
use mime_guess::from_path;
use rust_embed::RustEmbed;

use super::DesignerState;

// Points to the frontend build output directory.
#[derive(RustEmbed)]
#[folder = "designer_frontend/dist/"]
struct Asset;

pub async fn get_frontend_asset(
    req: HttpRequest,
    state: web::Data<Arc<RwLock<DesignerState>>>,
) -> Result<impl Responder, actix_web::Error> {
    let state_read = state.read().unwrap();

    let start_time = Instant::now();
    let path = req.path().trim_start_matches('/').to_owned();
    let client_ip = req
        .connection_info()
        .peer_addr()
        .unwrap_or("unknown")
        .to_string();

    if state_read.tman_config.verbose {
        println!(
            "[FRONTEND ASSET] Request from IP: {}, Path: '{}'",
            client_ip, path
        );
    }

    if path.is_empty() {
        // If the root path is requested, return `index.html`.
        match Asset::get("index.html") {
            Some(content) => {
                let size = content.data.len();

                if state_read.tman_config.verbose {
                    println!("[FRONTEND ASSET] Serving index.html, size: {} bytes, time: {:?}", size, start_time.elapsed());
                }
                Ok(HttpResponse::Ok()
                    .content_type("text/html")
                    .body(content.data.into_owned()))
            }
            None => {
                if state_read.tman_config.verbose {
                    println!("[FRONTEND ASSET] ERROR: index.html not found!");
                }

                Ok(HttpResponse::NotFound().body("404 Not Found"))
            }
        }
    } else {
        match Asset::get(&path) {
            Some(content) => {
                let mime = from_path(&path).first_or_octet_stream();
                let size = content.data.len();

                if state_read.tman_config.verbose {
                    println!("[FRONTEND ASSET] Serving: '{}', type: {}, size: {} bytes, time: {:?}",
                             path, mime.as_ref(), size, start_time.elapsed());
                }

                Ok(HttpResponse::Ok()
                    .content_type(mime.as_ref())
                    .body(content.data.into_owned()))
            }
            // If the file is not found, return `index.html` to support React
            // Router.
            None => {
                if state_read.tman_config.verbose {
                    println!("[FRONTEND ASSET] Asset '{}' not found, falling back to index.html (SPA mode)", path);
                }

                match Asset::get("index.html") {
                    Some(content) => {
                        let size = content.data.len();

                        if state_read.tman_config.verbose {
                            println!("[FRONTEND ASSET] Serving index.html (fallback), size: {} bytes, time: {:?}",
                                 size, start_time.elapsed());
                        }

                        Ok(HttpResponse::Ok()
                            .content_type("text/html")
                            .body(content.data.into_owned()))
                    }
                    None => {
                        if state_read.tman_config.verbose {
                            println!("[FRONTEND ASSET] ERROR: index.html fallback not found!");
                        }

                        Ok(HttpResponse::NotFound().body("404 Not Found"))
                    }
                }
            }
        }
    }
}
