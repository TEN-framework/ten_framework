//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import z from "zod";

import { API_DESIGNER_V1 } from "@/api/endpoints/constant";
import { ENDPOINT_METHOD } from "@/api/endpoints/constant";
import { TenCloudStorePackageSchema } from "@/types/extension";

export const ENDPOINT_EXTENSION = {
  registryPackages: {
    [ENDPOINT_METHOD.GET]: {
      url: `${API_DESIGNER_V1}/registry/packages`,
      method: ENDPOINT_METHOD.GET,
      queryParams: ["page", "pageSize"],
      responseSchema: z.object({
        status: z.string(),
        meta: z.any(),
        data: z.object({
          totalSize: z.number(),
          packages: z.array(TenCloudStorePackageSchema),
        }),
      }),
    },
  },
};
