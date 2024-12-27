export interface IBackendNode {
  addon: string;
  name: string;
  extension_group: string;
  app: string;
  property: unknown;
  api?: unknown;
}

export interface IBackendConnection {
  app: string;
  extension: string;
  cmd?: {
    name: string;
    dest: {
      app: string;
      extension: string;
    }[];
  }[];
}

export interface IGraph {
  name: string;
  auto_start: boolean;
}
