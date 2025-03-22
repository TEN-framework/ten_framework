//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import { makeAPIRequest } from "@/api/services/utils";
import { ENDPOINT_HELP_TEXT } from "@/api/endpoints";
import { ENDPOINT_METHOD } from "@/api/endpoints/constant";

export const retrieveHelpText = async (key: string, locale?: string) => {
  const template = ENDPOINT_HELP_TEXT.helpText[ENDPOINT_METHOD.POST];
  const req = makeAPIRequest(template, {
    body: { key, locale: locale || "en-US" },
  });
  const res = await req;
  return template.responseSchema.parse(res).data;
};
