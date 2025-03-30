//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import z from "zod";

import { TenLocalStorePackageSchema } from "@/types/extension";

export interface ISetBaseDirResponse {
  app_uri: string | null;
}

export interface IGetBaseDirResponse {
  base_dir: string | null;
}

export interface IGetAppsResponse {
  app_info: {
    base_dir: string;
    app_uri: string;
  }[];
}

// eslint-disable-next-line @typescript-eslint/no-empty-object-type
export interface IExtensionAddon
  extends z.infer<typeof TenLocalStorePackageSchema> {}

export enum EWSMessageType {
  STANDARD_OUTPUT = "stdout",
  STANDARD_ERROR = "stderr",
  EXIT = "exit",
  NORMAL_LINE = "normal_line",
}

export enum EPreferencesTabs {
  GENERAL = "general",
  LOG = "log",
}

export const PREFERENCES_SCHEMA_LOG = z.object({
  logviewer_line_size: z.number().min(1).default(1000),
});
// tmp, need to move to scripts and retrieve from the backend.
export const PREFERENCES_SCHEMA = z.object({
  locale: z.enum(["en-US", "zh-CN", "zh-TW", "ja-JP"]).default("en-US"),
  ...PREFERENCES_SCHEMA_LOG.shape,
});
