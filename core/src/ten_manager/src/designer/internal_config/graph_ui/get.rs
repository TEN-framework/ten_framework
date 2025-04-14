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
pub struct GetGraphUiRequestPayload {
    pub graph_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetGraphUiResponseData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub graph_geometry: Option<GraphGeometry>,
}

pub async fn get_graph_ui_endpoint(
    _request_payload: web::Json<GetGraphUiRequestPayload>,
    _state: web::Data<Arc<RwLock<DesignerState>>>,
) -> Result<impl Responder, actix_web::Error> {
    let response_data = GetGraphUiResponseData {
        graph_geometry: None,
    };
    let response = ApiResponse {
        status: Status::Ok,
        data: response_data,
        meta: None,
    };

    Ok(HttpResponse::Ok().json(response))
}
