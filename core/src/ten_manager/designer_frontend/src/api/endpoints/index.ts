//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import z from "zod";

import { API_DESIGNER_V1, ENDPOINT_METHOD } from "@/api/endpoints/constant";
import { genResSchema } from "@/api/endpoints/utils";

export interface IReqTemplate<T, R> {
  url: string;
  method: T;
  query?: string[];
  pathParams?: string[];
  responseSchema: z.ZodType<R>;
  // requestSchema?: z.ZodType<any>;
}

// Common endpoints
// export const ENDPOINT_COMMON: Record<
//   string,
//   Partial<Record<ENDPOINT_METHOD, IReqTemplate<ENDPOINT_METHOD, unknown>>>
export const ENDPOINT_COMMON = {
  version: {
    [ENDPOINT_METHOD.GET]: {
      url: `${API_DESIGNER_V1}/version`,
      method: ENDPOINT_METHOD.GET,
      responseSchema: genResSchema<{ version: string }>(
        z.object({
          version: z.string(),
        })
      ),
    },
  },
  checkUpdate: {
    [ENDPOINT_METHOD.GET]: {
      url: `${API_DESIGNER_V1}/check-update`,
      method: ENDPOINT_METHOD.GET,
      responseSchema: genResSchema<{
        update_available: boolean;
        latest_version: string | null;
        release_page: string | null;
        message: string | null;
      }>(
        z.object({
          update_available: z.boolean(),
          latest_version: z.string().nullable(),
          release_page: z.string().nullable(),
          message: z.string().nullable(),
        })
      ),
    },
  },
  env: {
    [ENDPOINT_METHOD.GET]: {
      url: `${API_DESIGNER_V1}/env`,
      method: ENDPOINT_METHOD.GET,
      responseSchema: genResSchema<{ os?: string; arch?: string }>(
        z.object({
          os: z.string().optional(),
          arch: z.string().optional(),
        })
      ),
    },
  },
};

export { ENDPOINT_GRAPHS } from "@/api/endpoints/graphs";
export { ENDPOINT_FILE_SYSTEM } from "@/api/endpoints/fileSystem";
export { ENDPOINT_APPS, ENDPOINT_ADDONS } from "@/api/endpoints/apps";
export { ENDPOINT_GH } from "@/api/endpoints/github";
export { ENDPOINT_HELP_TEXT } from "@/api/endpoints/help-text";
