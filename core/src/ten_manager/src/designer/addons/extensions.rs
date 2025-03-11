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

use crate::designer::{
    app::base_dir::get_base_dir_from_pkgs_cache,
    common::{
        get_designer_api_cmd_likes_from_pkg,
        get_designer_api_data_likes_from_pkg,
        get_designer_property_hashmap_from_pkg,
    },
    graphs::nodes::DesignerApi,
    response::{ApiResponse, ErrorResponse, Status},
    DesignerState,
};

#[derive(Serialize, Deserialize)]
pub struct GetExtensionAddonsRequestPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub base_dir: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub addon_name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct GetExtensionAddonsResponseData {
    #[serde(rename = "name")]
    addon_name: String,

    url: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub api: Option<DesignerApi>,
}

fn map_pkg_to_extension_addon(
    pkg_info_with_src: &PkgInfo,
) -> GetExtensionAddonsResponseData {
    GetExtensionAddonsResponseData {
        addon_name: pkg_info_with_src.basic_info.type_and_name.name.clone(),
        url: pkg_info_with_src.url.clone(),
        api: pkg_info_with_src.api.as_ref().map(|api| DesignerApi {
            property: if api.property.is_empty() {
                None
            } else {
                Some(get_designer_property_hashmap_from_pkg(
                    api.property.clone(),
                ))
            },

            cmd_in: if api.cmd_in.is_empty() {
                None
            } else {
                Some(get_designer_api_cmd_likes_from_pkg(api.cmd_in.clone()))
            },
            cmd_out: if api.cmd_out.is_empty() {
                None
            } else {
                Some(get_designer_api_cmd_likes_from_pkg(api.cmd_out.clone()))
            },

            data_in: if api.data_in.is_empty() {
                None
            } else {
                Some(get_designer_api_data_likes_from_pkg(api.data_in.clone()))
            },
            data_out: if api.data_out.is_empty() {
                None
            } else {
                Some(get_designer_api_data_likes_from_pkg(api.data_out.clone()))
            },

            audio_frame_in: if api.audio_frame_in.is_empty() {
                None
            } else {
                Some(get_designer_api_data_likes_from_pkg(
                    api.audio_frame_in.clone(),
                ))
            },
            audio_frame_out: if api.audio_frame_out.is_empty() {
                None
            } else {
                Some(get_designer_api_data_likes_from_pkg(
                    api.audio_frame_out.clone(),
                ))
            },

            video_frame_in: if api.video_frame_in.is_empty() {
                None
            } else {
                Some(get_designer_api_data_likes_from_pkg(
                    api.video_frame_in.clone(),
                ))
            },
            video_frame_out: if api.video_frame_out.is_empty() {
                None
            } else {
                Some(get_designer_api_data_likes_from_pkg(
                    api.video_frame_out.clone(),
                ))
            },
        }),
    }
}

