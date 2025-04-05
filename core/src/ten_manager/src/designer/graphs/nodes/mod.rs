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

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use ten_rust::pkg_info::manifest::api::{
    ManifestApiCmdLike, ManifestApiDataLike,
};
use ten_rust::pkg_info::manifest::api::{
    ManifestCmdResult, ManifestPropertyAttributes,
};
use ten_rust::pkg_info::value_type::ValueType;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct DesignerApi {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub property: Option<HashMap<String, DesignerPropertyAttributes>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cmd_in: Option<Vec<DesignerApiCmdLike>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cmd_out: Option<Vec<DesignerApiCmdLike>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_in: Option<Vec<DesignerApiDataLike>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_out: Option<Vec<DesignerApiDataLike>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_frame_in: Option<Vec<DesignerApiDataLike>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_frame_out: Option<Vec<DesignerApiDataLike>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub video_frame_in: Option<Vec<DesignerApiDataLike>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub video_frame_out: Option<Vec<DesignerApiDataLike>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct DesignerPropertyAttributes {
    #[serde(rename = "type")]
    pub prop_type: ValueType,
}

impl From<ManifestPropertyAttributes> for DesignerPropertyAttributes {
    fn from(api_property: ManifestPropertyAttributes) -> Self {
        DesignerPropertyAttributes {
            prop_type: api_property.prop_type,
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

impl From<ManifestCmdResult> for DesignerCmdResult {
    fn from(cmd_result: ManifestCmdResult) -> Self {
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
pub struct DesignerApiCmdLike {
    pub name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub property: Option<HashMap<String, DesignerPropertyAttributes>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<DesignerCmdResult>,
}

impl From<ManifestApiCmdLike> for DesignerApiCmdLike {
    fn from(api_cmd_like: ManifestApiCmdLike) -> Self {
        DesignerApiCmdLike {
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

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct DesignerApiDataLike {
    pub name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub property: Option<HashMap<String, DesignerPropertyAttributes>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,
}

impl From<ManifestApiDataLike> for DesignerApiDataLike {
    fn from(api_data_like: ManifestApiDataLike) -> Self {
        DesignerApiDataLike {
            name: api_data_like.name,
            property: api_data_like
                .property
                .as_ref()
                .filter(|p| !p.is_empty())
                .map(|p| {
                    p.iter()
                        .map(|(k, v)| (k.clone(), v.clone().into()))
                        .collect()
                }),
            required: api_data_like
                .required
                .as_ref()
                .filter(|req| !req.is_empty())
                .cloned(),
        }
    }
}
