//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import * as React from "react";

import { useWidgetStore } from "@/store/widget";
import {
  EDefaultWidgetType,
  EWidgetCategory,
  EWidgetDisplayType,
} from "@/types/widgets";
import TerminalPopup from "@/components/Popup/TerminalPopup";
import EditorPopup from "@/components/Popup/EditorPopup";
import CustomNodeConnPopup from "@/components/Popup/CustomNodeConnPopup";
import { LogViewerPopup } from "@/components/Popup/LogViewerPopup";
import { GraphSelectPopup } from "@/components/Popup/GraphSelectPopup";
import { AboutPopup } from "@/components/Popup/AboutPopup";
import { PreferencesPopup } from "@/components/Popup/PreferencesPopup";
import {
  AppFolderPopup,
  LoadedAppsPopup,
  AppRunPopup,
  AppCreatePopup,
} from "@/components/Popup/AppPopup";
import {
  ExtensionStorePopup,
  ExtensionPopup,
} from "@/components/Popup/ExtensionPopup";
import { NodePopup } from "@/components/Popup/NodePopup";

export function GlobalPopups() {
  const { widgets, removeWidget } = useWidgetStore();

  const [
    ,
    editorWidgetsMemo,
    terminalWidgetsMemo,
    customConnectionWidgetsMemo,
    logViewerWidgetsMemo,
    defaultWidgetsMemo,
    extensionWidgetsMemo,
    nodeWidgetsMemo,
  ] = React.useMemo(() => {
    const popupWidgets = widgets.filter(
      (widget) => widget.display_type === EWidgetDisplayType.Popup
    );
    const editorWidgets = popupWidgets.filter(
      (widget) => widget.category === EWidgetCategory.Editor
    );
    const terminalWidgets = popupWidgets.filter(
      (widget) => widget.category === EWidgetCategory.Terminal
    );
    const customConnectionWidgets = popupWidgets.filter(
      (widget) => widget.category === EWidgetCategory.CustomConnection
    );
    const logViewerWidgets = popupWidgets.filter(
      (widget) => widget.category === EWidgetCategory.LogViewer
    );
    const defaultWidgets = popupWidgets.filter(
      (widget) => widget.category === EWidgetCategory.Default
    );
    const extensionWidgets = popupWidgets.filter(
      (widget) => widget.category === EWidgetCategory.Extension
    );
    const nodeWidgets = popupWidgets.filter(
      (widget) => widget.category === EWidgetCategory.Node
    );
    return [
      popupWidgets,
      editorWidgets,
      terminalWidgets,
      customConnectionWidgets,
      logViewerWidgets,
      defaultWidgets,
      extensionWidgets,
      nodeWidgets,
    ];
  }, [widgets]);

  return (
    <>
      {terminalWidgetsMemo.map((widget) => (
        <TerminalPopup
          id={widget.id}
          key={`TerminalPopup-${widget.id}`}
          data={widget.metadata}
          onClose={() => removeWidget(widget.id)}
        />
      ))}
      {editorWidgetsMemo.map((widget) => (
        <EditorPopup
          id={widget.id}
          key={`EditorPopup-${widget.id}`}
          data={widget.metadata}
          onClose={() => removeWidget(widget.id)}
          hasUnsavedChanges={widget.isEditing}
        />
      ))}
      {customConnectionWidgetsMemo.map((widget) => (
        <CustomNodeConnPopup
          id={widget.id}
          key={`CustomNodeConnPopup-${widget.id}`}
          source={widget.metadata.source}
          target={widget.metadata.target}
          filters={widget.metadata.filters}
          onClose={() => removeWidget(widget.id)}
        />
      ))}
      {logViewerWidgetsMemo.map((widget) => (
        <LogViewerPopup
          id={widget.id}
          key={`LogViewerPopup-${widget.id}`}
          data={widget.metadata}
          onStop={widget.metadata?.onStop}
        />
      ))}
      {defaultWidgetsMemo.map((widget) => {
        switch (widget.metadata.type) {
          case EDefaultWidgetType.GraphSelect:
            return <GraphSelectPopup key={`GraphSelectPopup-${widget.id}`} />;
          case EDefaultWidgetType.About:
            return <AboutPopup key={`AboutPopup-${widget.id}`} />;
          case EDefaultWidgetType.AppFolder:
            return <AppFolderPopup key={`AppPopup-${widget.id}`} />;
          case EDefaultWidgetType.AppCreate:
            return <AppCreatePopup key={`AppCreatePopup-${widget.id}`} />;
          case EDefaultWidgetType.AppsManager:
            return <LoadedAppsPopup key={`AppsManagerPopup-${widget.id}`} />;
          case EDefaultWidgetType.AppRun:
            return (
              <AppRunPopup
                key={`AppRunPopup-${widget.id}`}
                id={widget.id}
                data={widget.metadata}
              />
            );
          case EDefaultWidgetType.ExtensionStore:
            return (
              <ExtensionStorePopup key={`ExtensionStorePopup-${widget.id}`} />
            );
          case EDefaultWidgetType.Preferences:
            return <PreferencesPopup key={`PreferencesPopup-${widget.id}`} />;
        }
      })}
      {extensionWidgetsMemo.map((widget) => (
        <ExtensionPopup
          key={`ExtensionPopup-${widget.id}`}
          id={widget.id}
          name={widget.metadata.name}
          versions={widget.metadata.versions}
        />
      ))}
      {nodeWidgetsMemo.map((widget) => (
        <NodePopup
          key={`NodePopup-${widget.id}`}
          id={widget.id}
          metadata={widget.metadata}
        />
      ))}
    </>
  );
}
