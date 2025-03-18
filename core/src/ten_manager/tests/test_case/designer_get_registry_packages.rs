//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#[cfg(test)]
mod tests {
    use actix_web::web;

    use ten_manager::designer::registry::packages::{
        get_packages_endpoint, GetPackagesRequestPayload,
    };
    use ten_rust::pkg_info::pkg_type::PkgType;

    use crate::test_case::common::builtin_server::start_test_server;

    #[actix_rt::test]
    async fn test_get_packages_success() {
        // Start the http server and get its address.
        let server_addr =
            start_test_server("/api/designer/v1/packages", || {
                web::post().to(get_packages_endpoint)
            })
            .await;
        println!("Server started at: {}", server_addr);

        // Create a request without filters (to get all packages).
        let request_payload = GetPackagesRequestPayload {
            pkg_type: Some(PkgType::Extension),
            name: Some("ext_a".to_string()),
            version_req: Some("1.0.0".to_string()),
        };

        println!("Request payload: {:?}", request_payload);

        // Create a reqwest client to send the request.
        let client = reqwest::Client::builder()
            .no_proxy() // Disable using proxy from environment variables
            .build()
            .expect("Failed to build client");

        // Construct the full URL using the server address.
        let url = format!("http://{}/api/designer/v1/packages", server_addr);
        println!("Sending request to URL: {}", url);

        // Send the POST request with the payload.
        let response = client
            .post(&url)
            .json(&request_payload)
            .send()
            .await
            .expect("Failed to send request");

        assert_eq!(response.status(), 200);
        let body = response.text().await.expect("Failed to read response");
        println!("Response body: {}", body);
    }
}
