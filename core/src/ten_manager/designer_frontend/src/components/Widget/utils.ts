//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import { z } from "zod";

export type TExtPropertySchema = Record<string, Record<"type", string>>;

export const convertExtensionPropertySchema2ZodSchema = (
  input: TExtPropertySchema
) => {
  const schemaEntries: [string, z.ZodType][] = Object.entries(input).map(
    ([key, value]) => {
      const { type } = value;
      let zodType;

      switch (type) {
        case "int64":
        case "int32":
          zodType = z.number().optional();
          break;
        case "float64":
        case "float32":
          zodType = z.number().optional();
          break;
        case "bool":
          zodType = z.boolean().optional();
          break;
        case "string":
          zodType = z.string().optional();
          break;
        default:
          zodType = z.any().optional();
      }

      return [key, zodType];
    }
  );

  return schemaEntries;
};
