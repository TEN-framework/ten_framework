import z from "zod";

import { API_DESIGNER_V1, ENDPOINT_METHOD } from "@/api/endpoints/constant";
import { genResSchema } from "@/api/endpoints/utils";

import type {
  IFileContentResponse,
  ISetBaseDirResponse,
  IBaseDirResponse,
} from "@/types/fileSystem";

export const ENDPOINT_FILE_SYSTEM = {
  fileContent: {
    [ENDPOINT_METHOD.GET]: {
      url: `${API_DESIGNER_V1}/file-content/:path`,
      method: ENDPOINT_METHOD.GET,
      pathParams: ["path"],
      responseSchema: genResSchema<IFileContentResponse>(
        z.object({
          content: z.string(),
        })
      ),
    },
    [ENDPOINT_METHOD.PUT]: {
      url: `${API_DESIGNER_V1}/file-content/:path`,
      method: ENDPOINT_METHOD.PUT,
      pathParams: ["path"],
      requestSchema: z.object({
        content: z.string(),
      }),
      responseSchema: genResSchema<null>(z.null()),
    },
  },
  baseDir: {
    [ENDPOINT_METHOD.PUT]: {
      url: `${API_DESIGNER_V1}/base-dir`,
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
  },
  dirList: {
    [ENDPOINT_METHOD.GET]: {
      url: `${API_DESIGNER_V1}/dir-list/:path`,
      method: ENDPOINT_METHOD.GET,
      pathParams: ["path"],
      responseSchema: genResSchema<IBaseDirResponse>(
        z.object({
          entries: z.array(
            z.object({
              name: z.string(),
              path: z.string(),
              is_dir: z.boolean().optional(),
            })
          ),
        })
      ),
    },
  },
};
