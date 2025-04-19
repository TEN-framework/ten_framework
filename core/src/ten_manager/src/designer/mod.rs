//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
pub mod apps;
pub mod builtin_function;
pub mod common;
pub mod dir_list;
pub mod doc_link;
pub mod env;
pub mod exec;
pub mod extensions;
pub mod file_content;
pub mod frontend;
pub mod graphs;
pub mod help_text;
pub mod locale;
pub mod manifest;
pub mod messages;
pub mod metadata;
pub mod preferences;
pub mod property;
pub mod registry;
pub mod response;
pub mod template_pkgs;
pub mod terminal;
pub mod version;

use std::{collections::HashMap, sync::Arc};

use actix_web::web;
use uuid::Uuid;

use ten_rust::{
    base_dir_pkg_info::PkgsInfoInApp, graph::graph_info::GraphInfo,
};

use crate::config::{metadata::TmanMetadata, TmanConfig};
use crate::output::TmanOutput;

pub struct DesignerState {
    pub tman_config: Arc<tokio::sync::RwLock<TmanConfig>>,
    pub tman_metadata: Arc<tokio::sync::RwLock<TmanMetadata>>,
    pub out: Arc<Box<dyn TmanOutput>>,
    pub pkgs_cache: tokio::sync::RwLock<HashMap<String, PkgsInfoInApp>>,
    pub graphs_cache: tokio::sync::RwLock<HashMap<Uuid, GraphInfo>>,
}

pub fn configure_routes(
    cfg: &mut web::ServiceConfig,
    state: web::Data<Arc<DesignerState>>,
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
            .route(
                "/apps/schema",
                web::post().to(apps::schema::get_app_schema_endpoint),
            )
            // Extension endpoints.
            .route(
                "/extensions/create",
                web::post().to(extensions::create::create_extension_endpoint),
            )
            .route(
                "/extensions/schema",
                web::post().to(extensions::schema::get_extension_schema_endpoint),
            )
            .route(
                "/extensions/property/get",
                web::post().to(extensions::property::get_extension_property_endpoint),
            )
            // Manifest validation endpoints.
            .route(
                "/manifest/validate",
                web::post().to(manifest::validate::validate_manifest_endpoint),
            )
            // Property validation endpoints.
            .route(
                "/property/validate",
                web::post().to(property::validate::validate_property_endpoint),
            )
            // Template packages endpoint.
            .route(
                "/template-pkgs",
                web::post().to(template_pkgs::get_template_endpoint),
            )
            // Graphs endpoints.
            .route("/graphs", web::post().to(graphs::get::get_graphs_endpoint))
            .route(
                "/graphs/update",
                web::post().to(graphs::update::update_graph_endpoint),
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
            .route(
                "/graphs/nodes/replace",
                web::post().to(graphs::nodes::replace::replace_graph_node_endpoint),
            )
            .route(
                "/graphs/nodes/property/update",
                web::post().to(graphs::nodes::property::update::update_graph_node_property_endpoint),
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
            .route(
                "/graphs/connections/delete",
                web::post().to(graphs::connections::delete::delete_graph_connection_endpoint),
            )
            .route(
                "/graphs/connections/msg_conversion/update",
                web::post().to(graphs::connections::msg_conversion::update::update_graph_connection_msg_conversion_endpoint),
            )
            // Messages endpoints.
            .route(
                "/messages/compatible",
                web::post().to(messages::compatible::get_compatible_messages_endpoint),
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
            // Internal config endpoints.
            .route(
                "/internal-config/graph-ui/set",
                web::post().to(metadata::graph_ui::set::set_graph_ui_endpoint),
            )
            .route(
                "/internal-config/graph-ui/get",
                web::post().to(metadata::graph_ui::get::get_graph_ui_endpoint),
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
            // Doc endpoints.
            .route(
                "/help-text",
                web::post().to(help_text::get_help_text_endpoint),
            )
            .route(
                "/doc-link",
                web::post().to(doc_link::get_doc_link_endpoint),
            )
            // Registry endpoints.
            .route(
                "/registry/packages",
                web::get().to(registry::packages::get_packages_endpoint),
            )
            // Environment endpoints.
            .route("/env", web::get().to(env::get_env_endpoint)),
    );
}
