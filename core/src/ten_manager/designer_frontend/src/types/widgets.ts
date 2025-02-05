//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
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
  TerminalViewer = "terminal_viewer",
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

interface ILogViewerWidgetData {
  wsUrl: string;
  baseDir: string;
  scriptName: string;
  supportStop?: boolean;
}

export interface ILogViewerWidget extends IWidgetBase {
  category: EWidgetCategory.LogViewer;
  metadata: ILogViewerWidgetData;
}

interface ITerminalViewerWidgetData {
  wsUrl: string;
  baseDir: string;
  supportStop?: boolean;
}

export interface ITerminalViewerWidget extends IWidgetBase {
  category: EWidgetCategory.TerminalViewer;
  metadata: ITerminalViewerWidgetData;
}

export type IWidget =
  | ITerminalWidget
  | IEditorWidget
  | ICustomConnectionWidget
  | ILogViewerWidget
  | ITerminalViewerWidget;
