//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
pub mod apps;
pub mod builtin_function;
mod common;
mod dir_list;
mod env;
mod exec;
mod file_content;
pub mod frontend;
pub mod graphs;
mod help_text;
mod messages;
pub mod mock;
pub mod registry;
pub mod response;
pub mod terminal;
mod version;

use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use actix_web::web;

use ten_rust::pkg_info::PkgInfo;

use crate::config::TmanConfig;
use crate::output::TmanOutput;

pub struct DesignerState {
    pub tman_config: Arc<TmanConfig>,
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
            .route("/version", web::get().to(version::get_version_endpoint))
            .route(
                "/check-update",
                web::get().to(version::check_update_endpoint),
            )
            .route("/env", web::get().to(env::get_env_endpoint))
            .route("/graphs", web::post().to(graphs::get_graphs_endpoint))
            .route(
                "/graphs",
                web::put().to(graphs::update::update_graph_endpoint),
            )
            .route(
                "/graphs/nodes",
                web::post().to(graphs::nodes::get_graph_nodes_endpoint),
            )
            .route(
                "/graphs/connections",
                web::post()
                    .to(graphs::connections::get_graph_connections_endpoint),
            )
            .route(
                "/help-text",
                web::post().to(help_text::get_help_text_endpoint),
            )
            .route(
                "/messages/compatible",
                web::post()
                    .to(messages::compatible::get_compatible_messages_endpoint),
            )
            .route(
                "/registry/packages",
                web::get().to(registry::packages::get_packages_endpoint),
            )
            .route("/apps", web::get().to(apps::get::get_apps_endpoint))
            .route("/apps", web::post().to(apps::load::load_app_endpoint))
            .route(
                "/apps/unload",
                web::post().to(apps::unload::unload_app_endpoint),
            )
            .route(
                "/apps/reload",
                web::post().to(apps::reload::reload_app_endpoint),
            )
            .route(
                "/apps/addons",
                web::post().to(apps::addons::get_app_addons_endpoint),
            )
            .route(
                "/apps/scripts",
                web::post().to(apps::scripts::get_app_scripts_endpoint),
            )
            .route("/dir-list", web::post().to(dir_list::list_dir_endpoint))
            .route(
                "/file-content",
                web::post().to(file_content::get_file_content_endpoint),
            )
            .route(
                "/file-content",
                web::put().to(file_content::save_file_content_endpoint),
            )
            .route(
                "/ws/builtin-function",
                web::get().to(builtin_function::builtin_function_endpoint),
            )
            .route("/ws/exec", web::get().to(exec::exec_endpoint))
            .route(
                "/ws/terminal",
                web::get().to(terminal::ws_terminal_endpoint),
            ),
    );
}
