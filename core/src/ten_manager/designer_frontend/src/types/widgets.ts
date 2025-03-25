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
  INSTALL = "install",
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
  [ELogViewerScriptType.INSTALL]: z.object({
    type: z.literal(ELogViewerScriptType.INSTALL),
    base_dir: z.string().default(""),
    name: z.string().default(""),
    pkg_type: z.string().default(""),
    pkg_name: z.string().default(""),
    pkg_version: z.string().default(""),
  }),
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
  postActions?: () => void;
}

export interface ILogViewerWidget extends IWidgetBase {
  category: EWidgetCategory.LogViewer;
  metadata: ILogViewerWidgetData<ELogViewerScriptType>;
}

export enum EDefaultWidgetType {
  GraphSelect = "graph_select",
  About = "about",
  AppFolder = "app_folder",
  AppsManager = "apps_manager",
  AppRun = "app_run",
  ExtensionStore = "extension_store",
  Preferences = "preferences",
}

export interface IDefaultWidget<T extends { type: EDefaultWidgetType }>
  extends IWidgetBase {
  category: EWidgetCategory.Default;
  metadata: T;
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
  | IDefaultWidget<{ type: EDefaultWidgetType; base_dir?: string }>
  | IExtensionWidget;
