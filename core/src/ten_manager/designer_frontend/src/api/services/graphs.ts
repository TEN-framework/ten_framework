import { makeAPIRequest } from "@/api/services/utils";
import { ENDPOINT_GRAPHS } from "@/api/endpoints";
import { ENDPOINT_METHOD } from "@/api/endpoints/constant";

export const getGraphNodes = async (graphName: string) => {
  const template = ENDPOINT_GRAPHS.nodes[ENDPOINT_METHOD.GET];
  const req = makeAPIRequest(template, {
    pathParams: { graphName: encodeURIComponent(graphName) },
  });
  const res = await req;
  return template.responseSchema.parse(res).data;
};

export const getGraphConnections = async (graphName: string) => {
  const template = ENDPOINT_GRAPHS.connections[ENDPOINT_METHOD.GET];
  const req = makeAPIRequest(template, {
    pathParams: { graphName: encodeURIComponent(graphName) },
  });
  const res = await req;
  return template.responseSchema.parse(res).data;
};

export const getGraphs = async () => {
  const template = ENDPOINT_GRAPHS.graphs[ENDPOINT_METHOD.GET];
  const req = makeAPIRequest(template);
  const res = await req;
  return template.responseSchema.parse(res).data;
};
