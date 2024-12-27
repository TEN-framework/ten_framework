import z from "zod";
import {
  makeAPIRequest,
  useCancelableSWR,
  prepareReqUrl,
} from "@/api/services/utils";
import { ENDPOINT_FILE_SYSTEM } from "@/api/endpoints";
import { ENDPOINT_METHOD } from "@/api/endpoints/constant";

export const getFileContent = async (path: string) => {
  const encodedPath = encodeURIComponent(path);
  const template = ENDPOINT_FILE_SYSTEM.fileContent.get;
  const req = makeAPIRequest(template, {
    pathParams: {
      path: encodedPath,
    },
  });
  const res = await req;
  return template.responseSchema.parse(res).data;
};

export const putFileContent = async (
  path: string,
  data: { content: string }
) => {
  const encodedPath = encodeURIComponent(path);
  const template = ENDPOINT_FILE_SYSTEM.fileContent[ENDPOINT_METHOD.PUT];
  const req = makeAPIRequest(template, {
    pathParams: {
      path: encodedPath,
    },
    body: data,
  });
  const res = await req;
  return res;
};

export const putBaseDir = async (baseDir: string) => {
  const template = ENDPOINT_FILE_SYSTEM.baseDir[ENDPOINT_METHOD.PUT];
  const req = makeAPIRequest(template, {
    body: { base_dir: baseDir },
  });
  const res = await req;
  return template.responseSchema.parse(res).data;
};

export const useFileContent = (path: string) => {
  const template = ENDPOINT_FILE_SYSTEM.fileContent[ENDPOINT_METHOD.GET];
  const url = prepareReqUrl(template, {
    pathParams: {
      path: encodeURIComponent(path),
    },
  });
  const [{ data, error, isLoading }] = useCancelableSWR<
    z.infer<typeof template.responseSchema>
  >(url, {
    revalidateOnFocus: false,
    refreshInterval: 0,
  });
  return {
    content: data?.data.content,
    error,
    isLoading,
  };
};
