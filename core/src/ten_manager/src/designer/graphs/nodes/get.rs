//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::sync::{Arc, RwLock};

use actix_web::{web, HttpResponse, Responder};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use ten_rust::graph::extension::{
    get_extension_nodes_in_graph, get_pkg_info_for_extension,
};
use ten_rust::graph::node::GraphNode;
use ten_rust::pkg_info::pkg_type::PkgType;
use ten_rust::pkg_info::pkg_type_and_name::PkgTypeAndName;

use crate::designer::common::{
    get_designer_api_cmd_likes_from_pkg, get_designer_api_data_likes_from_pkg,
    get_designer_property_hashmap_from_pkg,
};
use crate::designer::response::{ApiResponse, ErrorResponse, Status};
use crate::designer::DesignerState;

use super::DesignerApi;

#[derive(Serialize, Deserialize)]
pub struct GetGraphNodesRequestPayload {
    pub base_dir: String,
    pub graph_name: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct GraphNodesSingleResponseData {
    pub addon: String,
    pub name: String,

    // The app which this extension belongs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app: Option<String>,

    // The extension group which this extension belongs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extension_group: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub api: Option<DesignerApi>,

    pub property: Option<serde_json::Value>,

    /// This indicates that the extension has been installed under the
    /// `ten_packages/` directory.
    pub is_installed: bool,
}

impl TryFrom<GraphNode> for GraphNodesSingleResponseData {
    type Error = anyhow::Error;

