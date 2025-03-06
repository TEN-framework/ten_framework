//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
export const TEN_FRAMEWORK_URL = "https://www.theten.ai/";
export const TEN_FRAMEWORK_GITHUB_URL = "https://github.com/TEN-framework/";
export const TEN_FRAMEWORK_RELEASE_URL =
  "https://github.com/TEN-framework/ten_framework/releases";
export const TEN_AGENT_URL = "https://agent.theten.ai/";
export const TEN_AGENT_GITHUB_URL =
  "https://github.com/TEN-framework/TEN-Agent";

export const TEN_DEFAULT_APP_RUN_SCRIPT = "start";

// --- Backend

export const TEN_DEFAULT_BACKEND_ENDPOINT = "localhost:49483";
export const TEN_DEFAULT_BACKEND_WS_ENDPOINT =
  import.meta.env.VITE_TMAN_GD_BACKEND_WS_ENDPOINT ||
  `ws://${TEN_DEFAULT_BACKEND_ENDPOINT}`;
export const TEN_DEFAULT_BACKEND_HTTP_ENDPOINT =
  import.meta.env.VITE_TMAN_GD_BACKEND_HTTP_ENDPOINT ||
  `http://${TEN_DEFAULT_BACKEND_ENDPOINT}`;

// --- Backend Paths

export const TEN_PATH_WS_TERMINAL = "/api/designer/v1/ws/terminal";
export const TEN_PATH_WS_BUILTIN_FUNCTION =
  "/api/designer/v1/ws/builtin-function";
export const TEN_PATH_WS_RUN_SCRIPT = "/api/designer/v1/ws/run-script";

// --- Github
export const TEN_FRAMEWORK_GH_FULL_NAME = "Ten-framework/ten-framework";
export const TEN_FRAMEWORK_GH_OWNER = "Ten-framework";
export const TEN_FRAMEWORK_GH_REPO = "ten-framework";
