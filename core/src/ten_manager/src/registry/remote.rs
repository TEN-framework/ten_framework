//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::fs;
use std::sync::Arc;
use std::{io::Write, time::Duration};

use anyhow::{anyhow, Context, Result};
use console::Emoji;
use reqwest::header::{HeaderMap, AUTHORIZATION, CONTENT_TYPE};
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tempfile::NamedTempFile;
use ten_rust::pkg_info::pkg_basic_info::PkgBasicInfo;
use ten_rust::pkg_info::pkg_type::PkgType;
use tokio::sync::RwLock;
use tokio::time::sleep;

use ten_rust::pkg_info::PkgInfo;

use super::pkg_cache::{find_in_package_cache, store_file_to_package_cache};
use crate::constants::{
    REMOTE_REGISTRY_MAX_RETRIES, REMOTE_REGISTRY_REQUEST_TIMEOUT_SECS,
    REMOTE_REGISTRY_RETRY_DELAY_MS,
};
use crate::http::create_http_client_with_proxies;
use crate::output::TmanOutput;
use crate::{config::TmanConfig, registry::found_result::PkgRegistryInfo};

async fn retry_async<'a, F, T>(
    tman_config: &TmanConfig,
    max_retries: u32,
    retry_delay: Duration,
    out: Arc<Box<dyn TmanOutput>>,
    mut operation: F,
) -> Result<T>
where
    F: FnMut() -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<T>> + Send + 'a>,
    >,
{
    for attempt in 0..=max_retries {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                if tman_config.verbose {
                    out.output_line(&format!(
                        "Attempt {} failed: {:?}",
                        attempt + 1,
                        e
                    ));
                }

                if attempt == max_retries {
                    return Err(e);
                }

                sleep(retry_delay).await;
            }
        }
    }

    unreachable!("retry logic should either return or error out")
}

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
    out: Arc<Box<dyn TmanOutput>>,
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

    let max_retries = REMOTE_REGISTRY_MAX_RETRIES;
    let retry_delay = Duration::from_millis(REMOTE_REGISTRY_RETRY_DELAY_MS);

    retry_async(tman_config, max_retries, retry_delay, out.clone(), || {
        let base_url = base_url.to_string();
        let client = client.clone();
        let pkg_info = pkg_info.clone();
        let out = out.clone();

        Box::pin(async move {
            let payload = json!(PkgRegistryInfo {
                basic_info: PkgBasicInfo::from(&pkg_info),
                dependencies: pkg_info
                    .dependencies
                    .clone()
                    .into_iter()
                    .collect(),
                hash: pkg_info.hash.clone(),
                download_url: String::new(),
                content_format: Some("gzip".to_string()),
            });

            if tman_config.verbose {
                out.output_line(&format!("Payload of publishing: {}", payload));
            }

            let mut headers = HeaderMap::new();

            if let Some(user_token) = &tman_config.user_token {
                let basic_token = format!("Basic {}", user_token);
                headers.insert(
                    AUTHORIZATION,
                    basic_token.parse().map_err(|e| {
                        out.output_err_line(&format!(
                            "Failed to parse authorization token: {}",
                            e
                        ));
                        e
                    })?,
                );
            } else {
                if tman_config.verbose {
                    out.output_line("Authorization token is missing");
                }
                return Err(anyhow!("Authorization token is missing"));
            }

            let response = client
                .post(base_url)
                .headers(headers)
                .timeout(Duration::from_secs(
                    REMOTE_REGISTRY_REQUEST_TIMEOUT_SECS,
                ))
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
        })
    })
    .await
}

async fn upload_package_to_remote(
    tman_config: &TmanConfig,
    client: &reqwest::Client,
    package_file_path: &str,
    url: &str,
    out: Arc<Box<dyn TmanOutput>>,
) -> Result<()> {
    let max_retries = REMOTE_REGISTRY_MAX_RETRIES;
    let retry_delay = Duration::from_millis(REMOTE_REGISTRY_RETRY_DELAY_MS);

    retry_async(tman_config, max_retries, retry_delay, out.clone(), || {
        let client = client.clone();
        let package_file_path = package_file_path.to_string();
        let url = url.to_string();
        let out = out.clone();

        Box::pin(async move {
            let body = match std::fs::read(&package_file_path) {
                Ok(file) => file,
                Err(e) => {
                    out.output_err_line(&format!(
                        "Failed to read file at '{}': {}",
                        &package_file_path, e
                    ));
                    return Err(e.into());
                }
            };

            let mut headers = HeaderMap::new();

            headers.insert(
                CONTENT_TYPE,
                "application/gzip".parse().map_err(|e| {
                    out.output_err_line(&format!(
                        "Failed to parse content type: {}",
                        e
                    ));
                    e
                })?,
            );

            let response =
                client.put(url).headers(headers).body(body).send().await;
            match response {
                Ok(resp) => {
                    if resp.status().is_success() {
                        Ok(())
                    } else {
                        Err(anyhow!(
                            "Failed to upload package with HTTP status {:?}",
                            resp.text().await
                        ))
                    }
                }
                Err(e) => Err(e.into()),
            }
        })
    })
    .await
}