pub async fn get_extension_addon(
    request_payload: web::Json<GetExtensionAddonsRequestPayload>,
    state: web::Data<Arc<RwLock<DesignerState>>>,
) -> Result<impl Responder, actix_web::Error> {
    let state_read = state.read().unwrap();

    let base_dir = match get_base_dir_from_pkgs_cache(
        request_payload.base_dir.clone(),
        &state_read.pkgs_cache,
    ) {
        Ok(base_dir) => base_dir,
        Err(e) => {
            let error_response = ErrorResponse {
                status: Status::Fail,
                message: e.to_string(),
                error: None,
            };
            return Ok(HttpResponse::BadRequest().json(error_response));
        }
    };

    let all_extensions: Vec<GetExtensionAddonsResponseData> = state_read
        .pkgs_cache
        .get(&base_dir)
        .unwrap()
        .iter()
        .filter(|pkg| {
            pkg.basic_info.type_and_name.pkg_type == PkgType::Extension
        })
        .map(map_pkg_to_extension_addon)
        .collect();

    if let Some(addon_name) = &request_payload.addon_name {
        if let Some(extension) = all_extensions
            .into_iter()
            .find(|ext| &ext.addon_name == addon_name)
        {
            let response = ApiResponse {
                status: Status::Ok,
                data: extension,
                meta: None,
            };
            Ok(HttpResponse::Ok().json(response))
        } else {
            let error_response = ErrorResponse {
                status: Status::Fail,
                message: format!(
                    "Extension addon with name '{}' not found",
                    addon_name
                ),
                error: None,
            };
            Ok(HttpResponse::NotFound().json(error_response))
        }
    } else {
        let response = ApiResponse {
            status: Status::Ok,
            data: all_extensions,
            meta: None,
        };
        Ok(HttpResponse::Ok().json(response))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{
        config::TmanConfig,
        constants::TEST_DIR,
        designer::{
            graphs::nodes::{
                DesignerApiCmdLike, DesignerApiDataLike,
                DesignerPropertyAttributes, DesignerPropertyItem,
            },
            mock::inject_all_pkgs_for_mock,
        },
        output::TmanOutputCli,
    };

    use super::*;
    use actix_web::{test, App};

    #[actix_web::test]
    async fn test_get_extension_addons() {
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
                "/api/designer/v1/addons/extensions",
                web::post().to(get_extension_addon),
            ),
        )
        .await;

        let request_payload = GetExtensionAddonsRequestPayload {
            base_dir: Some(TEST_DIR.to_string()),
            addon_name: None,
        };

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/addons/extensions")
            .set_json(request_payload)
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();

        let addons: ApiResponse<Vec<GetExtensionAddonsResponseData>> =
            serde_json::from_str(body_str).unwrap();

        let expected_addons = vec![
            GetExtensionAddonsResponseData {
                addon_name: "extension_addon_1".to_string(),
                url: "".to_string(),
                api: Some(DesignerApi {
                    property: None,
                    cmd_in: None,
                    cmd_out: Some(vec![
                        DesignerApiCmdLike {
                            name: "test_cmd".to_string(),
                            property: Some(vec![DesignerPropertyItem {
                                name: "test_property".to_string(),
                                attributes: DesignerPropertyAttributes {
                                    prop_type: "int8".to_string(),
                                },
                            }]),
                            required: None,
                            result: None,
                        },
                        DesignerApiCmdLike {
                            name: "has_required".to_string(),
                            property: Some(vec![DesignerPropertyItem {
                                name: "foo".to_string(),
                                attributes: DesignerPropertyAttributes {
                                    prop_type: "string".to_string(),
                                },
                            }]),
                            required: Some(vec!["foo".to_string()]),
                            result: None,
                        },
                        DesignerApiCmdLike {
                            name: "has_required_mismatch".to_string(),
                            property: Some(vec![DesignerPropertyItem {
                                name: "foo".to_string(),
                                attributes: DesignerPropertyAttributes {
                                    prop_type: "string".to_string(),
                                },
                            }]),
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
            GetExtensionAddonsResponseData {
                addon_name: "extension_addon_2".to_string(),
                url: "".to_string(),
                api: Some(DesignerApi {
                    property: None,
                    cmd_in: Some(vec![
                        DesignerApiCmdLike {
                            name: "test_cmd".to_string(),
                            property: Some(vec![DesignerPropertyItem {
                                name: "test_property".to_string(),
                                attributes: DesignerPropertyAttributes {
                                    prop_type: "int32".to_string(),
                                },
                            }]),
                            required: None,
                            result: None,
                        },
                        DesignerApiCmdLike {
                            name: "another_test_cmd".to_string(),
                            property: Some(vec![DesignerPropertyItem {
                                name: "test_property".to_string(),
                                attributes: DesignerPropertyAttributes {
                                    prop_type: "int8".to_string(),
                                },
                            }]),
                            required: None,
                            result: None,
                        },
                        DesignerApiCmdLike {
                            name: "has_required".to_string(),
                            property: Some(vec![DesignerPropertyItem {
                                name: "foo".to_string(),
                                attributes: DesignerPropertyAttributes {
                                    prop_type: "string".to_string(),
                                },
                            }]),
                            required: Some(vec!["foo".to_string()]),
                            result: None,
                        },
                        DesignerApiCmdLike {
                            name: "has_required_mismatch".to_string(),
                            property: Some(vec![DesignerPropertyItem {
                                name: "foo".to_string(),
                                attributes: DesignerPropertyAttributes {
                                    prop_type: "string".to_string(),
                                },
                            }]),
                            required: None,
                            result: None,
                        },
                    ]),
                    cmd_out: None,
                    data_in: None,
                    data_out: Some(vec![DesignerApiDataLike {
                        name: "data_has_required".to_string(),
                        property: Some(vec![DesignerPropertyItem {
                            name: "foo".to_string(),
                            attributes: DesignerPropertyAttributes {
                                prop_type: "int8".to_string(),
                            },
                        }]),
                        required: Some(vec!["foo".to_string()]),
                    }]),
                    audio_frame_in: None,
                    audio_frame_out: None,
                    video_frame_in: None,
                    video_frame_out: None,
                }),
            },
            GetExtensionAddonsResponseData {
                addon_name: "extension_addon_3".to_string(),
                url: "".to_string(),
                api: Some(DesignerApi {
                    property: None,
                    cmd_in: Some(vec![DesignerApiCmdLike {
                        name: "test_cmd".to_string(),
                        property: Some(vec![DesignerPropertyItem {
                            name: "test_property".to_string(),
                            attributes: DesignerPropertyAttributes {
                                prop_type: "string".to_string(),
                            },
                        }]),
                        required: None,
                        result: None,
                    }]),
                    cmd_out: None,
                    data_in: Some(vec![DesignerApiDataLike {
                        name: "data_has_required".to_string(),
                        property: Some(vec![DesignerPropertyItem {
                            name: "foo".to_string(),
                            attributes: DesignerPropertyAttributes {
                                prop_type: "int8".to_string(),
                            },
                        }]),
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

        let json: ApiResponse<Vec<GetExtensionAddonsResponseData>> =
            serde_json::from_str(body_str).unwrap();
        let pretty_json = serde_json::to_string_pretty(&json).unwrap();

        println!("Response body: {}", pretty_json);
    }

    #[actix_web::test]
    async fn test_get_extension_addon_by_name() {
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
                    "/api/designer/v1/addons/extensions",
                    web::post().to(get_extension_addon),
                ),
        )
        .await;

        let request_payload = GetExtensionAddonsRequestPayload {
            base_dir: Some(TEST_DIR.to_string()),
            addon_name: Some("extension_addon_1".to_string()),
        };

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/addons/extensions")
            .set_json(request_payload)
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();

        let addon: ApiResponse<GetExtensionAddonsResponseData> =
            serde_json::from_str(body_str).unwrap();

        let expected_addon = GetExtensionAddonsResponseData {
            addon_name: "extension_addon_1".to_string(),
            url: "".to_string(),
            api: Some(DesignerApi {
                property: None,
                cmd_in: None,
                cmd_out: Some(vec![
                    DesignerApiCmdLike {
                        name: "test_cmd".to_string(),
                        property: Some(vec![DesignerPropertyItem {
                            name: "test_property".to_string(),
                            attributes: DesignerPropertyAttributes {
                                prop_type: "int8".to_string(),
                            },
                        }]),
                        required: None,
                        result: None,
                    },
                    DesignerApiCmdLike {
                        name: "has_required".to_string(),
                        property: Some(vec![DesignerPropertyItem {
                            name: "foo".to_string(),
                            attributes: DesignerPropertyAttributes {
                                prop_type: "string".to_string(),
                            },
                        }]),
                        required: Some(vec!["foo".to_string()]),
                        result: None,
                    },
                    DesignerApiCmdLike {
                        name: "has_required_mismatch".to_string(),
                        property: Some(vec![DesignerPropertyItem {
                            name: "foo".to_string(),
                            attributes: DesignerPropertyAttributes {
                                prop_type: "string".to_string(),
                            },
                        }]),
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
        };

        assert_eq!(addon.data, expected_addon);

        let request_payload = GetExtensionAddonsRequestPayload {
            base_dir: Some(TEST_DIR.to_string()),
            addon_name: Some("non_existent_addon".to_string()),
        };

        let req = test::TestRequest::post()
            .uri("/api/designer/v1/addons/extensions")
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
            "Extension addon with name 'non_existent_addon' not found"
        );
    }
}
