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

use ten_rust::graph::node::GraphNode;
use ten_rust::pkg_info::pkg_type_and_name::PkgTypeAndName;
use ten_rust::pkg_info::predefined_graphs::extension::get_pkg_info_for_extension;
use ten_rust::pkg_info::{
    pkg_type::PkgType,
    predefined_graphs::extension::get_extension_nodes_in_graph,
};

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

    if let Some(all_pkgs) =
        &state_read.pkgs_cache.get(&request_payload.base_dir)
    {
        let graph_name = &request_payload.graph_name;

        let extension_graph_nodes =
            match get_extension_nodes_in_graph(graph_name, all_pkgs) {
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
            let pkg_info =
                get_pkg_info_for_extension(extension_graph_node, all_pkgs);
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use actix_web::{test, App};
    use serde_json::json;

    use super::*;
    use crate::{
        config::TmanConfig,
        constants::TEST_DIR,
        designer::{
            graphs::nodes::{
                DesignerApiCmdLike, DesignerApiDataLike,
                DesignerPropertyAttributes,
            },
            mock::inject_all_pkgs_for_mock,
        },
        output::TmanOutputCli,
    };

    #[actix_web::test]
    async fn test_get_extensions_success() {
        let mut designer_state = DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
        };

        let all_pkgs_json = vec![
            (
                include_str!("../test_data_embed/app_manifest.json")
                    .to_string(),
                include_str!("../test_data_embed/app_property.json")
                    .to_string(),
            ),
            (
                include_str!(
                    "../test_data_embed/extension_addon_1_manifest.json"
                )
                .to_string(),
                "{}".to_string(),
            ),
            (
                include_str!(
                    "../test_data_embed/extension_addon_2_manifest.json"
                )
                .to_string(),
                "{}".to_string(),
            ),
            (
                include_str!(
                    "../test_data_embed/extension_addon_3_manifest.json"
                )
                .to_string(),
                "{}".to_string(),
            ),
        ];

        let inject_ret = inject_all_pkgs_for_mock(
            TEST_DIR,
            &mut designer_state.pkgs_cache,
            all_pkgs_json,
        );
        assert!(inject_ret.is_ok());

        let designer_state = Arc::new(RwLock::new(designer_state));

        let app = test::init_service(
            App::new().app_data(web::Data::new(designer_state)).route(
                "/api/designer/v1/graphs/nodes",
                web::post().to(get_graph_nodes_endpoint),
            ),
        )
        .await;

        let request_payload = GetGraphNodesRequestPayload {
            base_dir: TEST_DIR.to_string(),
            graph_name: "default".to_string(),
        };

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/graphs/nodes")
            .set_json(request_payload)
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();

        let extensions: ApiResponse<Vec<GraphNodesSingleResponseData>> =
            serde_json::from_str(body_str).unwrap();

        // Create the expected Version struct
        let expected_extensions = vec![
            GraphNodesSingleResponseData {
                addon: "extension_addon_1".to_string(),
                name: "extension_1".to_string(),
                extension_group: Some("extension_group_1".to_string()),
                app: None,
                api: Some(DesignerApi {
                    property: None,
                    cmd_in: None,
                    cmd_out: Some(vec![
                        DesignerApiCmdLike {
                            name: "test_cmd".to_string(),
                            property: Some({
                                let mut map = HashMap::new();
                                map.insert(
                                    "test_property".to_string(),
                                    DesignerPropertyAttributes {
                                        prop_type: "int8".to_string(),
                                    },
                                );
                                map
                            }),
                            required: None,
                            result: None,
                        },
                        DesignerApiCmdLike {
                            name: "has_required".to_string(),
                            property: Some({
                                let mut map = HashMap::new();
                                map.insert(
                                    "foo".to_string(),
                                    DesignerPropertyAttributes {
                                        prop_type: "string".to_string(),
                                    },
                                );
                                map
                            }),
                            required: Some(vec!["foo".to_string()]),
                            result: None,
                        },
                        DesignerApiCmdLike {
                            name: "has_required_mismatch".to_string(),
                            property: Some({
                                let mut map = HashMap::new();
                                map.insert(
                                    "foo".to_string(),
                                    DesignerPropertyAttributes {
                                        prop_type: "string".to_string(),
                                    },
                                );
                                map
                            }),
                            required: Some(vec!["foo".to_string()]),
                            result: None,
                        },
                    ]),
                    data_in: None,
                    data_out: None,
                    audio_frame_in: None,
                    audio_frame_out: None,
                    video_frame_in: None,
                    video_frame_out: None,
                }),
                property: None,
                is_installed: true,
            },
            GraphNodesSingleResponseData {
                addon: "extension_addon_2".to_string(),
                name: "extension_2".to_string(),
                extension_group: Some("extension_group_1".to_string()),
                app: None,
                api: Some(DesignerApi {
                    property: None,
                    cmd_in: Some(vec![
                        DesignerApiCmdLike {
                            name: "test_cmd".to_string(),
                            property: Some({
                                let mut map = HashMap::new();
                                map.insert(
                                    "test_property".to_string(),
                                    DesignerPropertyAttributes {
                                        prop_type: "int32".to_string(),
                                    },
                                );
                                map
                            }),
                            required: None,
                            result: None,
                        },
                        DesignerApiCmdLike {
                            name: "another_test_cmd".to_string(),
                            property: Some({
                                let mut map = HashMap::new();
                                map.insert(
                                    "test_property".to_string(),
                                    DesignerPropertyAttributes {
                                        prop_type: "int8".to_string(),
                                    },
                                );
                                map
                            }),
                            required: None,
                            result: None,
                        },
                        DesignerApiCmdLike {
                            name: "has_required".to_string(),
                            property: Some({
                                let mut map = HashMap::new();
                                map.insert(
                                    "foo".to_string(),
                                    DesignerPropertyAttributes {
                                        prop_type: "string".to_string(),
                                    },
                                );
                                map
                            }),
                            required: Some(vec!["foo".to_string()]),
                            result: None,
                        },
                        DesignerApiCmdLike {
                            name: "has_required_mismatch".to_string(),
                            property: Some({
                                let mut map = HashMap::new();
                                map.insert(
                                    "foo".to_string(),
                                    DesignerPropertyAttributes {
                                        prop_type: "string".to_string(),
                                    },
                                );
                                map
                            }),
                            required: None,
                            result: None,
                        },
                    ]),
                    cmd_out: None,
                    data_in: None,
                    data_out: Some(vec![DesignerApiDataLike {
                        name: "data_has_required".to_string(),
                        property: Some({
                            let mut map = HashMap::new();
                            map.insert(
                                "foo".to_string(),
                                DesignerPropertyAttributes {
                                    prop_type: "int8".to_string(),
                                },
                            );
                            map
                        }),
                        required: Some(vec!["foo".to_string()]),
                    }]),
                    audio_frame_in: None,
                    audio_frame_out: None,
                    video_frame_in: None,
                    video_frame_out: None,
                }),
                property: Some(json!({
                    "a": 1
                })),
                is_installed: true,
            },
            GraphNodesSingleResponseData {
                addon: "extension_addon_3".to_string(),
                name: "extension_3".to_string(),
                extension_group: Some("extension_group_1".to_string()),
                app: None,
                api: Some(DesignerApi {
                    property: None,
                    cmd_in: Some(vec![DesignerApiCmdLike {
                        name: "test_cmd".to_string(),
                        property: Some({
                            let mut map = HashMap::new();
                            map.insert(
                                "test_property".to_string(),
                                DesignerPropertyAttributes {
                                    prop_type: "string".to_string(),
                                },
                            );
                            map
                        }),
                        required: None,
                        result: None,
                    }]),
                    cmd_out: None,
                    data_in: Some(vec![DesignerApiDataLike {
                        name: "data_has_required".to_string(),
                        property: Some({
                            let mut map = HashMap::new();
                            map.insert(
                                "foo".to_string(),
                                DesignerPropertyAttributes {
                                    prop_type: "int8".to_string(),
                                },
                            );
                            map
                        }),
                        required: Some(vec!["foo".to_string()]),
                    }]),
                    data_out: None,
                    audio_frame_in: None,
                    audio_frame_out: None,
                    video_frame_in: None,
                    video_frame_out: None,
                }),
                property: None,
                is_installed: true,
            },
        ];

        assert_eq!(extensions.data, expected_extensions);
        assert!(!extensions.data.is_empty());

        let json: ApiResponse<Vec<GraphNodesSingleResponseData>> =
            serde_json::from_str(body_str).unwrap();
        let pretty_json = serde_json::to_string_pretty(&json).unwrap();
        println!("Response body: {}", pretty_json);
    }

    #[actix_web::test]
    async fn test_get_extensions_no_graph() {
        let designer_state = Arc::new(RwLock::new(DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
        }));

        let app = test::init_service(
            App::new().app_data(web::Data::new(designer_state)).route(
                "/api/designer/v1/graphs/nodes",
                web::post().to(get_graph_nodes_endpoint),
            ),
        )
        .await;

        let request_payload = GetGraphNodesRequestPayload {
            base_dir: TEST_DIR.to_string(),
            graph_name: "no_existing_graph".to_string(),
        };

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/graphs/nodes")
            .set_json(request_payload)
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_client_error());

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();

        let error: ErrorResponse = serde_json::from_str(body_str).unwrap();

        assert!(error.message.contains("Package information is missing"));
    }

    #[actix_web::test]
    async fn test_get_extensions_has_wrong_addon() {
        let mut designer_state = DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
        };

        let all_pkgs_json = vec![
            (
                include_str!("../test_data_embed/app_manifest.json")
                    .to_string(),
                include_str!("../test_data_embed/app_property.json")
                    .to_string(),
            ),
            (
                include_str!(
                    "../test_data_embed/extension_addon_1_manifest.json"
                )
                .to_string(),
                "{}".to_string(),
            ),
            (
                include_str!(
                    "../test_data_embed/extension_addon_2_manifest.json"
                )
                .to_string(),
                "{}".to_string(),
            ),
            (
                include_str!(
                    "../test_data_embed/extension_addon_3_manifest.json"
                )
                .to_string(),
                "{}".to_string(),
            ),
        ];

        let inject_ret = inject_all_pkgs_for_mock(
            TEST_DIR,
            &mut designer_state.pkgs_cache,
            all_pkgs_json,
        );
        assert!(inject_ret.is_ok());

        let designer_state = Arc::new(RwLock::new(designer_state));

        let app = test::init_service(
            App::new().app_data(web::Data::new(designer_state)).route(
                "/api/designer/v1/graphs/nodes",
                web::post().to(get_graph_nodes_endpoint),
            ),
        )
        .await;

        let request_payload = GetGraphNodesRequestPayload {
            base_dir: TEST_DIR.to_string(),
            graph_name: "addon_not_found".to_string(),
        };

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/graphs/nodes")
            .set_json(request_payload)
            .to_request();

        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();

        let json: ApiResponse<Vec<GraphNodesSingleResponseData>> =
            serde_json::from_str(body_str).unwrap();
        let pretty_json = serde_json::to_string_pretty(&json).unwrap();
        println!("Response body: {}", pretty_json);
    }
}