async fn ack_of_uploading(
    tman_config: &TmanConfig,
    base_url: &str,
    client: &reqwest::Client,
    resource_id: &str,
    out: Arc<Box<dyn TmanOutput>>,
) -> Result<()> {
    let max_retries = REMOTE_REGISTRY_MAX_RETRIES;
    let retry_delay = Duration::from_millis(REMOTE_REGISTRY_RETRY_DELAY_MS);

    retry_async(tman_config, max_retries, retry_delay, out.clone(), || {
        let client = client.clone();
        let base_url = base_url.to_string();
        let resource_id = resource_id.to_string();
        let out = out.clone();

        Box::pin(async move {
            let url = match reqwest::Url::parse(&base_url) {
                Ok(url) => url,
                Err(e) => {
                    out.output_err_line(&format!("Failed to parse URL: {}", e));
                    return Err(e.into());
                }
            };

            let body = json!({ "resourceId": resource_id });

            let response = client
                .patch(url)
                .timeout(Duration::from_secs(
                    REMOTE_REGISTRY_REQUEST_TIMEOUT_SECS,
                ))
                .json(&body)
                .send()
                .await;

            match response {
                Ok(resp) => {
                    if resp.status().is_success() {
                        Ok(())
                    } else {
                        Err(anyhow!(
                    "Failed to acknowledge uploading with HTTP status {}",
                    resp.status()
                ))
                    }
                }
                Err(e) => Err(e.into()),
            }
        })
    })
    .await
}

pub async fn upload_package(
    tman_config: &TmanConfig,
    base_url: &str,
    package_file_path: &str,
    pkg_info: &PkgInfo,
    out: Arc<Box<dyn TmanOutput>>,
) -> Result<String> {
    let client = create_http_client_with_proxies()?;

    let upload_info = get_package_upload_info(
        tman_config,
        base_url,
        &client,
        pkg_info,
        out.clone(),
    )
    .await?;

    upload_package_to_remote(
        tman_config,
        &client,
        package_file_path,
        &upload_info.url,
        out.clone(),
    )
    .await?;

    ack_of_uploading(
        tman_config,
        base_url,
        &client,
        &upload_info.resource_id,
        out.clone(),
    )
    .await?;

    Ok(upload_info.url)
}

// Parse the header of content-range.
fn parse_content_range(content_range: &str) -> Option<(u64, u64, u64)> {
    let parts: Vec<&str> = content_range.split('/').collect();
    if parts.len() == 2 {
        let total_size = parts[1].parse().ok()?;
        let range_parts: Vec<&str> = parts[0].split('-').collect();
        if range_parts.len() == 2 {
            let start = range_parts[0].parse().ok()?;
            let end = range_parts[1].parse().ok()?;
            return Some((start, end, total_size));
        }
    }
    None
}

