//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::sync::{Arc, RwLock};

use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

use ten_rust::pkg_info::{pkg_type::PkgType, PkgInfo};

use crate::designer::common::{
    get_designer_api_cmd_likes_from_pkg, get_designer_api_data_likes_from_pkg,
    get_designer_property_hashmap_from_pkg,
};
use crate::designer::graphs::nodes::DesignerApi;
use crate::designer::{
    response::{ApiResponse, ErrorResponse, Status},
    DesignerState,
};

#[derive(Serialize, Deserialize)]
pub struct GetAppAddonsRequestPayload {
    pub base_dir: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub addon_type: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub addon_name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct GetAppAddonsSingleResponseData {
    #[serde(rename = "type")]
    pub addon_type: String,

    #[serde(rename = "name")]
    pub addon_name: String,

    pub url: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub api: Option<DesignerApi>,
}

fn convert_pkg_info_to_addon(
    pkg_info_with_src: &PkgInfo,
) -> GetAppAddonsSingleResponseData {
    GetAppAddonsSingleResponseData {
        addon_type: pkg_info_with_src.manifest.as_ref().map_or_else(
            || PkgType::Extension.to_string(),
            |m| m.type_and_name.pkg_type.to_string(),
        ),
        addon_name: pkg_info_with_src.manifest.as_ref().map_or_else(
            || "unknown".to_string(),
            |m| m.type_and_name.name.clone(),
        ),
        url: pkg_info_with_src.url.clone(),
        api: pkg_info_with_src
            .manifest
            .as_ref()
            .map(|manifest| DesignerApi {
                property: manifest
                    .api
                    .as_ref()
                    .and_then(|api| api.property.as_ref())
                    .map(|prop| {
                        get_designer_property_hashmap_from_pkg(prop.clone())
                    }),

                cmd_in: manifest
                    .api
                    .as_ref()
                    .and_then(|api| api.cmd_in.as_ref())
                    .map(|cmd| {
                        get_designer_api_cmd_likes_from_pkg(cmd.clone())
                    }),

                cmd_out: manifest
                    .api
                    .as_ref()
                    .and_then(|api| api.cmd_out.as_ref())
                    .map(|cmd| {
                        get_designer_api_cmd_likes_from_pkg(cmd.clone())
                    }),

                data_in: manifest
                    .api
                    .as_ref()
                    .and_then(|api| api.data_in.as_ref())
                    .map(|data| {
                        get_designer_api_data_likes_from_pkg(data.clone())
                    }),

                data_out: manifest
                    .api
                    .as_ref()
                    .and_then(|api| api.data_out.as_ref())
                    .map(|data| {
                        get_designer_api_data_likes_from_pkg(data.clone())
                    }),

                audio_frame_in: manifest
                    .api
                    .as_ref()
                    .and_then(|api| api.audio_frame_in.as_ref())
                    .map(|data| {
                        get_designer_api_data_likes_from_pkg(data.clone())
                    }),

                audio_frame_out: manifest
                    .api
                    .as_ref()
                    .and_then(|api| api.audio_frame_out.as_ref())
                    .map(|data| {
                        get_designer_api_data_likes_from_pkg(data.clone())
                    }),

                video_frame_in: manifest
                    .api
                    .as_ref()
                    .and_then(|api| api.video_frame_in.as_ref())
                    .map(|data| {
                        get_designer_api_data_likes_from_pkg(data.clone())
                    }),

                video_frame_out: manifest
                    .api
                    .as_ref()
                    .and_then(|api| api.video_frame_out.as_ref())
                    .map(|data| {
                        get_designer_api_data_likes_from_pkg(data.clone())
                    }),
            }),
    }
}

pub async fn get_app_addons_endpoint(
    request_payload: web::Json<GetAppAddonsRequestPayload>,
    state: web::Data<Arc<RwLock<DesignerState>>>,
) -> Result<impl Responder, actix_web::Error> {
    let state_read = state.read().unwrap();

    // Check if base_dir exists in pkgs_cache.
    if request_payload.base_dir.is_empty()
        || !state_read
            .pkgs_cache
            .contains_key(&request_payload.base_dir)
    {
        let error_response = ErrorResponse {
            status: Status::Fail,
            message: "Base directory not found or not specified".to_string(),
            error: None,
        };

        return Ok(HttpResponse::NotFound().json(error_response));
    }

    let mut all_addons: Vec<GetAppAddonsSingleResponseData> = Vec::new();

    // Get the BaseDirPkgInfo and extract only the requested packages from it.
    if let Some(base_dir_pkg_info) =
        state_read.pkgs_cache.get(&request_payload.base_dir)
    {
        let addon_type_filter = request_payload.addon_type.as_deref();

        // Only process these packages if no addon_type filter is specified or
        // if it matches "extension"
        if addon_type_filter.is_none() || addon_type_filter == Some("extension")
        {
            // Extract extension packages if they exist.
            if let Some(extensions) = &base_dir_pkg_info.extension_pkg_info {
                for ext in extensions {
                    all_addons.push(convert_pkg_info_to_addon(ext));
                }
            }
        }

        // Only process these packages if no addon_type filter is specified or
        // if it matches "protocol"
        if addon_type_filter.is_none() || addon_type_filter == Some("protocol")
        {
            // Extract protocol packages if they exist.
            if let Some(protocols) = &base_dir_pkg_info.protocol_pkg_info {
                for protocol in protocols {
                    all_addons.push(convert_pkg_info_to_addon(protocol));
                }
            }
        }

        // Only process these packages if no addon_type filter is specified or
        // if it matches "addon_loader".
        if addon_type_filter.is_none()
            || addon_type_filter == Some("addon_loader")
        {
            // Extract addon loader packages if they exist.
            if let Some(addon_loaders) =
                &base_dir_pkg_info.addon_loader_pkg_info
            {
                for loader in addon_loaders {
                    all_addons.push(convert_pkg_info_to_addon(loader));
                }
            }
        }

        // Only process these packages if no addon_type filter is specified or
        // if it matches "system".
        if addon_type_filter.is_none() || addon_type_filter == Some("system") {
            // Extract system packages if they exist.
            if let Some(systems) = &base_dir_pkg_info.system_pkg_info {
                for system in systems {
                    all_addons.push(convert_pkg_info_to_addon(system));
                }
            }
        }
    }

    // Filter by addon_name if provided.
    if let Some(addon_name) = &request_payload.addon_name {
        all_addons.retain(|addon| &addon.addon_name == addon_name);
    }

    // Return success response even if all_addons is empty.
    let response = ApiResponse {
        status: Status::Ok,
        data: all_addons,
        meta: None,
    };

    Ok(HttpResponse::Ok().json(response))
}
