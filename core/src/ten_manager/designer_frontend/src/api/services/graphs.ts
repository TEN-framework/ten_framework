//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import * as React from "react";

import {
  makeAPIRequest,
  prepareReqUrl,
  getQueryHookCache,
} from "@/api/services/utils";
import { ENDPOINT_GRAPHS } from "@/api/endpoints";
import { ENDPOINT_METHOD } from "@/api/endpoints/constant";

import type { IGraph } from "@/types/graphs";

export const retrieveGraphNodes = async (
  graphName: string,
  baseDir?: string | null
) => {
  const template = ENDPOINT_GRAPHS.nodes[ENDPOINT_METHOD.POST];
  const req = makeAPIRequest(template, {
    body: { graph_name: graphName, ...(baseDir && { base_dir: baseDir }) },
  });
  const res = await req;
  return template.responseSchema.parse(res).data;
};

export const retrieveGraphConnections = async (
  graphName: string,
  baseDir?: string | null
) => {
  const template = ENDPOINT_GRAPHS.connections[ENDPOINT_METHOD.POST];
  const req = makeAPIRequest(template, {
    body: { graph_name: graphName, ...(baseDir && { base_dir: baseDir }) },
  });
  const res = await req;
  return template.responseSchema.parse(res).data;
};

// TODO: refine this hook(post should not be used)
export const useGraphs = (baseDir?: string | null) => {
  const template = ENDPOINT_GRAPHS.graphs[ENDPOINT_METHOD.POST];
  const url =
    prepareReqUrl(template) + `${baseDir ? encodeURIComponent(baseDir) : ""}`;
  const queryHookCache = getQueryHookCache();

  const [data, setData] = React.useState<IGraph[] | null>(() => {
    const [cachedData, cachedDataIsExpired] = queryHookCache.get<IGraph[]>(url);
    if (!cachedData || cachedDataIsExpired) {
      return null;
    }
    return cachedData;
  });
  const [error, setError] = React.useState<Error | null>(null);
  const [isLoading, setIsLoading] = React.useState<boolean>(false);

  const fetchData = React.useCallback(async () => {
    if (baseDir === null) {
      return;
    }
    setIsLoading(true);
    try {
      const template = ENDPOINT_GRAPHS.graphs[ENDPOINT_METHOD.POST];
      const req = makeAPIRequest(template, {
        body: { base_dir: baseDir },
      });
      const res = await req;
      const parsedData = template.responseSchema.parse(res).data;
      setData(parsedData);
      queryHookCache.set(url, parsedData);
    } catch (err) {
      setError(err as Error);
    } finally {
      setIsLoading(false);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [baseDir, url]);

  React.useEffect(() => {
    fetchData();
  }, [fetchData]);

  return {
    graphs: data,
    error,
    isLoading,
    mutate: fetchData,
  };
};
