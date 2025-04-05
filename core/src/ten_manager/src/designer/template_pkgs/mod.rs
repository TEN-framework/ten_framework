//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use actix_web::{web, HttpResponse, Responder};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

use ten_rust::pkg_info::pkg_type::PkgType;

use crate::{
    constants::{
        DEFAULT_APP_CPP, DEFAULT_APP_GO, DEFAULT_APP_NODEJS,
        DEFAULT_APP_PYTHON, DEFAULT_EXTENSION_CPP, DEFAULT_EXTENSION_GO,
        DEFAULT_EXTENSION_NODEJS, DEFAULT_EXTENSION_PYTHON,
    },
    designer::response::{ApiResponse, ErrorResponse, Status},
};

#[derive(
    Deserialize, Serialize, Debug, EnumString, Display, Clone, PartialEq,
)]
#[strum(serialize_all = "lowercase")]
pub enum TemplateLanguage {
    #[serde(rename = "cpp")]
    Cpp,

    #[serde(rename = "golang")]
    Golang,

    #[serde(rename = "python")]
    Python,

    #[serde(rename = "typescript")]
    TypeScript,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct GetTemplateRequestPayload {
    pub pkg_type: PkgType,
    pub language: TemplateLanguage,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetTemplateResponseData {
    pub template_name: Vec<String>,
}

pub async fn get_template_endpoint(
    request_payload: web::Json<GetTemplateRequestPayload>,
) -> Result<impl Responder, actix_web::Error> {
    let GetTemplateRequestPayload { pkg_type, language } =
        request_payload.into_inner();

    // Clone the language for later use in error messages.
    let language_clone = language.clone();

    // Generate template name based on package type and language.
    let template_name = match (pkg_type, language) {
        (PkgType::App, TemplateLanguage::Cpp) => DEFAULT_APP_CPP,
        (PkgType::App, TemplateLanguage::Golang) => DEFAULT_APP_GO,
        (PkgType::App, TemplateLanguage::Python) => DEFAULT_APP_PYTHON,
        (PkgType::App, TemplateLanguage::TypeScript) => DEFAULT_APP_NODEJS,
        (PkgType::Extension, TemplateLanguage::Cpp) => DEFAULT_EXTENSION_CPP,
        (PkgType::Extension, TemplateLanguage::Golang) => DEFAULT_EXTENSION_GO,
        (PkgType::Extension, TemplateLanguage::Python) => {
            DEFAULT_EXTENSION_PYTHON
        }
        (PkgType::Extension, TemplateLanguage::TypeScript) => {
            DEFAULT_EXTENSION_NODEJS
        }
        _ => {
            let error_message = format!(
                "Unsupported template combination: pkg_type={}, language={}",
                pkg_type, language_clone
            );

            let error = anyhow!(error_message);
            let error_response = ErrorResponse::from_error(
                &error,
                "Unsupported template combination",
            );

            return Ok(HttpResponse::BadRequest().json(error_response));
        }
    };

    let response = ApiResponse {
        status: Status::Ok,
        data: GetTemplateResponseData {
            template_name: vec![template_name.to_string()],
        },
        meta: None,
    };

    Ok(HttpResponse::Ok().json(response))
}
