//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import z from "zod";

import { API_EXTENSION_STORE_ROOT_PATH } from "@/api/endpoints/constant";
import { ENDPOINT_METHOD } from "@/api/endpoints/constant";

export const ENDPOINT_EXTENSION = {
  listTenCloudStorePackages: {
    [ENDPOINT_METHOD.GET]: {
      url: `${API_EXTENSION_STORE_ROOT_PATH}/api/ten-cloud-store/v1/packages`,
      method: ENDPOINT_METHOD.GET,
      queryParams: ["page", "pageSize"],
      responseSchema: z.object({
        status: z.string(),
        meta: z.any(),
        data: z.object({
          totalSize: z.number(),
          packages: z.array(
            z.object({
              type: z.string(),
              name: z.string(),
              version: z.string(),
              hash: z.string(),
              dependencies: z.array(
                z.object({
                  name: z.string(),
                  type: z.string(),
                  version: z.string(),
                })
              ),
              downloadUrl: z.string(),
              supports: z
                .array(
                  z.object({
                    os: z.string(),
                    arch: z.string(),
                  })
                )
                .optional(),
            })
          ),
        }),
      }),
    },
  },
};
