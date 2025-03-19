//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//

// Go to https://ten-console.theten.ai/ to get user token.
pub const DEFAULT_REGISTRY: &str =
    "https://ten-store.theten.ai/api/ten-cloud-store/v2/packages";

pub const CN_REGISTRY: &str =
    "https://registry-ten.rtcdeveloper.cn/api/ten-cloud-store/v2/packages";

pub const GITHUB_RELEASE_URL: &str =
    "https://api.github.com/repos/TEN-framework/ten_framework/releases/latest";

pub const GITHUB_RELEASE_PAGE: &str =
    "https://github.com/TEN-framework/ten_framework/releases/latest";

// This comment is just for reference.
//
// pub const ADMIN_TOKEN: &str =
//     "cnRlX3N0b3JlOjNjYzc2ODgxYjExNjQ0NTM4ZDQ5YmNmNmNlYjQ2Yjk0";

pub const DOT_TEN_DIR: &str = ".ten";
pub const APP_DIR_IN_DOT_TEN_DIR: &str = "app";
pub const INSTALLED_PATHS_JSON_FILENAME: &str = "installed_paths.json";
pub const PACKAGE_INFO_DIR_IN_DOT_TEN_DIR: &str = "package_info";
pub const PACKAGE_DIR_IN_DOT_TEN_DIR: &str = "package";

pub const LIB_DIR: &str = "lib";
pub const INCLUDE_DIR: &str = "include";

pub const APP_PKG_TYPE: &str = "app";

pub const SCRIPTS: &str = "scripts";

pub const INSTALL_PATHS_APP_PREFIX: &str = "@app";

pub const TEN_PACKAGE_FILE_EXTENSION: &str = ".tpkg";

// Maximum number of retry attempts.
pub const REMOTE_REGISTRY_MAX_RETRIES: u32 = 30;
// Delay between retries in milliseconds.
pub const REMOTE_REGISTRY_RETRY_DELAY_MS: u64 = 500;
// Timeout duration for requests in seconds.
pub const REMOTE_REGISTRY_REQUEST_TIMEOUT_SECS: u64 = 120;

pub const GITHUB_RELEASE_REQUEST_TIMEOUT_SECS: u64 = 10;

pub const DEFAULT_REGISTRY_PAGE_SIZE: u32 = 100;

// Designer-backend.
pub const PROCESS_STDOUT_MAX_LINE_CNT: usize = 1000;
pub const PROCESS_STDERR_MAX_LINE_CNT: usize = 1000;

pub const DESIGNER_BACKEND_DEFAULT_PORT: &str = "49483";

pub const TEST_DIR: &str = "test_dir";

pub const DEFAULT: &str = "default";
