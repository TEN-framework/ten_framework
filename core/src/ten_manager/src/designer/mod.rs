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
mod locale;
mod messages;
pub mod mock;
pub mod preferences;
pub mod registry;
pub mod response;
mod template_pkgs;
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
            // Version endpoints.
            .route("/version", web::get().to(version::get_version_endpoint))
            .route(
                "/check-update",
                web::get().to(version::check_update_endpoint),
            )
            // Apps endpoints.
            .route("/apps", web::get().to(apps::get::get_apps_endpoint))
            .route("/apps/load", web::post().to(apps::load::load_app_endpoint))
            .route(
                "/apps/unload",
                web::post().to(apps::unload::unload_app_endpoint),
            )
            .route(
                "/apps/reload",
                web::post().to(apps::reload::reload_app_endpoint),
            )
            .route(
                "/apps/create",
                web::post().to(apps::create::create_app_endpoint),
            )
            .route(
                "/apps/addons",
                web::post().to(apps::addons::get_app_addons_endpoint),
            )
            .route(
                "/apps/scripts",
                web::post().to(apps::scripts::get_app_scripts_endpoint),
            )
            // Template packages endpoint.
            .route(
                "/template-pkgs",
                web::post().to(template_pkgs::get_template_endpoint),
            )
            // Graphs endpoints.
            .route("/graphs", web::post().to(graphs::get::get_graphs_endpoint))
            .route(
                "/graphs",
                web::put().to(graphs::update::update_graph_endpoint),
            )
            // Graph nodes endpoints.
            .route(
                "/graphs/nodes",
                web::post().to(graphs::nodes::get::get_graph_nodes_endpoint),
            )
            .route(
                "/graphs/nodes/add",
                web::post().to(graphs::nodes::add::add_graph_node_endpoint),
            )
            .route(
                "/graphs/nodes/delete",
                web::post().to(graphs::nodes::delete::delete_graph_node_endpoint),
            )
            // Graph connections endpoints.
            .route(
                "/graphs/connections",
                web::post().to(graphs::connections::get::get_graph_connections_endpoint),
            )
            .route(
                "/graphs/connections/add",
                web::post().to(graphs::connections::add::add_graph_connection_endpoint),
            )
            // Preferences endpoints.
            .route(
                "/preferences/schema",
                web::get().to(
                    preferences::get_schema::get_preferences_schema_endpoint,
                ),
            )
            .route(
                "/preferences",
                web::get().to(preferences::get::get_preferences_endpoint),
            )
            .route(
                "/preferences",
                web::put().to(preferences::update::update_preferences_endpoint),
            )
            .route(
                "/preferences/field",
                web::patch().to(
                    preferences::update_field::update_preferences_field_endpoint,
                ),
            )
            // File system endpoints.
            .route("/dir-list", web::post().to(dir_list::list_dir_endpoint))
            .route(
                "/file-content",
                web::post().to(file_content::get_file_content_endpoint),
            )
            .route(
                "/file-content",
                web::put().to(file_content::save_file_content_endpoint),
            )
            // Websocket endpoints.
            .route(
                "/ws/builtin-function",
                web::get().to(builtin_function::builtin_function_endpoint),
            )
            .route("/ws/exec", web::get().to(exec::exec_endpoint))
            .route(
                "/ws/terminal",
                web::get().to(terminal::ws_terminal_endpoint),
            )
            // Misc endpoints.
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
            .route("/env", web::get().to(env::get_env_endpoint)),
    );
}
