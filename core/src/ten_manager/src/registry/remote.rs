//
// Copyright © 2024 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::{io::Write, path::PathBuf};

use anyhow::{anyhow, Context, Result};
use reqwest::header::{HeaderMap, AUTHORIZATION, CONTENT_TYPE};
use semver::Version;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tempfile::NamedTempFile;

use super::{FoundResult, SearchCriteria};
use crate::{
    config::TmanConfig, error::TmanError, log::tman_verbose_println,
    registry::found_result::RegistryPackageData,
};
use ten_rust::pkg_info::{
    pkg_identity::PkgIdentity, supports::get_manifest_supports_from_pkg,
    PkgInfo,
};

#[derive(Debug)]
struct UploadInfo {
    resource_id: String,
    url: String,
}

async fn get_package_upload_info(
    tman_config: &TmanConfig,
    base_url: &str,
    client: &reqwest::Client,
    pkg_info: &PkgInfo,
) -> Result<UploadInfo> {
    // Basically, the principle here is that when tman install is run, all the
    // necessary metadata for a package (such as the package's dependencies,
    // support, etc.) should be uploaded to the registry during tman publish and
    // stored by the registry. This way, when tman install is run and the
    // package's metadata is requested, the registry can return it directly,
    // without tman needing to request the actual zip file from the registry,
    // extract the manifest.json from it, and then retrieve the information.
    // That would be too cumbersome. Therefore, information not needed during
    // tman install, such as api/schema, does not need to be uploaded to the
    // registry during tman publish. Of course, if it is uploaded, the potential
    // benefit is that this information can be directly displayed in the cloud
    // store.

    // TODO: Implement the 'environments' mechanism.
    // Add a new environments field, or perhaps directly upload the content of
    // manifest.json to the cloud store.
    let payload = json!(RegistryPackageData {
        pkg_type: pkg_info.pkg_identity.pkg_type.clone(),
        name: pkg_info.pkg_identity.name.clone(),
        version: pkg_info.version.clone(),
        dependencies: pkg_info
            .dependencies
            .clone()
            .into_iter()
            .map(|d| d.into())
            .collect(),
        supports: Some(get_manifest_supports_from_pkg(&pkg_info.supports)),
        hash: pkg_info.hash.clone(),
    });

    tman_verbose_println!(
        tman_config,
        "Payload of publishing: {}",
        payload.to_string()
    );

    let mut headers = HeaderMap::new();

    if let Some(user_token) = &tman_config.user_token {
        let basic_token = format!("Basic {}", user_token);
        headers.insert(
            AUTHORIZATION,
            basic_token.parse().map_err(|e| {
                eprintln!("Failed to parse authorization token: {}", e);
                e
            })?,
        );
    } else {
        tman_verbose_println!(tman_config, "Authorization token is missing");
        return Err(TmanError::Custom(
            "Authorization token is missing".to_string(),
        )
        .into());
    }

    let response = client
        .post(base_url)
        .headers(headers)
        .json(&payload)
        .send()
        .await
        .map_err(|e| anyhow!("Failed to send request: {}", e))?;

    if !response.status().is_success() {
        return Err(anyhow!(
            "Received non-success status code: {}",
            response.status(),
        ));
    }

    let data: Value = response
        .json()
        .await
        .map_err(|e| anyhow!("Failed to parse JSON response: {}", e))?;

    let resource_id = data["data"]["resourceId"]
        .as_str()
        .ok_or_else(|| anyhow!("Missing 'resourceId' in response"))?
        .to_string();
    let url = data["data"]["url"]
        .as_str()
        .ok_or_else(|| anyhow!("Missing 'url' in response"))?
        .to_string();

    Ok(UploadInfo { resource_id, url })
}

async fn upload_package_to_remote(
    client: &reqwest::Client,
    package_file_path: &str,
    url: &str,
) -> Result<()> {
    let body = match std::fs::read(package_file_path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Failed to read file at '{}': {}", package_file_path, e);
            return Err(e.into());
        }
    };

    let mut headers = HeaderMap::new();

    headers.insert(
        CONTENT_TYPE,
        "application/zip".parse().map_err(|e| {
            eprintln!("Failed to parse content type: {}", e);
            e
        })?,
    );

    let response = client.put(url).headers(headers).body(body).send().await;
    match response {
        Ok(resp) => {
            if resp.status().is_success() {
                Ok(())
            } else {
                Err(TmanError::Custom(format!(
                    "Failed to upload package with HTTP status {:?}",
                    resp.text().await
                ))
                .into())
            }
        }
        Err(e) => Err(e.into()),
    }
}

async fn ack_of_uploading(
    base_url: &str,
    client: &reqwest::Client,
    resource_id: &str,
) -> Result<()> {
    let url = match reqwest::Url::parse(base_url) {
        Ok(url) => url,
        Err(e) => {
            eprintln!("Invalid base URL: {}", e);
            return Err(e.into());
        }
    };

    let body = json!({ "resourceId": resource_id });

    let response = client.patch(url).json(&body).send().await;

    match response {
        Ok(resp) => {
            if resp.status().is_success() {
                Ok(())
            } else {
                Err(TmanError::Custom(format!(
                    "Failed to acknowledge uploading with HTTP status {}",
                    resp.status()
                ))
                .into())
            }
        }
        Err(e) => Err(e.into()),
    }
}

