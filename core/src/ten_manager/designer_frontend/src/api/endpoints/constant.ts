export const API_ENDPOINT_ROOT = "/api";
export enum API_ENDPOINT_CATEGORY {
  DESIGNER = "designer",
}
export enum API_ENDPOINT_VERSION {
  V1 = "v1",
}
export const API_DESIGNER_V1 =
  API_ENDPOINT_ROOT +
  "/" +
  API_ENDPOINT_CATEGORY.DESIGNER +
  "/" +
  API_ENDPOINT_VERSION.V1;

export enum ENDPOINT_METHOD {
  GET = "get",
  POST = "post",
  PUT = "put",
  DELETE = "delete",
}
