//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import { z } from "zod";
import { IListTenCloudStorePackage } from "@/types/extension";

export interface TerminalData {
  title: string;
  url?: string;
}

export interface EditorData {
  title: string;

  // The url (path) of the editor to display.
  url: string;

  // The content of the editor to display.
  content: string;
}

export interface CustomConnectionData {
  id: string;
  source: string;
  target?: string;
}

export enum EWidgetDisplayType {
  Popup = "popup",
  Dock = "dock",
}

export enum EWidgetCategory {
  Terminal = "terminal",
  Editor = "editor",
  CustomConnection = "custom_connection",
  LogViewer = "log_viewer",
  Extension = "extension",
  Default = "default",
}

export interface IWidgetBase {
  id: string;
  display_type: EWidgetDisplayType;
}

export interface ITerminalWidget extends IWidgetBase {
  category: EWidgetCategory.Terminal;
  metadata: TerminalData;
}

export interface IEditorWidget extends IWidgetBase {
  category: EWidgetCategory.Editor;
  metadata: EditorData;
  isEditing?: boolean;
}

export interface ICustomConnectionWidget extends IWidgetBase {
  category: EWidgetCategory.CustomConnection;
  metadata: CustomConnectionData;
}

export enum ELogViewerScriptType {
  DEFAULT = "default",
  RUN_SCRIPT = "run_script",
  INSTALL_ALL = "install_all",
}

export const LogViewerScriptSchemaMap = {
  [ELogViewerScriptType.RUN_SCRIPT]: z.object({
    type: z.literal(ELogViewerScriptType.RUN_SCRIPT),
    base_dir: z.string().default(""),
    name: z.string().default(""),
  }),
  [ELogViewerScriptType.INSTALL_ALL]: z.object({
    type: z.literal(ELogViewerScriptType.INSTALL_ALL),
    base_dir: z.string().default(""),
  }),
  [ELogViewerScriptType.DEFAULT]: z.object({}),
};

export interface ILogViewerWidgetOptions {
  disableSearch?: boolean;
  title?: string | React.ReactNode;
}

export interface ILogViewerWidgetData<T extends ELogViewerScriptType> {
  wsUrl: string;
  onStop?: () => void;
  scriptType: T;
  script: z.infer<(typeof LogViewerScriptSchemaMap)[T]>;
  options?: ILogViewerWidgetOptions;
}

export interface ILogViewerWidget extends IWidgetBase {
  category: EWidgetCategory.LogViewer;
  metadata: ILogViewerWidgetData<ELogViewerScriptType>;
}

export enum EDefaultWidgetType {
  GraphSelect = "graph_select",
  About = "about",
  Preferences = "preferences",
  AppFolder = "app_folder",
  ExtensionStore = "extension_store",
}

export interface IDefaultWidget extends IWidgetBase {
  category: EWidgetCategory.Default;
  metadata: {
    type: EDefaultWidgetType;
  };
}

export interface IExtensionWidget extends IWidgetBase {
  category: EWidgetCategory.Extension;
  metadata: {
    name: string;
    versions: IListTenCloudStorePackage[];
  };
}

export type IWidget =
  | ITerminalWidget
  | IEditorWidget
  | ICustomConnectionWidget
  | ILogViewerWidget
  | IDefaultWidget
  | IExtensionWidget;