pub async fn upload_package(
    tman_config: &TmanConfig,
    base_url: &str,
    package_file_path: &str,
    pkg_info: &PkgInfo,
) -> Result<()> {
    let client = reqwest::Client::new();

    let upload_info =
        get_package_upload_info(tman_config, base_url, &client, pkg_info)
            .await?;
    upload_package_to_remote(&client, package_file_path, &upload_info.url)
        .await?;
    ack_of_uploading(base_url, &client, &upload_info.resource_id).await?;

    Ok(())
}

pub async fn get_package(
    tman_config: &TmanConfig,
    url: &str,
    temp_path: &mut NamedTempFile,
) -> Result<()> {
    let client = reqwest::Client::new();

    let response =
        client.get(url).send().await.with_context(|| {
            format!("Failed to send GET request to {}", url)
        })?;

    if !response.status().is_success() {
        return Err(anyhow!(
            "Failed to download the package: HTTP {}",
            response.status()
        ));
    }

    let content = response
        .bytes()
        .await
        .with_context(|| "Failed to read bytes from response")?;

    temp_path
        .write_all(&content)
        .with_context(|| "Failed to write content to temporary file")?;

    tman_verbose_println!(
        tman_config,
        "Package downloaded successfully from {} and written to {}",
        url,
        temp_path.path().display()
    );

    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
struct ApiResponse {
    status: String,
    data: RegistryPackagesData,
    meta: Value,
}

#[derive(Debug, Serialize, Deserialize)]
struct RegistryPackagesData {
    #[serde(rename = "totalSize")]
    total_size: u32,
    packages: Vec<RegistryPackageData>,
}

pub async fn get_package_list(
    tman_config: &TmanConfig,
    base_url: &str,
    pkg_identity: &PkgIdentity,
    criteria: &SearchCriteria,
) -> Result<Vec<FoundResult>> {
    let client = reqwest::Client::new();

    let mut results = Vec::new();
    let mut current_page = 1;
    let mut total_size;

    loop {
        let mut url = reqwest::Url::parse(base_url)?;
        url.query_pairs_mut()
            .append_pair("type", &pkg_identity.pkg_type.to_string())
            .append_pair("name", &pkg_identity.name)
            .append_pair("version", &criteria.version_req.to_string())
            .append_pair("pageSize", "10")
            .append_pair("page", &current_page.to_string());

        let response = client.get(url).send().await;

        let response = match response {
            Ok(response) => response,
            Err(e) => return Err(anyhow::anyhow!("Request failed: {}", e)),
        };

        if response.status() != reqwest::StatusCode::OK {
            return Err(anyhow::anyhow!(
                "API responded with non-ok status: {}",
                response.status()
            ));
        }

        let body = response.text().await?;
        let api_response = serde_json::from_str::<ApiResponse>(&body);
        let api_response = match api_response {
            Ok(api_response) => api_response,
            Err(e) => {
                return Err(anyhow::anyhow!(
                    "Failed to parse JSON response: {}",
                    e
                ))
            }
        };

        if api_response.status != "ok" {
            return Err(anyhow::anyhow!(
                "API responded with non-ok status: {}",
                api_response.status
            ));
        }

        total_size = api_response.data.total_size as usize;

        for package_data in api_response.data.packages {
            let url = PathBuf::from(format!(
                "{}/{}/{}/{}/{}",
                &base_url,
                package_data.pkg_type,
                package_data.name,
                package_data.version,
                package_data.hash,
            ));
            results.push(FoundResult { url, package_data });
        }

        // Check if we've fetched all packages based on totalSize.
        if results.len() >= total_size {
            break;
        }
        current_page += 1;
    }

    tman_verbose_println!(
        tman_config,
        "Fetched {} packages for {}:{}",
        results.len(),
        pkg_identity.pkg_type,
        pkg_identity.name,
    );

    Ok(results)
}

pub async fn delete_package(
    tman_config: &TmanConfig,
    base_url: &str,
    pkg_identity: &PkgIdentity,
    version: &Version,
    hash: &String,
) -> Result<()> {
    let client = reqwest::Client::new();

    // Ensure the base URL ends with a '/'.
    let base_url = if base_url.ends_with('/') {
        base_url.to_string()
    } else {
        format!("{}/", base_url)
    };

    let url = reqwest::Url::parse(&base_url).inspect_err(|&e| {
        eprintln!("Failed to parse base URL: {}", e);
    })?;

    let url = url
        .join(&format!(
            "{}/{}/{}/{}",
            &pkg_identity.pkg_type.to_string(),
            &pkg_identity.name,
            version,
            hash,
        ))
        .inspect_err(|&e| {
            eprintln!("Failed to join URL path: {}", e);
        })?;

    let mut headers = HeaderMap::new();

    if let Some(admin_token) = &tman_config.admin_token {
        let basic_token = format!("Basic {}", admin_token);
        headers.insert(
            AUTHORIZATION,
            basic_token.parse().map_err(|e| {
                eprintln!("Failed to parse authorization token: {}", e);
                e
            })?,
        );
    } else {
        tman_verbose_println!(tman_config, "Authorization token is missing");
        return Err(TmanError::Custom(
            "Authorization token is missing".to_string(),
        )
        .into());
    }

    // Sending the DELETE request.
    let response =
        client
            .delete(url)
            .headers(headers)
            .send()
            .await
            .map_err(|e| {
                eprintln!("Network request failed: {}", e);
                e
            })?;

    if response.status().is_success() {
        Ok(())
    } else {
        Err(
            TmanError::Custom(format!("HTTP error: {}", response.status()))
                .into(),
        )
    }
}
