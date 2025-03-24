//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import z from "zod";

import { TenLocalStorePackageSchema } from "@/types/extension";

export interface ISetBaseDirResponse {
  success: boolean;
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
