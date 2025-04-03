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
struct GetAppAddonsSingleResponseData {
    #[serde(rename = "type")]
    addon_type: String,

    #[serde(rename = "name")]
    addon_name: String,

    url: String,

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

    // Get the BaseDirPkgInfo and extract all packages from it.
    if let Some(base_dir_pkg_info) =
        state_read.pkgs_cache.get(&request_payload.base_dir)
    {
        // Extract app package if it exists.
        if let Some(app_pkg) = &base_dir_pkg_info.app_pkg_info {
            all_addons.push(convert_pkg_info_to_addon(app_pkg));
        }

        // Extract extension packages if they exist.
        if let Some(extensions) = &base_dir_pkg_info.extension_pkg_info {
            for ext in extensions {
                all_addons.push(convert_pkg_info_to_addon(ext));
            }
        }

        // Extract protocol packages if they exist.
        if let Some(protocols) = &base_dir_pkg_info.protocol_pkg_info {
            for protocol in protocols {
                all_addons.push(convert_pkg_info_to_addon(protocol));
            }
        }

        // Extract addon loader packages if they exist.
        if let Some(addon_loaders) = &base_dir_pkg_info.addon_loader_pkg_info {
            for loader in addon_loaders {
                all_addons.push(convert_pkg_info_to_addon(loader));
            }
        }

        // Extract system packages if they exist.
        if let Some(systems) = &base_dir_pkg_info.system_pkg_info {
            for system in systems {
                all_addons.push(convert_pkg_info_to_addon(system));
            }
        }
    }

    all_addons.retain(|addon| addon.addon_type != PkgType::App.to_string());

