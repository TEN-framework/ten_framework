//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

use actix_web::{web, HttpResponse, Responder};
use ten_rust::{
    graph::{
        connection::{GraphConnection, GraphDestination, GraphMessageFlow},
        msg_conversion::MsgAndResultConversion,
    },
    pkg_info::message::MsgType,
};

use crate::{
    designer::{
        graphs::util::find_app_package_from_base_dir,
        response::{ApiResponse, ErrorResponse, Status},
        DesignerState,
    },
    graph::update_graph_connections_all_fields,
};

#[derive(Serialize, Deserialize)]
pub struct UpdateGraphConnectionMsgConversionRequestPayload {
    pub base_dir: String,
    pub graph_name: String,

    pub src_app: Option<String>,
    pub src_extension: String,
    pub msg_type: MsgType,
    pub msg_name: String,
    pub dest_app: Option<String>,
    pub dest_extension: String,

    pub msg_conversion: Option<MsgAndResultConversion>,
}

#[derive(Serialize, Deserialize)]
pub struct UpdateGraphConnectionMsgConversionResponsePayload {
    pub success: bool,
}

pub async fn update_graph_connection_msg_conversion_endpoint(
    request_payload: web::Json<
        UpdateGraphConnectionMsgConversionRequestPayload,
    >,
    state: web::Data<Arc<RwLock<DesignerState>>>,
) -> Result<impl Responder, actix_web::Error> {
    // Get a write lock on the state since we may need to modify the graph.
    let mut state_write = state.write().unwrap();

    // Get the packages for this base_dir.
    if let Some(base_dir_pkg_info) =
        state_write.pkgs_cache.get_mut(&request_payload.base_dir)
    {
        // Find the app package.
        if let Some(app_pkg) = find_app_package_from_base_dir(base_dir_pkg_info)
        {
            // Check if the property exists.
            if let Some(property) = &mut app_pkg.property {
                // Create a GraphConnection with the message conversion to
                // update.
                let mut connection = GraphConnection {
                    app: request_payload.src_app.clone(),
                    extension: request_payload.src_extension.clone(),
                    cmd: None,
                    data: None,
                    audio_frame: None,
                    video_frame: None,
                };

                // Create the destination.
                let destination = GraphDestination {
                    app: request_payload.dest_app.clone(),
                    extension: request_payload.dest_extension.clone(),
                    msg_conversion: request_payload.msg_conversion.clone(),
                };

                // Create the message flow.
                let message_flow = GraphMessageFlow {
                    name: request_payload.msg_name.clone(),
                    dest: vec![destination],
                };

                // Set the appropriate message type field.
                match request_payload.msg_type {
                    MsgType::Cmd => {
                        connection.cmd = Some(vec![message_flow]);
                    }
                    MsgType::Data => {
                        connection.data = Some(vec![message_flow]);
                    }
                    MsgType::AudioFrame => {
                        connection.audio_frame = Some(vec![message_flow]);
                    }
                    MsgType::VideoFrame => {
                        connection.video_frame = Some(vec![message_flow]);
                    }
                }

                // Update the connection with the new message conversion.
                let connections_to_modify = vec![connection];

                // Update the property.json file.
                if let Err(e) = update_graph_connections_all_fields(
                    &request_payload.base_dir,
                    &mut property.all_fields,
                    &request_payload.graph_name,
                    None,
                    None,
                    Some(&connections_to_modify),
                ) {
                    // Return error if failed to update.
                    let error_response = ErrorResponse {
                        status: Status::Fail,
                        message: format!(
                            "Failed to update message conversion: {}",
                            e
                        ),
                        error: None,
                    };
                    return Ok(HttpResponse::InternalServerError()
                        .json(error_response));
                }

                // Return success response.
                let response = ApiResponse {
                    status: Status::Ok,
                    data: UpdateGraphConnectionMsgConversionResponsePayload {
                        success: true,
                    },
                    meta: None,
                };
                Ok(HttpResponse::Ok().json(response))
            } else {
                // Return error if property not found.
                let error_response = ErrorResponse {
                    status: Status::Fail,
                    message: "Property not found".to_string(),
                    error: None,
                };
                Ok(HttpResponse::NotFound().json(error_response))
            }
        } else {
            // Return error if app package not found.
            let error_response = ErrorResponse {
                status: Status::Fail,
                message: "App package not found".to_string(),
                error: None,
            };
            Ok(HttpResponse::NotFound().json(error_response))
        }
    } else {
        // Return error if base directory not found.
        let error_response = ErrorResponse {
            status: Status::Fail,
            message: "Base directory not found".to_string(),
            error: None,
        };
        Ok(HttpResponse::NotFound().json(error_response))
    }
}
