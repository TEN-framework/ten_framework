//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import { makeAPIRequest } from "@/api/services/utils";
import { ENDPOINT_ADDONS } from "@/api/endpoints";
import { ENDPOINT_METHOD } from "@/api/endpoints/constant";

export const retrieveAddons = async (payload: {
  // base_dir is the base directory of the app to retrieve addons for.
  base_dir: string;

  // addon_name is the name of the addon to retrieve.
  addon_name?: string;

  // addon_type is the type of the addon to retrieve.
  addon_type?: string;
}) => {
  const template = ENDPOINT_ADDONS.addons[ENDPOINT_METHOD.POST];
  const req = makeAPIRequest(template, {
    body: payload,
  });
  const res = await req;
  return template.responseSchema.parse(res).data;
};
