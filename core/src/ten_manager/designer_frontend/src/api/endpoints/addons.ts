import z from "zod";

import { API_DESIGNER_V1, ENDPOINT_METHOD } from "@/api/endpoints/constant";
import { genResSchema } from "@/api/endpoints/utils";

import type { IExtensionAddon } from "@/types/addons";

export const ENDPOINT_ADDONS = {
  extensionByName: {
    [ENDPOINT_METHOD.GET]: {
      url: `${API_DESIGNER_V1}/addons/extensions/:name`,
      method: ENDPOINT_METHOD.GET,
      pathParams: ["name"],
      responseSchema: genResSchema<IExtensionAddon>(
        z.object({
          addon_name: z.string(),
          url: z.string(),
          api: z.unknown().optional(),
        })
      ),
    },
  },
};
