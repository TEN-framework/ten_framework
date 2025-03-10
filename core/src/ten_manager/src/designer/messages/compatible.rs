//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::{
    str::FromStr,
    sync::{Arc, RwLock},
};

use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

use ten_rust::pkg_info::{
    message::{MsgDirection, MsgType},
    predefined_graphs::extension::{
        get_compatible_cmd_extension, get_compatible_data_like_msg_extension,
        get_extension, get_extension_nodes_in_graph,
        get_pkg_info_for_extension, CompatibleExtensionAndMsg,
    },
};

use crate::designer::{
    app::base_dir::get_base_dir_from_pkgs_cache,
    response::{ApiResponse, ErrorResponse, Status},
    DesignerState,
};

#[derive(Debug, Deserialize, Serialize)]
pub struct GetCompatibleMsgsRequestPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub base_dir: Option<String>,

    pub app: String,
    pub graph: String,
    pub extension_group: String,
    pub extension: String,
    pub msg_type: String,
    pub msg_direction: String,
    pub msg_name: String,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct GetCompatibleMsgsSingleResponseData {
    pub app: String,
    pub extension_group: String,
    pub extension: String,
    pub msg_type: MsgType,
    pub msg_direction: MsgDirection,
    pub msg_name: String,
}

impl From<CompatibleExtensionAndMsg<'_>>
    for GetCompatibleMsgsSingleResponseData
{
    fn from(compatible: CompatibleExtensionAndMsg) -> Self {
        GetCompatibleMsgsSingleResponseData {
            app: compatible.extension.app.as_ref().unwrap().clone(),
            extension_group: compatible
                .extension
                .extension_group
                .clone()
                .unwrap()
                .clone(),
            extension: compatible.extension.type_and_name.name.clone(),
            msg_type: compatible.msg_type,
            msg_direction: compatible.msg_direction,
            msg_name: compatible.msg_name,
        }
    }
}

