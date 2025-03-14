//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import z from "zod";

import { API_DESIGNER_V1, ENDPOINT_METHOD } from "@/api/endpoints/constant";
import { genResSchema } from "@/api/endpoints/utils";
import type { IExtensionAddon } from "@/types/addons";

export const ENDPOINT_ADDONS = {
  extensionByName: {
    [ENDPOINT_METHOD.GET]: {
      url: `${API_DESIGNER_V1}/addons/extensions/:name`,
      method: ENDPOINT_METHOD.POST,
      pathParams: ["name"],
      responseSchema: genResSchema<IExtensionAddon>(
        z.object({
          name: z.string(),
          url: z.string(),
          api: z.unknown().optional(),
          type: z.string(),
        })
      ),
    },
  },
  addons: {
    [ENDPOINT_METHOD.POST]: {
      url: `${API_DESIGNER_V1}/addons`,
      method: ENDPOINT_METHOD.POST,
      body: z.object({
        base_dir: z.string().optional(),
        addon_name: z.string().optional(),
        addon_type: z.string().optional(),
      }),
      responseSchema: genResSchema<IExtensionAddon[]>(
        z.array(
          z.object({
            name: z.string(),
            url: z.string(),
            api: z.unknown().optional(),
            type: z.string(),
          })
        )
      ),
    },
  },
};
