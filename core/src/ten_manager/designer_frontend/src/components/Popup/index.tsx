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
import TerminalPopup from "@/components/Popup/Terminal";
import EditorPopup from "@/components/Popup/Editor";
import CustomNodeConnPopup from "@/components/Popup/CustomNodeConn";
import { LogViewerPopup } from "@/components/Popup/LogViewer";
import { GraphSelectPopup } from "@/components/Popup/Default/GraphSelect";
import { AboutPopup } from "@/components/Popup/Default/About";
import { PreferencesPopup } from "@/components/Popup/Default/Preferences";
import {
  AppFolderPopup,
  LoadedAppsPopup,
  AppRunPopup,
  AppCreatePopup,
} from "@/components/Popup/Default/App";
import {
  ExtensionStorePopup,
  ExtensionPopup,
} from "@/components/Popup/Default/Extension";
import { DocRefPopup } from "@/components/Popup/Default/DocRef";
import { GraphPopup } from "@/components/Popup/Graph";
import { groupWidgetsById } from "@/components/Popup/utils";

export function GlobalPopups() {
  const { widgets, removeWidget } = useWidgetStore();

  const [
    ,
    [editorWidgetsMemo],
    [terminalWidgetsMemo],
    [customConnectionWidgetsMemo],
    [logViewerWidgetsMemo],
    [defaultWidgetsMemo, defaultSubTabWidgetsMemo],
    [extensionWidgetsMemo],
    [graphWidgetsMemo],
  ] = React.useMemo(() => {
    // get all widgets that are either popup or popup tab
    const popupAndSubTabWidgets = widgets.filter((widget) =>
      [EWidgetDisplayType.Popup, EWidgetDisplayType.PopupTab].includes(
        widget.display_type
      )
    );
    // 1. group editor widgets by id
    const editorRawWidgets = popupAndSubTabWidgets.filter(
      (widget) => widget.category === EWidgetCategory.Editor
    );
    const editorWidgets = groupWidgetsById(editorRawWidgets);

    // 2. group terminal widgets by id
    const terminalRawWidgets = popupAndSubTabWidgets.filter(
      (widget) => widget.category === EWidgetCategory.Terminal
    );
    const terminalWidgets = groupWidgetsById(terminalRawWidgets);

    // 3. group custom connection widgets by id
    const customConnectionRawWidgets = popupAndSubTabWidgets.filter(
      (widget) => widget.category === EWidgetCategory.CustomConnection
    );
    const customConnectionWidgets = groupWidgetsById(
      customConnectionRawWidgets
    );

    // 4. group log viewer widgets by id
    const logViewerRawWidgets = popupAndSubTabWidgets.filter(
      (widget) => widget.category === EWidgetCategory.LogViewer
    );
    const logViewerWidgets = groupWidgetsById(logViewerRawWidgets);

    // 5. group default widgets by id
    const defaultRawWidgets = popupAndSubTabWidgets.filter(
      (widget) => widget.category === EWidgetCategory.Default
    );
    const defaultWidgets = groupWidgetsById(defaultRawWidgets);

    // 6. group extension widgets by id
    const extensionRawWidgets = popupAndSubTabWidgets.filter(
      (widget) => widget.category === EWidgetCategory.Extension
    );
    const extensionWidgets = groupWidgetsById(extensionRawWidgets);

    // 7. group graph widgets by id
    const graphRawWidgets = popupAndSubTabWidgets.filter(
      (widget) => widget.category === EWidgetCategory.Graph
    );
    const graphWidgets = groupWidgetsById(graphRawWidgets);

    return [
      popupAndSubTabWidgets,
      editorWidgets,
      terminalWidgets,
      customConnectionWidgets,
      logViewerWidgets,
      defaultWidgets,
      extensionWidgets,
      graphWidgets,
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
          case EDefaultWidgetType.DocRef:
            return <DocRefPopup key={`DocRefPopup-${widget.id}`} />;
        }
      })}
      {defaultSubTabWidgetsMemo.map((widgets) => {
        if (widgets.length === 0) return null;
        const firstWidget = widgets[0];
        switch (firstWidget.metadata.type) {
          case EDefaultWidgetType.DocRef:
            return (
              <DocRefPopup
                key={`DocRefPopup-${firstWidget.id}`}
                tabs={widgets}
              />
            );
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
      {graphWidgetsMemo.map((widget) => (
        <GraphPopup
          key={`GraphPopup-${widget.id}`}
          id={widget.id}
          metadata={widget.metadata}
        />
      ))}
    </>
  );
}
