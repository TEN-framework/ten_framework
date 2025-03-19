//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import z from "zod";

import {
  makeAPIRequest,
  useCancelableSWR,
  prepareReqUrl,
} from "@/api/services/utils";
import { ENDPOINT_APPS } from "@/api/endpoints";
import { ENDPOINT_METHOD } from "@/api/endpoints/constant";

export const getApps = async () => {
  const template = ENDPOINT_APPS.apps[ENDPOINT_METHOD.GET];
  const req = makeAPIRequest(template);
  const res = await req;
  return template.responseSchema.parse(res).data;
};

export const useApps = () => {
  const template = ENDPOINT_APPS.apps[ENDPOINT_METHOD.GET];
  const url = prepareReqUrl(template);
  const [{ data, error, isLoading, mutate }] = useCancelableSWR<
    z.infer<typeof template.responseSchema>
  >(url, {
    revalidateOnFocus: false,
    refreshInterval: 0,
  });
  return {
    data: data?.data,
    error,
    isLoading,
    mutate,
  };
};

export const postBaseDir = async (baseDir: string) => {
  const template = ENDPOINT_APPS.apps[ENDPOINT_METHOD.POST];
  const req = makeAPIRequest(template, {
    body: { base_dir: baseDir },
  });
  const res = await req;
  return template.responseSchema.parse(res).data;
};

export const postReloadApps = async (baseDir?: string) => {
  const template = ENDPOINT_APPS.reloadApps[ENDPOINT_METHOD.POST];
  const req = makeAPIRequest(template, {
    body: baseDir ? { base_dir: baseDir } : {},
  });
  const res = await req;
  return template.responseSchema.parse(res).data;
};

export const postUnloadApps = async (baseDir: string) => {
  const template = ENDPOINT_APPS.unloadApps[ENDPOINT_METHOD.POST];
  const req = makeAPIRequest(template, {
    body: { base_dir: baseDir },
  });
  const res = await req;
  return template.responseSchema.parse(res).data;
};