pub async fn get_package(
    tman_config: &TmanConfig,
    pkg_type: &PkgType,
    pkg_name: &str,
    pkg_version: &Version,
    url: &str,
    temp_file: &mut NamedTempFile,
    out: Arc<Box<dyn TmanOutput>>,
) -> Result<()> {
    // First, check the cache. If there is a matching filename, use the cached
    // file directly.
    let parsed_url = url::Url::parse(url)
        .map_err(|e| anyhow::anyhow!("Invalid URL: {}", e))?;
    let file_name = parsed_url
        .path_segments()
        .and_then(|segments| segments.last())
        .ok_or_else(|| {
            anyhow::anyhow!("Failed to extract file name from URL")
        })?;

    if let Some(cached_file_path) =
        find_in_package_cache(pkg_type, pkg_name, pkg_version, file_name)?
    {
        // If the filename matches, directly copy the cached file to
        // `temp_file`.
        out.output_err_line(&format!(
            "{}  Found the package file ({}) in the package cache, using it directly.",
            Emoji("🚀", ":-("),
            cached_file_path.to_string_lossy()
        ));

        fs::copy(&cached_file_path, temp_file.path()).with_context(|| {
            format!("Failed to copy from cache {}", cached_file_path.display())
        })?;
        return Ok(());
    }

    // Not found in the package cache, so proceed with the standard process to
    // retrieve it from the registry.

    let client = create_http_client_with_proxies()?;

    let temp_file = Arc::new(RwLock::new(temp_file));

    let max_retries = REMOTE_REGISTRY_MAX_RETRIES;
    let retry_delay = Duration::from_millis(REMOTE_REGISTRY_RETRY_DELAY_MS);

    let download_complete = Arc::new(RwLock::new(false));

    retry_async(tman_config, max_retries, retry_delay, out.clone(), || {
        let client = client.clone();
        let url = url.to_string();
        let temp_file = Arc::clone(&temp_file);
        let download_complete = Arc::clone(&download_complete);

        Box::pin(async move {
            // Check the size of the file that has already been downloaded.
            let temp_file_len = {
                let temp_file_read = temp_file.read().await;
                temp_file_read
                    .as_file()
                    .metadata()
                    .map(|metadata| metadata.len())
                    .unwrap_or(0)
            };

            // Set the Range header to support resumable downloads.
            let mut headers = HeaderMap::new();
            if temp_file_len > 0 {
                headers.insert(
                    reqwest::header::RANGE,
                    format!("bytes={}-", temp_file_len).parse().unwrap(),
                );
            }

            let response = client
                .get(&url)
                .headers(headers)
                .timeout(Duration::from_secs(
                    REMOTE_REGISTRY_REQUEST_TIMEOUT_SECS,
                ))
                .send()
                .await
                .with_context(|| {
                    format!("Failed to send GET request to {}", url)
                })?;

            if !response.status().is_success()
                && response.status() != reqwest::StatusCode::PARTIAL_CONTENT
            {
                return Err(anyhow!(
                    "Failed to download the package from {}: HTTP {}.",
                    url,
                    response.status()
                ));
            }

            // Get the headers of Content-Range or Content-Length.
            let content_range = response
                .headers()
                .get(reqwest::header::CONTENT_RANGE)
                .cloned();
            let content_length = response
                .headers()
                .get(reqwest::header::CONTENT_LENGTH)
                .cloned();

            // Read the content of the response and append the newly downloaded
            // part to the file.
            let content = response
                .bytes()
                .await
                .with_context(|| "Failed to read bytes from response")?;

            if content.is_empty() {
                return Err(anyhow!("No new content downloaded"));
            }

            let mut temp_file_write = temp_file.write().await;
            temp_file_write
                .as_file_mut()
                .write_all(&content)
                .with_context(|| "Failed to write content to temporary file")?;

            // Check if we have downloaded the entire file.
            if let Some(content_range) = content_range {
                // Parse the content-range to check if download is complete.
                let content_range_str = content_range.to_str().unwrap();
                if let Some((_, _, total_size)) =
                    parse_content_range(content_range_str)
                {
                    if temp_file_len + content.len() as u64 >= total_size {
                        *download_complete.write().await = true;
                    }
                }
            } else if content_length.is_some() {
                // If there is no content-range but content-length exists, the
                // download should be complete in one go.
                *download_complete.write().await = true;
            }

            Ok(())
        })
    })
    .await?;

    // Only print when `download_complete` is `true`.
    if *download_complete.read().await {
        let temp_file_borrow = temp_file.read().await;
        if tman_config.verbose {
            out.output_line(&format!(
                "Package downloaded successfully from {} and written to {}",
                url,
                temp_file_borrow.path().display()
            ));
        }

        if tman_config.enable_package_cache {
            // Place the downloaded file into the cache.
            let downloaded_path = temp_file_borrow.path();
            store_file_to_package_cache(
                pkg_type,
                pkg_name,
                pkg_version,
                file_name,
                downloaded_path,
            )?;
        }
    }

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

    packages: Vec<PkgRegistryInfo>,
}

