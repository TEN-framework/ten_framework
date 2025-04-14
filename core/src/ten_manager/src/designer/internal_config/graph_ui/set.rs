//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::sync::{Arc, RwLock};

use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::config::internal::GraphGeometry;
use crate::designer::response::{ApiResponse, Status};
use crate::designer::DesignerState;

#[derive(Debug, Serialize, Deserialize)]
pub struct SetGraphUiRequestPayload {
    pub graph_id: Uuid,
    pub graph_geometry: GraphGeometry,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetGraphUiResponseData {
    pub success: bool,
}

pub async fn set_graph_ui_endpoint(
    _request_payload: web::Json<SetGraphUiRequestPayload>,
    _state: web::Data<Arc<RwLock<DesignerState>>>,
) -> Result<impl Responder, actix_web::Error> {
    let response_data = SetGraphUiResponseData { success: true };
    let response = ApiResponse {
        status: Status::Ok,
        data: response_data,
        meta: None,
    };

    Ok(HttpResponse::Ok().json(response))
}
