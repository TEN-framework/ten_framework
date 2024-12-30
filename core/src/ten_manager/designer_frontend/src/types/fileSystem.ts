export interface IFileContentResponse {
  content: string;
}

export interface ISetBaseDirResponse {
  success: boolean;
}

export type TBaseDirEntry = {
  name: string;
  path: string;
  is_dir?: boolean;
};

export interface IBaseDirResponse {
  entries: TBaseDirEntry[];
}