pub async fn get_compatible_messages(
    request_payload: web::Json<GetCompatibleMsgsRequestPayload>,
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

    if let Some(all_pkgs) = state_read.pkgs_cache.get(&base_dir) {
        let extensions = match get_extension_nodes_in_graph(
            &request_payload.graph,
            all_pkgs,
        ) {
            Ok(exts) => exts,
            Err(err) => {
                let error_response = ErrorResponse::from_error(
                    &err,
                    format!(
                        "Error fetching runtime extensions for graph '{}'",
                        request_payload.graph
                    )
                    .as_str(),
                );
                return Ok(HttpResponse::NotFound().json(error_response));
            }
        };

        let extension = match get_extension(
            &extensions,
            &request_payload.app,
            &request_payload.extension_group,
            &request_payload.extension,
        ) {
            Ok(ext) => ext,
            Err(err) => {
                let error_response = ErrorResponse::from_error(
                    &err,
                    format!(
                        "Failed to find the extension: {}",
                        request_payload.extension
                    )
                    .as_str(),
                );

                return Ok(HttpResponse::NotFound().json(error_response));
            }
        };

        let msg_ty = match MsgType::from_str(&request_payload.msg_type) {
            Ok(msg_ty) => msg_ty,
            Err(err) => {
                let error_response = ErrorResponse::from_error(
                    &err,
                    format!(
                        "Unsupported message type: {}",
                        request_payload.msg_type
                    )
                    .as_str(),
                );
                return Ok(
                    HttpResponse::InternalServerError().json(error_response)
                );
            }
        };

        let msg_dir =
            match MsgDirection::from_str(&request_payload.msg_direction) {
                Ok(msg_dir) => msg_dir,
                Err(err) => {
                    let error_response = ErrorResponse::from_error(
                        &err,
                        format!(
                            "Unsupported message direction: {}",
                            request_payload.msg_direction
                        )
                        .as_str(),
                    );
                    return Ok(HttpResponse::InternalServerError()
                        .json(error_response));
                }
            };

        let mut desired_msg_dir = msg_dir.clone();
        desired_msg_dir.toggle();

        let pkg_info = get_pkg_info_for_extension(extension, all_pkgs);
        if pkg_info.is_none() {
            let error_response = ErrorResponse::from_error(
                &anyhow::anyhow!("Extension not found"),
                format!(
                    "Error fetching runtime extensions for graph '{}'",
                    request_payload.graph
                )
                .as_str(),
            );
            return Ok(HttpResponse::NotFound().json(error_response));
        }

        let pkg_info = pkg_info.unwrap();

        let compatible_list = match msg_ty {
            MsgType::Cmd => {
                let src_cmd_schema =
                    pkg_info.schema_store.as_ref().and_then(|schema_store| {
                        match msg_dir {
                            MsgDirection::In => schema_store
                                .cmd_in
                                .get(request_payload.msg_name.as_str()),
                            MsgDirection::Out => schema_store
                                .cmd_out
                                .get(request_payload.msg_name.as_str()),
                        }
                    });

                let results = match get_compatible_cmd_extension(
                    &extensions,
                    all_pkgs,
                    &desired_msg_dir,
                    src_cmd_schema,
                    request_payload.msg_name.as_str(),
                ) {
                    Ok(results) => results,
                    Err(err) => {
                        let error_response = ErrorResponse::from_error(
                            &err,
                            format!(
                                "Failed to find compatible cmd/{}:",
                                request_payload.msg_name
                            )
                            .as_str(),
                        );
                        return Ok(
                            HttpResponse::NotFound().json(error_response)
                        );
                    }
                };

                results
            }
            _ => {
                let src_msg_schema =
                    pkg_info.schema_store.as_ref().and_then(|schema_store| {
                        match msg_ty {
                            MsgType::Data => match msg_dir {
                                MsgDirection::In => schema_store
                                    .data_in
                                    .get(request_payload.msg_name.as_str()),
                                MsgDirection::Out => schema_store
                                    .data_out
                                    .get(request_payload.msg_name.as_str()),
                            },
                            MsgType::AudioFrame => match msg_dir {
                                MsgDirection::In => schema_store
                                    .audio_frame_in
                                    .get(request_payload.msg_name.as_str()),
                                MsgDirection::Out => schema_store
                                    .audio_frame_out
                                    .get(request_payload.msg_name.as_str()),
                            },
                            MsgType::VideoFrame => match msg_dir {
                                MsgDirection::In => schema_store
                                    .video_frame_in
                                    .get(request_payload.msg_name.as_str()),
                                MsgDirection::Out => schema_store
                                    .video_frame_out
                                    .get(request_payload.msg_name.as_str()),
                            },
                            _ => panic!("should not happen."),
                        }
                    });

                let results = match get_compatible_data_like_msg_extension(
                    &extensions,
                    all_pkgs,
                    &desired_msg_dir,
                    src_msg_schema,
                    &msg_ty,
                    request_payload.msg_name.clone(),
                ) {
                    Ok(results) => results,
                    Err(err) => {
                        let error_response = ErrorResponse::from_error(
                            &err,
                            format!(
                                "Failed to find compatible cmd/{}:",
                                request_payload.msg_name
                            )
                            .as_str(),
                        );
                        return Ok(
                            HttpResponse::NotFound().json(error_response)
                        );
                    }
                };

                results
            }
        };

        let results: Vec<GetCompatibleMsgsSingleResponseData> =
            compatible_list.into_iter().map(|v| v.into()).collect();

        let response = ApiResponse {
            status: Status::Ok,
            data: results,
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

    use ten_rust::pkg_info::localhost;

    use super::*;
    use crate::{
        config::TmanConfig, constants::TEST_DIR,
        designer::mock::inject_all_pkgs_for_mock, output::TmanOutputCli,
    };

    #[actix_web::test]
    async fn test_get_compatible_messages_success() {
        let mut designer_state = DesignerState {
            tman_config: TmanConfig::default(),
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
                "/api/designer/v1/messages/compatible",
                web::post().to(get_compatible_messages),
            ),
        )
        .await;

        // Define input data.
        let input_data = json!({
          "base_dir": TEST_DIR,

          "app": "localhost",
          "graph": "default",
          "extension_group": "extension_group_1",
          "extension": "extension_1",
          "msg_type": "cmd",
          "msg_direction": "out",
          "msg_name": "test_cmd"
        });

        // Send request to the test server.
        let req = test::TestRequest::post()
            .uri("/api/designer/v1/messages/compatible")
            .set_json(&input_data)
            .to_request();

        // Call the service and get the response.
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();

        let compatibles: ApiResponse<Vec<GetCompatibleMsgsSingleResponseData>> =
            serde_json::from_str(body_str).unwrap();

        let expected_compatibles = vec![GetCompatibleMsgsSingleResponseData {
            app: localhost(),
            extension_group: "extension_group_1".to_string(),
            extension: "extension_2".to_string(),
            msg_type: MsgType::Cmd,
            msg_direction: MsgDirection::In,
            msg_name: "test_cmd".to_string(),
        }];

        assert_eq!(compatibles.data, expected_compatibles);

        println!("Response body: {:?}", compatibles);
    }

    #[actix_web::test]
    async fn test_get_compatible_messages_fail() {
        let mut designer_state = DesignerState {
            tman_config: TmanConfig::default(),
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
                "/api/designer/v1/messages/compatible",
                web::post().to(get_compatible_messages),
            ),
        )
        .await;

        // Define input data.
        let input_data = json!({
          "base_dir": TEST_DIR,

          "app": "localhost",
          "graph": "default",
          "extension_group": "default_extension_group",
          "extension": "default_extension_cpp",
          "msg_type": "data",
          "msg_direction": "in",
          "msg_name": "not_existing_cmd"
        });

        // Send request to the test server.
        let req = test::TestRequest::post()
            .uri("/api/designer/v1/messages/compatible")
            .set_json(&input_data)
            .to_request();

        // Call the service and get the response
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_client_error());
    }

    #[actix_web::test]
    async fn test_get_compatible_messages_cmd_has_required_success() {
        let mut designer_state = DesignerState {
            tman_config: TmanConfig::default(),
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
                "/api/designer/v1/messages/compatible",
                web::post().to(get_compatible_messages),
            ),
        )
        .await;

        // Define input data.
        let input_data = json!({
          "base_dir": TEST_DIR,

          "app": "localhost",
          "graph": "default",
          "extension_group": "extension_group_1",
          "extension": "extension_1",
          "msg_type": "cmd",
          "msg_direction": "out",
          "msg_name": "has_required"
        });

        // Send request to the test server.
        let req = test::TestRequest::post()
            .uri("/api/designer/v1/messages/compatible")
            .set_json(&input_data)
            .to_request();

        // Call the service and get the response.
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();

        let compatibles: ApiResponse<Vec<GetCompatibleMsgsSingleResponseData>> =
            serde_json::from_str(body_str).unwrap();

        let expected_compatibles = vec![GetCompatibleMsgsSingleResponseData {
            app: localhost(),
            extension_group: "extension_group_1".to_string(),
            extension: "extension_2".to_string(),
            msg_type: MsgType::Cmd,
            msg_direction: MsgDirection::In,
            msg_name: "has_required".to_string(),
        }];

        assert_eq!(compatibles.data, expected_compatibles);

        println!("Response body: {:?}", compatibles);
    }

    #[actix_web::test]
    async fn test_get_compatible_messages_has_required_subset() {
        let mut designer_state = DesignerState {
            tman_config: TmanConfig::default(),
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
                "/api/designer/v1/messages/compatible",
                web::post().to(get_compatible_messages),
            ),
        )
        .await;

        // Define input data.
        let input_data = json!({
          "base_dir": TEST_DIR,

          "app": "localhost",
          "graph": "default",
          "extension_group": "extension_group_1",
          "extension": "extension_1",
          "msg_type": "cmd",
          "msg_direction": "out",
          "msg_name": "has_required_mismatch"
        });

        // Send request to the test server.
        let req = test::TestRequest::post()
            .uri("/api/designer/v1/messages/compatible")
            .set_json(&input_data)
            .to_request();

        // Call the service and get the response.
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();

        let compatibles: ApiResponse<Vec<GetCompatibleMsgsSingleResponseData>> =
            serde_json::from_str(body_str).unwrap();

        let expected_compatibles = vec![GetCompatibleMsgsSingleResponseData {
            app: localhost(),
            extension_group: "extension_group_1".to_string(),
            extension: "extension_2".to_string(),
            msg_type: MsgType::Cmd,
            msg_direction: MsgDirection::In,
            msg_name: "has_required_mismatch".to_string(),
        }];

        assert_eq!(compatibles.data, expected_compatibles);
    }

    #[actix_web::test]
    async fn test_get_compatible_messages_cmd_no_property() {
        let mut designer_state = DesignerState {
            tman_config: TmanConfig::default(),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
        };

        // The first item is 'manifest.json', and the second item is
        // 'property.json'.
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
                "/api/designer/v1/messages/compatible",
                web::post().to(get_compatible_messages),
            ),
        )
        .await;

        // Define input data.
        let input_data = json!({
          "base_dir": TEST_DIR,

          "app": "localhost",
          "graph": "default",
          "extension_group": "extension_group_1",
          "extension": "extension_1",
          "msg_type": "cmd",
          "msg_direction": "out",
          "msg_name": "cmd1"
        });

        // Send request to the test server.
        let req = test::TestRequest::post()
            .uri("/api/designer/v1/messages/compatible")
            .set_json(&input_data)
            .to_request();

        // Call the service and get the response.
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();

        let compatibles: ApiResponse<Vec<GetCompatibleMsgsSingleResponseData>> =
            serde_json::from_str(body_str).unwrap();

        let expected_compatibles = vec![
            GetCompatibleMsgsSingleResponseData {
                app: localhost(),
                extension_group: "extension_group_1".to_string(),
                extension: "extension_1".to_string(),
                msg_type: MsgType::Cmd,
                msg_direction: MsgDirection::In,
                msg_name: "cmd1".to_string(),
            },
            GetCompatibleMsgsSingleResponseData {
                app: localhost(),
                extension_group: "extension_group_1".to_string(),
                extension: "extension_2".to_string(),
                msg_type: MsgType::Cmd,
                msg_direction: MsgDirection::In,
                msg_name: "cmd1".to_string(),
            },
        ];

        assert_eq!(compatibles.data, expected_compatibles);

        println!("Response body: {:?}", compatibles);
    }

    #[actix_web::test]
    async fn test_get_compatible_messages_cmd_property_overlap() {
        let mut designer_state = DesignerState {
            tman_config: TmanConfig::default(),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
        };

        // The first item is 'manifest.json', and the second item is
        // 'property.json'.
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
                "/api/designer/v1/messages/compatible",
                web::post().to(get_compatible_messages),
            ),
        )
        .await;

        // Define input data.
        let input_data = json!({
          "base_dir": TEST_DIR,

          "app": "localhost",
          "graph": "default",
          "extension_group": "extension_group_1",
          "extension": "extension_1",
          "msg_type": "cmd",
          "msg_direction": "out",
          "msg_name": "cmd2"
        });

        // Send request to the test server.
        let req = test::TestRequest::post()
            .uri("/api/designer/v1/messages/compatible")
            .set_json(&input_data)
            .to_request();

        // Call the service and get the response.
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();

        let compatibles: ApiResponse<Vec<GetCompatibleMsgsSingleResponseData>> =
            serde_json::from_str(body_str).unwrap();

        let expected_compatibles = vec![GetCompatibleMsgsSingleResponseData {
            app: localhost(),
            extension_group: "extension_group_1".to_string(),
            extension: "extension_2".to_string(),
            msg_type: MsgType::Cmd,
            msg_direction: MsgDirection::In,
            msg_name: "cmd2".to_string(),
        }];

        assert_eq!(compatibles.data, expected_compatibles);

        println!("Response body: {:?}", compatibles);
    }

    #[actix_web::test]
    async fn test_get_compatible_messages_cmd_property_required_missing() {
        let mut designer_state = DesignerState {
            tman_config: TmanConfig::default(),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
        };

        // The first item is 'manifest.json', and the second item is
        // 'property.json'.
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
                "/api/designer/v1/messages/compatible",
                web::post().to(get_compatible_messages),
            ),
        )
        .await;

        // Define input data.
        let input_data = json!({
          "base_dir": TEST_DIR,

          "app": "localhost",
          "graph": "default",
          "extension_group": "extension_group_1",
          "extension": "extension_1",
          "msg_type": "cmd",
          "msg_direction": "out",
          "msg_name": "cmd3"
        });

        // Send request to the test server.
        let req = test::TestRequest::post()
            .uri("/api/designer/v1/messages/compatible")
            .set_json(&input_data)
            .to_request();

        // Call the service and get the response.
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();

        let compatibles: ApiResponse<Vec<GetCompatibleMsgsSingleResponseData>> =
            serde_json::from_str(body_str).unwrap();

        assert!(compatibles.data.is_empty());
    }

    #[actix_web::test]
    async fn test_get_compatible_messages_cmd_result() {
        let mut designer_state = DesignerState {
            tman_config: TmanConfig::default(),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
        };

        // The first item is 'manifest.json', and the second item is
        // 'property.json'.
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
                "/api/designer/v1/messages/compatible",
                web::post().to(get_compatible_messages),
            ),
        )
        .await;

        // Define input data.
        let input_data = json!({
          "base_dir": TEST_DIR,

          "app": "localhost",
          "graph": "default",
          "extension_group": "extension_group_1",
          "extension": "extension_1",
          "msg_type": "cmd",
          "msg_direction": "out",
          "msg_name": "cmd4"
        });

        // Send request to the test server.
        let req = test::TestRequest::post()
            .uri("/api/designer/v1/messages/compatible")
            .set_json(&input_data)
            .to_request();

        // Call the service and get the response.
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();

        let compatibles: ApiResponse<Vec<GetCompatibleMsgsSingleResponseData>> =
            serde_json::from_str(body_str).unwrap();
        assert!(compatibles.data.is_empty());
    }

    #[actix_web::test]
    async fn test_get_compatible_messages_data_no_property() {
        let mut designer_state = DesignerState {
            tman_config: TmanConfig::default(),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
        };

        // The first item is 'manifest.json', and the second item is
        // 'property.json'.
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
                "/api/designer/v1/messages/compatible",
                web::post().to(get_compatible_messages),
            ),
        )
        .await;

        // Define input data.
        let input_data = json!({
          "base_dir": TEST_DIR,

          "app": "localhost",
          "graph": "default",
          "extension_group": "extension_group_1",
          "extension": "extension_1",
          "msg_type": "data",
          "msg_direction": "out",
          "msg_name": "data1"
        });

        // Send request to the test server.
        let req = test::TestRequest::post()
            .uri("/api/designer/v1/messages/compatible")
            .set_json(&input_data)
            .to_request();

        // Call the service and get the response.
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();

        let compatibles: ApiResponse<Vec<GetCompatibleMsgsSingleResponseData>> =
            serde_json::from_str(body_str).unwrap();
        assert!(compatibles.data.is_empty());
    }

    #[actix_web::test]
    async fn test_get_compatible_messages_video_target_has_property() {
        let mut designer_state = DesignerState {
            tman_config: TmanConfig::default(),
            out: Arc::new(Box::new(TmanOutputCli)),
            pkgs_cache: HashMap::new(),
        };

        // The first item is 'manifest.json', and the second item is
        // 'property.json'.
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
                "/api/designer/v1/messages/compatible",
                web::post().to(get_compatible_messages),
            ),
        )
        .await;

        // Define input data.
        let input_data = json!({
          "base_dir": TEST_DIR,

          "app": "localhost",
          "graph": "default",
          "extension_group": "extension_group_1",
          "extension": "extension_1",
          "msg_type": "video_frame",
          "msg_direction": "out",
          "msg_name": "pcm_frame1"
        });

        // Send request to the test server.
        let req = test::TestRequest::post()
            .uri("/api/designer/v1/messages/compatible")
            .set_json(&input_data)
            .to_request();

        // Call the service and get the response.
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();

        let compatibles: ApiResponse<Vec<GetCompatibleMsgsSingleResponseData>> =
            serde_json::from_str(body_str).unwrap();

        let expected_compatibles = vec![GetCompatibleMsgsSingleResponseData {
            app: localhost(),
            extension_group: "extension_group_1".to_string(),
            extension: "extension_1".to_string(),
            msg_type: MsgType::VideoFrame,
            msg_direction: MsgDirection::In,
            msg_name: "pcm_frame1".to_string(),
        }];

        assert_eq!(compatibles.data, expected_compatibles);
    }
}
