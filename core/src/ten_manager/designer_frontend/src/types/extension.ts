//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import z from "zod";

export const TenCloudStorePackageSchema = z.object({
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
});

// eslint-disable-next-line @typescript-eslint/no-empty-object-type
export interface IListTenCloudStorePackage
  extends z.infer<typeof TenCloudStorePackageSchema> {}
