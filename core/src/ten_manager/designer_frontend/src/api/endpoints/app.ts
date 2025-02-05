//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import z from "zod";

import { API_DESIGNER_V1, ENDPOINT_METHOD } from "@/api/endpoints/constant";
import { genResSchema } from "@/api/endpoints/utils";
import type {
  ISetBaseDirResponse,
  IGetBaseDirResponse,
} from "@/types/fileSystem";

export const ENDPOINT_APP = {
  baseDir: {
    [ENDPOINT_METHOD.PUT]: {
      url: `${API_DESIGNER_V1}/app/base-dir`,
      method: ENDPOINT_METHOD.PUT,
      requestSchema: z.object({
        base_dir: z.string(),
      }),
      responseSchema: genResSchema<ISetBaseDirResponse>(
        z.object({
          success: z.boolean(),
        })
      ),
    },
    [ENDPOINT_METHOD.GET]: {
      url: `${API_DESIGNER_V1}/app/base-dir`,
      method: ENDPOINT_METHOD.GET,
      responseSchema: genResSchema<IGetBaseDirResponse>(
        z.object({
          base_dir: z.string().nullable(),
        })
      ),
    },
  },
};
