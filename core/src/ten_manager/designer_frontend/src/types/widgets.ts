//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import { z } from "zod";

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
  GraphSelect = "graph_select",
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
  START = "start",
  INSTALL = "install",
}

export const LogViewerScriptSchemaMap = {
  [ELogViewerScriptType.START]: z.object({
    type: z.literal(ELogViewerScriptType.START),
    base_dir: z.string().default(""),
    name: z.string().default(""),
  }),
  [ELogViewerScriptType.INSTALL]: z.object({
    type: z.literal(ELogViewerScriptType.INSTALL),
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

export interface IGraphSelectWidget extends IWidgetBase {
  category: EWidgetCategory.GraphSelect;
  metadata: null;
}

export type IWidget =
  | ITerminalWidget
  | IEditorWidget
  | ICustomConnectionWidget
  | ILogViewerWidget
  | IGraphSelectWidget;