pub async fn get_package_list(
    tman_config: &TmanConfig,
    base_url: &str,
    pkg_type: PkgType,
    name: &String,
    version_req: &VersionReq,
    out: Arc<Box<dyn TmanOutput>>,
) -> Result<Vec<PkgRegistryInfo>> {
    let max_retries = REMOTE_REGISTRY_MAX_RETRIES;
    let retry_delay = Duration::from_millis(REMOTE_REGISTRY_RETRY_DELAY_MS);

    retry_async(tman_config, max_retries, retry_delay, out.clone(), || {
        let client = match create_http_client_with_proxies() {
            Ok(c) => c,
            Err(e) => return Box::pin(async { Err(e) }),
        };

        let out = out.clone();

        Box::pin(async move {
            let mut results = Vec::new();
            let mut current_page = 1;
            let mut total_size;

            loop {
                let mut url = reqwest::Url::parse(base_url)?;
                url.query_pairs_mut()
                    .append_pair("type", &pkg_type.to_string())
                    .append_pair("name", name)
                    .append_pair("version", &version_req.to_string())
                    .append_pair("pageSize", "100")
                    .append_pair("page", &current_page.to_string());

                let response = client
                    .get(url)
                    .timeout(Duration::from_secs(
                        REMOTE_REGISTRY_REQUEST_TIMEOUT_SECS,
                    ))
                    .send()
                    .await;

                let response = match response {
                    Ok(response) => response,
                    Err(e) => {
                        return Err(anyhow::anyhow!("Request failed: {}", e))
                    }
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

                for pkg_registry_info in api_response.data.packages {
                    results.push(pkg_registry_info);
                }

                if tman_config.verbose {
                    out.output_line(&format!(
                        "Fetched {} packages at page {} for {}:{}@{}",
                        results.len(),
                        current_page,
                        pkg_type,
                        name,
                        version_req
                    ));
                }

                // Check if we've fetched all packages based on totalSize.
                if results.len() >= total_size {
                    break;
                }

                current_page += 1;
            }

            Ok(results)
        })
    })
    .await
}

pub async fn delete_package(
    tman_config: &TmanConfig,
    base_url: &str,
    pkg_type: PkgType,
    name: &String,
    version: &Version,
    hash: &str,
    out: Arc<Box<dyn TmanOutput>>,
) -> Result<()> {
    let max_retries = REMOTE_REGISTRY_MAX_RETRIES;
    let retry_delay = Duration::from_millis(REMOTE_REGISTRY_RETRY_DELAY_MS);

    retry_async(tman_config, max_retries, retry_delay, out.clone(), || {
        let client = match create_http_client_with_proxies() {
            Ok(c) => c,
            Err(e) => return Box::pin(async { Err(e) }),
        };

        let base_url = base_url.to_string();
        let out = out.clone();

        Box::pin(async move {
            // Ensure the base URL ends with a '/'.
            let base_url = if base_url.ends_with('/') {
                base_url
            } else {
                format!("{}/", base_url)
            };

            let url = reqwest::Url::parse(&base_url).inspect_err(|&e| {
                out.output_err_line(&format!(
                    "Failed to parse base URL: {}",
                    e
                ));
            })?;

            let url = url
                .join(&format!(
                    "{}/{}/{}/{}",
                    &pkg_type.to_string(),
                    &name,
                    version,
                    hash,
                ))
                .inspect_err(|&e| {
                    out.output_err_line(&format!(
                        "Failed to join URL path: {}",
                        e
                    ));
                })?;

            let mut headers = HeaderMap::new();

            if let Some(admin_token) = &tman_config.admin_token {
                let basic_token = format!("Basic {}", admin_token);
                headers.insert(
                    AUTHORIZATION,
                    basic_token.parse().map_err(|e| {
                        out.output_err_line(&format!(
                            "Failed to parse authorization token: {}",
                            e
                        ));
                        e
                    })?,
                );
            } else {
                if tman_config.verbose {
                    out.output_line("Authorization token is missing");
                }
                return Err(anyhow!("Authorization token is missing"));
            }

            // Sending the DELETE request.
            let response = client
                .delete(url)
                .headers(headers)
                .timeout(Duration::from_secs(
                    REMOTE_REGISTRY_REQUEST_TIMEOUT_SECS,
                ))
                .send()
                .await;

            match response {
                Ok(resp) => {
                    if resp.status().is_success() {
                        Ok(())
                    } else {
                        Err(anyhow!("HTTP error: {}", resp.status()))
                    }
                }
                Err(e) => Err(e.into()),
            }
        })
    })
    .await
}
