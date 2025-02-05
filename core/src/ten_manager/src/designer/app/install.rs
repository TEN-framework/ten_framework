//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::sync::{Arc, RwLock};

use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

use ten_rust::pkg_info::supports::PkgSupport;

use crate::cmd::cmd_install::{InstallCommand, LocalInstallMode};
use crate::config::TmanConfig;
use crate::designer::{
    response::{ApiResponse, ErrorResponse, Status},
    DesignerState,
};

#[derive(Deserialize, Serialize, Debug)]
pub struct InstallAppRequest {
    pub addon_type: Option<String>,
    pub addon_name: Option<String>,
}

pub async fn install_app(
    req: web::Json<InstallAppRequest>,
    _state: web::Data<Arc<RwLock<DesignerState>>>,
) -> impl Responder {
    // Construct the InstallCommand based on the payload.
    let command_data = if req.addon_type.is_none() && req.addon_name.is_none() {
        // Both fields are empty, which is equivalent to `tman install` (without
        // command line arguments).
        InstallCommand {
            package_type: None,
            package_name: None,
            support: PkgSupport {
                os: None,
                arch: None,
            },
            local_install_mode: LocalInstallMode::Link,
            standalone: false,
            local_path: None,
        }
    } else if req.addon_type.is_some() {
        // When addon_type has a value, addon_name must also have a value.
        if let (Some(ref addon_type), Some(ref addon_name)) =
            (&req.addon_type, &req.addon_name)
        {
            InstallCommand {
                package_type: Some(addon_type.clone()),
                package_name: Some(addon_name.clone()),
                support: PkgSupport {
                    os: None,
                    arch: None,
                },
                local_install_mode: LocalInstallMode::Link,
                standalone: false,
                local_path: None,
            }
        } else {
            // If only one field has a value, return an error.
            let error_response = ErrorResponse::from_error(
                &anyhow::anyhow!(
                    "Both addon_type and addon_name must be provided when
                    addon_type is specified."
                ),
                "Invalid payload for /app/install",
            );
            return HttpResponse::BadRequest().json(error_response);
        }
    } else {
        let error_response = ErrorResponse::from_error(
            &anyhow::anyhow!(
                "addon_type is required when addon_name is provided."
            ),
            "Invalid payload for /app/install",
        );
        return HttpResponse::BadRequest().json(error_response);
    };

    match crate::cmd::cmd_install::execute_cmd(
        &TmanConfig::default(),
        command_data,
    )
    .await
    {
        Ok(_) => {
            let response = ApiResponse {
                status: Status::Ok,
                data: serde_json::json!({ "success": true }),
                meta: None,
            };
            HttpResponse::Ok().json(response)
        }
        Err(err) => {
            let error_response = ErrorResponse::from_error(
                &err,
                "Failed to execute install command.",
            );
            HttpResponse::InternalServerError().json(error_response)
        }
    }
}
