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
import { AppFolderPopup, PreferencesPopup } from "@/components/Popup/AppPopup";

export function GlobalPopups() {
  const { widgets, removeWidget } = useWidgetStore();

  const [
    ,
    editorWidgetsMemo,
    terminalWidgetsMemo,
    customConnectionWidgetsMemo,
    logViewerWidgetsMemo,
    defaultWidgetsMemo,
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
    return [
      popupWidgets,
      editorWidgets,
      terminalWidgets,
      customConnectionWidgets,
      logViewerWidgets,
      defaultWidgets,
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
          case EDefaultWidgetType.Preferences:
            return <PreferencesPopup key={`PreferencesPopup-${widget.id}`} />;
        }
      })}
    </>
  );
}