    fn try_from(node: GraphNode) -> Result<Self, Self::Error> {
        if node.type_and_name.pkg_type != PkgType::Extension {
            return Err(anyhow!(
                "Graph node '{}' is not of type 'extension'",
                node.type_and_name.name
            ));
        }

        Ok(GraphNodesSingleResponseData {
            addon: node.addon,
            name: node.type_and_name.name,
            extension_group: node.extension_group,
            app: node.app,
            api: None,
            property: node.property,
            is_installed: false,
        })
    }
}

impl From<GraphNodesSingleResponseData> for GraphNode {
    fn from(designer_extension: GraphNodesSingleResponseData) -> Self {
        GraphNode {
            type_and_name: PkgTypeAndName {
                pkg_type: PkgType::Extension,
                name: designer_extension.name,
            },
            addon: designer_extension.addon,
            extension_group: designer_extension.extension_group,
            app: designer_extension.app,
            property: designer_extension.property,
        }
    }
}

/// Retrieve graph nodes for a specific graph.
///
/// # Parameters
/// - `state`: The state of the designer.
/// - `path`: The name of the graph.
pub async fn get_graph_nodes_endpoint(
    request_payload: web::Json<GetGraphNodesRequestPayload>,
    state: web::Data<Arc<RwLock<DesignerState>>>,
) -> Result<impl Responder, actix_web::Error> {
    let state_read = state.read().unwrap();

    if let Some(base_dir_pkg_info) =
        &state_read.pkgs_cache.get(&request_payload.base_dir)
    {
        if base_dir_pkg_info.app_pkg_info.is_none() {
            let error_response = ErrorResponse {
                status: Status::Fail,
                message: "Application package information is missing"
                    .to_string(),
                error: None,
            };
            return Ok(HttpResponse::NotFound().json(error_response));
        }

        // Get the app package directly for finding the graph.
        let app_pkg = base_dir_pkg_info.app_pkg_info.as_ref().unwrap();
        // Get extension package information directly, if available
        let extensions_slice = base_dir_pkg_info.get_extensions();

        let graph_name = &request_payload.graph_name;

        let extension_graph_nodes =
            match get_extension_nodes_in_graph(graph_name, app_pkg) {
                Ok(exts) => exts,
                Err(err) => {
                    let error_response = ErrorResponse::from_error(
                        &err,
                        format!(
                            "Error fetching runtime extensions for graph '{}'",
                            graph_name
                        )
                        .as_str(),
                    );
                    return Ok(HttpResponse::NotFound().json(error_response));
                }
            };

        let mut resp_extensions: Vec<GraphNodesSingleResponseData> = Vec::new();

        for extension_graph_node in &extension_graph_nodes {
            let pkg_info = get_pkg_info_for_extension(
                extension_graph_node,
                extensions_slice,
            );
            if let Some(pkg_info) = pkg_info {
                resp_extensions.push(GraphNodesSingleResponseData {
                    addon: extension_graph_node.addon.clone(),
                    name: extension_graph_node.type_and_name.name.clone(),
                    extension_group: extension_graph_node
                        .extension_group
                        .clone(),
                    app: extension_graph_node.app.clone(),
                    api: pkg_info
                        .manifest
                        .as_ref()
                        .and_then(|manifest| manifest.api.as_ref())
                        .map(|api| DesignerApi {
                            property: api
                                .property
                                .as_ref()
                                .filter(|p| !p.is_empty())
                                .map(|p| {
                                    get_designer_property_hashmap_from_pkg(
                                        p.clone(),
                                    )
                                }),

                            cmd_in: api
                                .cmd_in
                                .as_ref()
                                .filter(|c| !c.is_empty())
                                .map(|c| {
                                    get_designer_api_cmd_likes_from_pkg(
                                        c.clone(),
                                    )
                                }),

                            cmd_out: api
                                .cmd_out
                                .as_ref()
                                .filter(|c| !c.is_empty())
                                .map(|c| {
                                    get_designer_api_cmd_likes_from_pkg(
                                        c.clone(),
                                    )
                                }),

                            data_in: api
                                .data_in
                                .as_ref()
                                .filter(|d| !d.is_empty())
                                .map(|d| {
                                    get_designer_api_data_likes_from_pkg(
                                        d.clone(),
                                    )
                                }),

                            data_out: api
                                .data_out
                                .as_ref()
                                .filter(|d| !d.is_empty())
                                .map(|d| {
                                    get_designer_api_data_likes_from_pkg(
                                        d.clone(),
                                    )
                                }),

                            audio_frame_in: api
                                .audio_frame_in
                                .as_ref()
                                .filter(|d| !d.is_empty())
                                .map(|d| {
                                    get_designer_api_data_likes_from_pkg(
                                        d.clone(),
                                    )
                                }),

                            audio_frame_out: api
                                .audio_frame_out
                                .as_ref()
                                .filter(|d| !d.is_empty())
                                .map(|d| {
                                    get_designer_api_data_likes_from_pkg(
                                        d.clone(),
                                    )
                                }),

                            video_frame_in: api
                                .video_frame_in
                                .as_ref()
                                .filter(|d| !d.is_empty())
                                .map(|d| {
                                    get_designer_api_data_likes_from_pkg(
                                        d.clone(),
                                    )
                                }),

                            video_frame_out: api
                                .video_frame_out
                                .as_ref()
                                .filter(|d| !d.is_empty())
                                .map(|d| {
                                    get_designer_api_data_likes_from_pkg(
                                        d.clone(),
                                    )
                                }),
                        }),
                    property: extension_graph_node.property.clone(),
                    is_installed: true,
                });
            } else {
                match GraphNodesSingleResponseData::try_from(
                    extension_graph_node.clone(),
                ) {
                    Ok(designer_ext) => {
                        resp_extensions.push(designer_ext);
                    }
                    Err(e) => {
                        let error_response = ErrorResponse::from_error(
                            &e,
                            "This graph node's content is not a valid graph node.",
                        );
                        return Ok(
                            HttpResponse::NotFound().json(error_response)
                        );
                    }
                }
            }
        }

        let response = ApiResponse {
            status: Status::Ok,
            data: resp_extensions,
            meta: None,
        };

        Ok(HttpResponse::Ok().json(response))
    } else {
        let error_response = ErrorResponse {
            status: Status::Fail,
            message: "Package information is missing".to_string(),
            error: None,
        };
        Ok(HttpResponse::NotFound().json(error_response))
    }
}
