//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import { z } from "zod";
import { IListTenCloudStorePackage } from "@/types/extension";
import { EConnectionType, EGraphActions } from "@/types/graphs";
import { TCustomNode } from "@/types/flow";
import { EDocLinkKey } from "@/types/doc";

export enum EWidgetDisplayType {
  Popup = "popup",
  Dock = "dock",
  PopupTab = "popup_tab",
}

export enum EWidgetCategory {
  Terminal = "terminal",
  Editor = "editor",
  CustomConnection = "custom_connection",
  LogViewer = "log_viewer",
  Extension = "extension",
  Default = "default",
  Graph = "graph",
}

// 0. Widget Base

export interface IWidgetBase<T> {
  // The container id of the groups(of tabs/widgets)
  // container > group > widget
  container_id: string;
  // The group id of the tabs/widgets
  group_id: string;
  // The widget id of the tab/widget
  widget_id: string;
  // The display type of the widget
  display_type: EWidgetDisplayType;
  // The title of the tab/widget
  title?: string | React.ReactNode;
  // The metadata of the tab/widget
  metadata: T;
}

// 1. Terminal Widget
export interface ITerminalWidgetData {
  title: string;
  url?: string;
}

export interface ITerminalWidget extends IWidgetBase<ITerminalWidgetData> {
  category: EWidgetCategory.Terminal;
}

// 2. Editor Widget
export interface IEditorWidgetData {
  title: string;
  url: string; // The url (path) of the editor to display.
  content: string; // The content of the editor to display.
  isEditing?: boolean;
}

export interface IEditorWidget extends IWidgetBase<IEditorWidgetData> {
  category: EWidgetCategory.Editor;
}

// 3. Custom Connection Widget
export interface ICustomConnectionWidgetData {
  id: string;
  source: string;
  target?: string;
  filters?: {
    type?: EConnectionType;
    source?: boolean;
    target?: boolean;
  };
}

export interface ICustomConnectionWidget
  extends IWidgetBase<ICustomConnectionWidgetData> {
  category: EWidgetCategory.CustomConnection;
}

// 4. Graph Widget
export interface IGraphWidgetData {
  type: EGraphActions;
  base_dir: string;
  graph_name?: string;
  app_uri?: string;
  node?: TCustomNode;
}

export interface IGraphWidget extends IWidgetBase<IGraphWidgetData> {
  category: EWidgetCategory.Graph;
}

// 5. Log Viewer Widget
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

export interface ILogViewerWidget
  extends IWidgetBase<ILogViewerWidgetData<ELogViewerScriptType>> {
  category: EWidgetCategory.LogViewer;
}

// 6. Default Widget
export enum EDefaultWidgetType {
  GraphSelect = "graph_select",
  About = "about",
  AppFolder = "app_folder",
  AppsManager = "apps_manager",
  AppRun = "app_run",
  ExtensionStore = "extension_store",
  Preferences = "preferences",
  AppCreate = "app_create",
  DocRef = "doc_ref",
}

export interface IDefaultWidgetData {
  type: EDefaultWidgetType;
  base_dir?: string;
  scripts?: string[];
  doc_link_key?: EDocLinkKey;
}

export interface IDefaultWidget extends IWidgetBase<IDefaultWidgetData> {
  category: EWidgetCategory.Default;
}

// 7. Extension Widget
export interface IExtensionWidgetData {
  name: string;
  versions: IListTenCloudStorePackage[];
}

export interface IExtensionWidget extends IWidgetBase<IExtensionWidgetData> {
  category: EWidgetCategory.Extension;
}

// --- All Widget Types ---

export type IWidget =
  | ITerminalWidget
  | IEditorWidget
  | ICustomConnectionWidget
  | IGraphWidget
  | ILogViewerWidget
  | IDefaultWidget
  | IExtensionWidget;
