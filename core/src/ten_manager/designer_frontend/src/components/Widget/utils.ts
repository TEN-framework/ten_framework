//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import { z } from "zod";

export type TExtPropertySchema = Record<string, Record<"type", string>>;

export const defaultTypeValue = (type: string) => {
  switch (type) {
    case "int64":
    case "int32":
      return 0;
    case "float64":
    case "float32":
      return 0.1;
    case "bool":
      return false;
    case "string":
    default:
      return "";
  }
};

export const convertExtensionPropertySchema2ZodSchema = (
  input: TExtPropertySchema
) => {
  const schemaEntries: [string, z.ZodType][] = Object.entries(input).map(
    ([key, value]) => {
      const { type } = value;
      let zodType;
      const defaultValue = defaultTypeValue(type);

      // switch (type) {
      //   case "int64":
      //   case "int32":
      //     zodType = z
      //       .string()
      //       .transform((val) => parseInt(val, 10) || defaultValue)
      //       .optional();
      //     break;
      //   case "float64":
      //   case "float32":
      //     zodType = z
      //       .string()
      //       .transform((val) => parseFloat(val) || defaultValue)
      //       .optional();
      //     break;
      //   case "bool":
      //     zodType = z
      //       .union([z.boolean(), z.string()])
      //       .transform((val) => val === true || val === "true" || defaultValue)
      //       .optional();
      //     break;
      //   case "string":
      //     zodType = z
      //       .any()
      //       .transform((val) => String(val) || defaultValue)
      //       .optional();
      //     break;
      //   default:
      //     zodType = z.any().optional();
      // }

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
