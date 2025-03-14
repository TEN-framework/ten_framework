//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import z from "zod";

export const TenPackageBaseSchema = z.object({
  type: z.string(),
  name: z.string(),
});

export const TenLocalStorePackageSchema = TenPackageBaseSchema.extend({
  url: z.string(),
  api: z.unknown().optional(),
});

export const TenCloudStorePackageSchema = TenPackageBaseSchema.extend({
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

export enum ETenPackageType {
  Local = "local",
  Default = "default",
}

// eslint-disable-next-line @typescript-eslint/no-empty-object-type
export interface IListTenCloudStorePackage
  extends z.infer<typeof TenCloudStorePackageSchema> {}

// eslint-disable-next-line @typescript-eslint/no-empty-object-type
export interface IListTenLocalStorePackage
  extends z.infer<typeof TenLocalStorePackageSchema> {}

export interface ITenPackageLocal extends IListTenLocalStorePackage {
  isInstalled: true;
  _type: ETenPackageType.Local;
}

export interface ITenPackage extends IListTenCloudStorePackage {
  isInstalled?: boolean;
  _type: ETenPackageType.Default;
}
