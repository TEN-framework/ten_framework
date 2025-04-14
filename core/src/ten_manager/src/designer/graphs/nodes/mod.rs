//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
pub mod add;
pub mod delete;
pub mod get;
pub mod property;
pub mod validate;

use std::collections::HashMap;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use ten_rust::graph::graph_info::GraphInfo;
use ten_rust::graph::node::GraphNode;
use ten_rust::pkg_info::manifest::api::ManifestApiMsg;
use ten_rust::pkg_info::manifest::api::{
    ManifestApiCmdResult, ManifestApiPropertyAttributes,
};
use ten_rust::pkg_info::value_type::ValueType;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct DesignerApi {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub property: Option<HashMap<String, DesignerPropertyAttributes>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cmd_in: Option<Vec<DesignerApiMsg>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cmd_out: Option<Vec<DesignerApiMsg>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_in: Option<Vec<DesignerApiMsg>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_out: Option<Vec<DesignerApiMsg>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_frame_in: Option<Vec<DesignerApiMsg>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_frame_out: Option<Vec<DesignerApiMsg>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub video_frame_in: Option<Vec<DesignerApiMsg>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub video_frame_out: Option<Vec<DesignerApiMsg>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct DesignerPropertyAttributes {
    #[serde(rename = "type")]
    pub prop_type: ValueType,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Box<DesignerPropertyAttributes>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<HashMap<String, DesignerPropertyAttributes>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,
}

impl From<ManifestApiPropertyAttributes> for DesignerPropertyAttributes {
    fn from(api_property: ManifestApiPropertyAttributes) -> Self {
        DesignerPropertyAttributes {
            prop_type: api_property.prop_type,
            items: api_property.items.map(|items| Box::new((*items).into())),
            properties: api_property.properties.map(|props| {
                props.into_iter().map(|(k, v)| (k, v.into())).collect()
            }),
            required: api_property.required,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct DesignerCmdResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub property: Option<HashMap<String, DesignerPropertyAttributes>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,
}

impl From<ManifestApiCmdResult> for DesignerCmdResult {
    fn from(cmd_result: ManifestApiCmdResult) -> Self {
        DesignerCmdResult {
            property: cmd_result.property.map(|prop| {
                prop.into_iter().map(|(k, v)| (k, v.into())).collect()
            }),
            required: cmd_result
                .required
                .as_ref()
                .filter(|req| !req.is_empty())
                .cloned(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct DesignerApiMsg {
    pub name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub property: Option<HashMap<String, DesignerPropertyAttributes>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<DesignerCmdResult>,
}

impl From<ManifestApiMsg> for DesignerApiMsg {
    fn from(api_cmd_like: ManifestApiMsg) -> Self {
        DesignerApiMsg {
            name: api_cmd_like.name,
            property: api_cmd_like
                .property
                .as_ref()
                .filter(|p| !p.is_empty())
                .map(|p| {
                    p.iter()
                        .map(|(k, v)| (k.clone(), v.clone().into()))
                        .collect()
                }),
            required: api_cmd_like
                .required
                .as_ref()
                .filter(|req| !req.is_empty())
                .cloned(),
            result: api_cmd_like.result.as_ref().cloned().map(Into::into),
        }
    }
}

/// Retrieves all extension nodes from a specified graph.
pub fn get_extension_nodes_in_graph<'a>(
    graph_id: &Uuid,
    graphs_cache: &'a HashMap<Uuid, GraphInfo>,
) -> Result<&'a Vec<GraphNode>> {
    // Look for the graph by ID in the graphs_cache.
    if let Some(graph_info) = graphs_cache.get(graph_id) {
        // Collect all extension nodes from the graph.
        Ok(&graph_info.graph.nodes)
    } else {
        Err(anyhow::anyhow!(
            "Graph with ID '{}' not found in graph caches",
            graph_id
        ))
    }
}