    // Filter by addon_type if provided.
    if let Some(addon_type) = &request_payload.addon_type {
        all_addons.retain(|addon| &addon.addon_type == addon_type);
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

#[cfg(test)]
mod tests {
    use crate::{
        config::TmanConfig,
        constants::TEST_DIR,
        designer::{mock::inject_all_pkgs_for_mock, DesignerState},
        output::TmanOutputCli,
    };
    use actix_web::{test, App};
    use std::collections::HashMap;
    use std::sync::{Arc, RwLock};

    use super::*;
    use crate::designer::graphs::nodes::{
        DesignerApi, DesignerApiCmdLike, DesignerApiDataLike,
        DesignerPropertyAttributes,
    };

    #[actix_web::test]
    async fn test_get_addons() {
        let mut designer_state = DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
        };

        let all_pkgs_json = vec![
            (
                include_str!("test_data_embed/app_manifest.json").to_string(),
                include_str!("test_data_embed/app_property.json").to_string(),
            ),
            (
                include_str!("test_data_embed/extension_addon_1_manifest.json")
                    .to_string(),
                "{}".to_string(),
            ),
            (
                include_str!("test_data_embed/extension_addon_2_manifest.json")
                    .to_string(),
                "{}".to_string(),
            ),
            (
                include_str!("test_data_embed/extension_addon_3_manifest.json")
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
                "/api/designer/v1/addons",
                web::post().to(get_app_addons_endpoint),
            ),
        )
        .await;

        let request_payload = GetAppAddonsRequestPayload {
            base_dir: TEST_DIR.to_string(),
            addon_name: None,
            addon_type: None,
        };

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/addons")
            .set_json(request_payload)
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();

        let addons: ApiResponse<Vec<GetAppAddonsSingleResponseData>> =
            serde_json::from_str(body_str).unwrap();

        let expected_addons = vec![
            GetAppAddonsSingleResponseData {
                addon_type: "extension".to_string(),
                addon_name: "extension_addon_1".to_string(),
                url: "".to_string(),
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
            },
            GetAppAddonsSingleResponseData {
                addon_type: "extension".to_string(),
                addon_name: "extension_addon_2".to_string(),
                url: "".to_string(),
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
            },
            GetAppAddonsSingleResponseData {
                addon_type: "extension".to_string(),
                addon_name: "extension_addon_3".to_string(),
                url: "".to_string(),
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
            },
        ];

        assert_eq!(addons.data, expected_addons);

        let json: ApiResponse<Vec<GetAppAddonsSingleResponseData>> =
            serde_json::from_str(body_str).unwrap();
        let pretty_json = serde_json::to_string_pretty(&json).unwrap();

        println!("Response body: {}", pretty_json);
    }

    #[actix_web::test]
    async fn test_get_addon_by_name() {
        let mut designer_state = DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
        };

        let all_pkgs_json = vec![
            (
                include_str!("test_data_embed/app_manifest.json").to_string(),
                include_str!("test_data_embed/app_property.json").to_string(),
            ),
            (
                include_str!("test_data_embed/extension_addon_1_manifest.json")
                    .to_string(),
                "{}".to_string(),
            ),
            (
                include_str!("test_data_embed/extension_addon_2_manifest.json")
                    .to_string(),
                "{}".to_string(),
            ),
            (
                include_str!("test_data_embed/extension_addon_3_manifest.json")
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
            App::new()
                .app_data(web::Data::new(designer_state.clone()))
                .route(
                    "/api/designer/v1/addons",
                    web::post().to(get_app_addons_endpoint),
                ),
        )
        .await;

        let request_payload = GetAppAddonsRequestPayload {
            base_dir: TEST_DIR.to_string(),
            addon_name: Some("extension_addon_1".to_string()),
            addon_type: None,
        };

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/addons")
            .set_json(request_payload)
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();

        let addons: ApiResponse<Vec<GetAppAddonsSingleResponseData>> =
            serde_json::from_str(body_str).unwrap();

        let expected_addons = vec![GetAppAddonsSingleResponseData {
            addon_type: "extension".to_string(),
            addon_name: "extension_addon_1".to_string(),
            url: "".to_string(),
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
        }];

        assert_eq!(addons.data, expected_addons);

        // Test non-existent addon should return empty array with success
        // status.
        let request_payload = GetAppAddonsRequestPayload {
            base_dir: TEST_DIR.to_string(),
            addon_name: Some("non_existent_addon".to_string()),
            addon_type: None,
        };

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/addons")
            .set_json(request_payload)
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();

        let addons: ApiResponse<Vec<GetAppAddonsSingleResponseData>> =
            serde_json::from_str(body_str).unwrap();

        assert_eq!(addons.status, Status::Ok);
        assert!(addons.data.is_empty());
    }

    #[actix_web::test]
    async fn test_get_addon_by_type() {
        let mut designer_state = DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
        };

        let all_pkgs_json = vec![
            (
                include_str!("test_data_embed/app_manifest.json").to_string(),
                include_str!("test_data_embed/app_property.json").to_string(),
            ),
            (
                include_str!("test_data_embed/extension_addon_1_manifest.json")
                    .to_string(),
                "{}".to_string(),
            ),
            (
                include_str!("test_data_embed/extension_addon_2_manifest.json")
                    .to_string(),
                "{}".to_string(),
            ),
            (
                include_str!("test_data_embed/extension_addon_3_manifest.json")
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
            App::new()
                .app_data(web::Data::new(designer_state.clone()))
                .route(
                    "/api/designer/v1/addons",
                    web::post().to(get_app_addons_endpoint),
                ),
        )
        .await;

        // Test filtering by addon_type
        let request_payload = GetAppAddonsRequestPayload {
            base_dir: TEST_DIR.to_string(),
            addon_name: None,
            addon_type: Some("extension".to_string()),
        };

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/addons")
            .set_json(request_payload)
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();

        let addons: ApiResponse<Vec<GetAppAddonsSingleResponseData>> =
            serde_json::from_str(body_str).unwrap();

        // Verify that all returned addons have the correct type
        for addon in &addons.data {
            assert_eq!(addon.addon_type, "extension");
        }

        // Test filtering by both addon_type and addon_name
        let request_payload = GetAppAddonsRequestPayload {
            base_dir: TEST_DIR.to_string(),
            addon_name: Some("extension_addon_1".to_string()),
            addon_type: Some("extension".to_string()),
        };

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/addons")
            .set_json(request_payload)
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();

        let addons: ApiResponse<Vec<GetAppAddonsSingleResponseData>> =
            serde_json::from_str(body_str).unwrap();

        assert_eq!(addons.data[0].addon_type, "extension");
        assert_eq!(addons.data[0].addon_name, "extension_addon_1");
    }

    #[actix_web::test]
    async fn test_get_addon_invalid_base_dir() {
        let designer_state = DesignerState {
            tman_config: Arc::new(TmanConfig::default()),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
        };

        let designer_state = Arc::new(RwLock::new(designer_state));

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(designer_state.clone()))
                .route(
                    "/api/designer/v1/addons",
                    web::post().to(get_app_addons_endpoint),
                ),
        )
        .await;

        // Test with non-existent base_dir.
        let request_payload = GetAppAddonsRequestPayload {
            base_dir: "non_existent_dir".to_string(),
            addon_name: None,
            addon_type: None,
        };

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/addons")
            .set_json(request_payload)
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), actix_web::http::StatusCode::NOT_FOUND);

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();

        let error_response: ErrorResponse =
            serde_json::from_str(body_str).unwrap();
        assert_eq!(
            error_response.message,
            "Base directory not found or not specified"
        );

        // Test with empty base_dir.
        let request_payload = GetAppAddonsRequestPayload {
            base_dir: "".to_string(),
            addon_name: None,
            addon_type: None,
        };

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/addons")
            .set_json(request_payload)
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), actix_web::http::StatusCode::NOT_FOUND);

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();

        let error_response: ErrorResponse =
            serde_json::from_str(body_str).unwrap();
        assert_eq!(
            error_response.message,
            "Base directory not found or not specified"
        );
    }
}
