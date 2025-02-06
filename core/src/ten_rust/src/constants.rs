//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//

pub const TELEMETRY_DEFAULT_ENDPOINT: &str = "0.0.0.0:49484";
pub const TELEMETRY_DEFAULT_PATH: &str = "/metrics";

pub const TELEMETRY_SERVER_START_RETRY_MAX_ATTEMPTS: u32 = 3;
pub const TELEMETRY_SERVER_START_RETRY_INTERVAL: u64 = 1; // seconds
