//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
mod addons;
mod app;
mod builtin_function;
mod common;
mod dir_list;
mod exec;
mod file_content;
pub mod frontend;
pub mod graphs;
mod manifest;
mod messages;
pub mod mock;
mod packages;
mod property;
pub mod response;
mod terminal;
mod version;

use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use actix_web::web;

use ten_rust::pkg_info::PkgInfo;

use super::config::TmanConfig;
use crate::output::TmanOutput;
pub struct DesignerState {
    pub tman_config: TmanConfig,
    pub out: Arc<Box<dyn TmanOutput>>,
    pub pkgs_cache: HashMap<String, Vec<PkgInfo>>,
}

pub fn configure_routes(
    cfg: &mut web::ServiceConfig,
    state: web::Data<Arc<RwLock<DesignerState>>>,
) {
    cfg.service(
        web::scope("/api/designer/v1")
            .app_data(state)
            .route("/version", web::get().to(version::get_version))
            .route(
                "/check-update",
                web::get().to(version::check_update_endpoint),
            )
            .route(
                "/addons/extensions",
                web::post().to(addons::extensions::get_extension_addon),
            )
            .route(
                "/packages/reload",
                web::post().to(packages::reload::clear_and_reload_pkgs),
            )
            .route("/graphs", web::post().to(graphs::get_graphs))
            .route("/graphs", web::put().to(graphs::update::update_graph))
            .route(
                "/graphs/nodes",
                web::post().to(graphs::nodes::get_graph_nodes),
            )
            .route(
                "/graphs/connections",
                web::post().to(graphs::connections::get_graph_connections),
            )
            .route(
                "/manifest/check",
                web::post().to(manifest::check::check_manifest),
            )
            .route(
                "/property/check",
                web::get().to(property::check::check_property),
            )
            .route(
                "/messages/compatible",
                web::post().to(messages::compatible::get_compatible_messages),
            )
            .route(
                "/file-content",
                web::post().to(file_content::get_file_content),
            )
            .route(
                "/file-content",
                web::put().to(file_content::save_file_content),
            )
            .route("/app/base-dir", web::post().to(app::base_dir::add_base_dir))
            .route(
                "/app/base-dir",
                web::delete().to(app::base_dir::delete_base_dir),
            )
            .route("/app/base-dir", web::get().to(app::base_dir::get_base_dir))
            .route("/dir-list", web::post().to(dir_list::list_dir))
            .route("/ws/exec", web::get().to(exec::exec))
            .route(
                "/ws/builtin-function",
                web::get().to(builtin_function::builtin_function),
            )
            .route("/ws/terminal", web::get().to(terminal::ws_terminal)),
    );
}
