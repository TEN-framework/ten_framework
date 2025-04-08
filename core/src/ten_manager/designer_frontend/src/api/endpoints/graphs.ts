//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import z from "zod";

import { API_DESIGNER_V1, ENDPOINT_METHOD } from "@/api/endpoints/constant";
import { genResSchema } from "@/api/endpoints/utils";
import {
  AddNodePayloadSchema,
  DeleteNodePayloadSchema,
  AddConnectionPayloadSchema,
  type IBackendNode,
  type IBackendConnection,
  type IGraph,
} from "@/types/graphs";

export const ENDPOINT_GRAPHS = {
  nodes: {
    [ENDPOINT_METHOD.POST]: {
      url: `${API_DESIGNER_V1}/graphs/nodes`,
      method: ENDPOINT_METHOD.POST,
      requestSchema: z.object({
        base_dir: z.string().optional(),
        graph_name: z.string(),
      }),
      responseSchema: genResSchema<IBackendNode[]>(
        z.array(
          z.object({
            addon: z.string(),
            name: z.string(),
            extension_group: z.string().optional(),
            app: z.string().optional(),
            property: z.unknown().optional(),
            api: z.unknown().optional(),
            is_installed: z.boolean(),
          })
        ) as z.ZodType<IBackendNode[]>
      ),
    },
  },
  addNode: {
    [ENDPOINT_METHOD.POST]: {
      url: `${API_DESIGNER_V1}/graphs/nodes/add`,
      method: ENDPOINT_METHOD.POST,
      requestSchema: AddNodePayloadSchema,
      responseSchema: genResSchema(
        z.object({
          success: z.boolean(),
        })
      ),
    },
  },
  deleteNode: {
    [ENDPOINT_METHOD.POST]: {
      url: `${API_DESIGNER_V1}/graphs/nodes/delete`,
      method: ENDPOINT_METHOD.POST,
      requestSchema: DeleteNodePayloadSchema,
      responseSchema: genResSchema(
        z.object({
          success: z.boolean(),
        })
      ),
    },
  },
  nodesPropertyUpdate: {
    [ENDPOINT_METHOD.POST]: {
      url: `${API_DESIGNER_V1}/graphs/nodes/property/update`,
      method: ENDPOINT_METHOD.POST,
      requestSchema: z.object({
        graph_app_base_dir: z.string(),
        graph_name: z.string(),
        addon_app_base_dir: z.string().optional(),
        node_name: z.string(),
        addon_name: z.string(),
        extension_group_name: z.string().optional(),
        app_uri: z.string().optional(),
        property: z.unknown().optional(),
      }),
      responseSchema: genResSchema(z.any()), // TODO: add response schema
    },
  },
  nodesValidate: {
    [ENDPOINT_METHOD.POST]: {
      url: `${API_DESIGNER_V1}/graphs/nodes/validate`,
      method: ENDPOINT_METHOD.POST,
      requestSchema: z.object({
        addon_app_base_dir: z.string().optional(),
        node_name: z.string(),
        addon_name: z.string(),
        extension_group_name: z.string().optional(),
        app_uri: z.string().optional(),
        property: z.unknown().optional(),
      }),
      responseSchema: genResSchema(z.any()), // TODO: add response schema
    },
  },
  connections: {
    [ENDPOINT_METHOD.POST]: {
      url: `${API_DESIGNER_V1}/graphs/connections`,
      method: ENDPOINT_METHOD.POST,
      requestSchema: z.object({
        base_dir: z.string().optional(),
        graph_name: z.string(),
      }),
      responseSchema: genResSchema<IBackendConnection[]>(
        z.array(
          z.object({
            app: z.string().optional(),
            extension: z.string(),
            cmd: z
              .array(
                z.object({
                  name: z.string(),
                  dest: z.array(
                    z.object({
                      app: z.string().optional(),
                      extension: z.string(),
                    })
                  ),
                })
              )
              .optional(),
            data: z
              .array(
                z.object({
                  name: z.string(),
                  dest: z.array(
                    z.object({
                      app: z.string().optional(),
                      extension: z.string(),
                    })
                  ),
                })
              )
              .optional(),
            audio_frame: z
              .array(
                z.object({
                  name: z.string(),
                  dest: z.array(
                    z.object({
                      app: z.string().optional(),
                      extension: z.string(),
                    })
                  ),
                })
              )
              .optional(),
            video_frame: z
              .array(
                z.object({
                  name: z.string(),
                  dest: z.array(
                    z.object({
                      app: z.string().optional(),
                      extension: z.string(),
                    })
                  ),
                })
              )
              .optional(),
          })
        )
      ),
    },
  },
  addConnection: {
    [ENDPOINT_METHOD.POST]: {
      url: `${API_DESIGNER_V1}/graphs/connections/add`,
      method: ENDPOINT_METHOD.POST,
      requestSchema: AddConnectionPayloadSchema,
      responseSchema: genResSchema(z.any()), // TODO: add response schema
    },
  },
  graphs: {
    [ENDPOINT_METHOD.POST]: {
      url: `${API_DESIGNER_V1}/graphs`,
      method: ENDPOINT_METHOD.POST,
      requestSchema: z.object({
        base_dir: z.string().optional(),
      }),
      responseSchema: genResSchema<IGraph[]>(
        z.array(
          z.object({
            name: z.string(),
            auto_start: z.boolean(),
          })
        )
      ),
    },
  },
};
