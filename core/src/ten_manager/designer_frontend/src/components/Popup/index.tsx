//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import * as React from "react";

import { useWidgetStore } from "@/store/widget";
import { TerminalPopupContent } from "@/components/Popup/Terminal";
import { CustomNodeConnPopupContent } from "@/components/Popup/CustomNodeConn";
import { LogViewerPopupContent } from "@/components/Popup/LogViewer";
import { GraphPopupContent } from "@/components/Popup/Graph";
import { PopupTabContentDefault } from "@/components/Popup/Default";
import { ExtensionPopupContent } from "@/components/Popup/Default/Extension";
import { EditorPopupContent } from "@/components/Popup/Editor";
import {
  PopupBase,
  PopupTabsBar,
  PopupTabsBarItem,
  PopupTabsBarContent,
} from "@/components/Popup/Base";
import { groupWidgetsById } from "@/components/Popup/utils";
import { EWidgetCategory, IWidget } from "@/types/widgets";
import { cn } from "@/lib/utils";

const PopupWithTabs = (props: {
  widgets: IWidget[];
  groupId: string;
  containerId: string;
}) => {
  const { widgets, groupId, containerId } = props;

  const { removeWidgets } = useWidgetStore();

  const handleClose = React.useCallback(async () => {
    const widgetOnCloseActions = widgets
      .map((widget) => widget.actions?.onClose)
      .filter((action) => action !== undefined);
    await Promise.all(widgetOnCloseActions);
    removeWidgets(widgets.map((widget) => widget.widget_id));
  }, [removeWidgets, widgets]);

  const globalCustomActions = React.useMemo(() => {
    if (widgets.length > 1) return undefined;
    const firstWidget = widgets[0];
    return firstWidget.actions?.custom_actions;
  }, [widgets]);

  const globalTitle = React.useMemo(() => {
    if (widgets.length > 1) return undefined;
    const firstWidget = widgets[0];
    return firstWidget.title;
  }, [widgets]);

  return (
    <PopupBase
      id={`${containerId}-${groupId}`}
      onClose={handleClose}
      customActions={globalCustomActions}
      title={globalTitle}
      contentClassName={cn("p-0 flex flex-col")}
    >
      <PopupTabs widgets={widgets} />
    </PopupBase>
  );
};

const WidgetContentRenderMappings: Record<
  EWidgetCategory,
  React.ComponentType<{ widget: IWidget }>
> = {
  [EWidgetCategory.Default]: PopupTabContentDefault as React.ComponentType<{
    widget: IWidget;
  }>,
  [EWidgetCategory.Extension]: ExtensionPopupContent as React.ComponentType<{
    widget: IWidget;
  }>,
  [EWidgetCategory.Graph]: GraphPopupContent as React.ComponentType<{
    widget: IWidget;
  }>,
  [EWidgetCategory.CustomConnection]:
    CustomNodeConnPopupContent as React.ComponentType<{ widget: IWidget }>,
  [EWidgetCategory.Terminal]: TerminalPopupContent as React.ComponentType<{
    widget: IWidget;
  }>,
  [EWidgetCategory.LogViewer]: LogViewerPopupContent as React.ComponentType<{
    widget: IWidget;
  }>,
  [EWidgetCategory.Editor]: EditorPopupContent as React.ComponentType<{
    widget: IWidget;
  }>,
};

const PopupTabs = (props: { widgets: IWidget[] }) => {
  const { widgets } = props;

  const { removeWidget } = useWidgetStore();

  const [activeWidgetId, setActiveWidgetId] = React.useState<string>(
    widgets[0].widget_id
  );

  const showTabsBar = React.useMemo(() => {
    return widgets?.length > 1;
  }, [widgets]);

  return (
    <>
      {showTabsBar && (
        <PopupTabsBar>
          {widgets.map((widget) => (
            <PopupTabsBarItem
              key={"PopupTabsBarItem" + widget.widget_id}
              id={widget.widget_id}
              isActive={widget.widget_id === activeWidgetId}
              onSelect={(id) => setActiveWidgetId(id)}
              onClose={() => {
                removeWidget(widget.widget_id);
                setActiveWidgetId(widgets[0].widget_id);
              }}
            >
              {widget.title}
            </PopupTabsBarItem>
          ))}
        </PopupTabsBar>
      )}
      {widgets.map((widget) => {
        const Renderer = WidgetContentRenderMappings[widget.category];

        if (!Renderer) return null;

        return (
          <PopupTabsBarContent
            key={"PopupTabsBarContent" + widget.widget_id}
            isActive={widget.widget_id === activeWidgetId}
            fullHeight={widgets.length === 1}
          >
            <Renderer widget={widget} />
          </PopupTabsBarContent>
        );
      })}
    </>
  );
};

export function GlobalPopups() {
  const { widgets } = useWidgetStore();

  const groupedWidgets = React.useMemo(() => {
    return groupWidgetsById(widgets);
  }, [widgets]);

  const containerIds = React.useMemo(() => {
    return Object.keys(groupedWidgets);
  }, [groupedWidgets]);

  const getGroupByContainerId = React.useCallback(
    (containerId: string) => {
      const targetGroups = groupedWidgets[containerId];
      if (!targetGroups) {
        return [];
      }
      return Object.keys(targetGroups).map((groupId) => {
        return {
          groupId,
          widgets: targetGroups[groupId],
        };
      });
    },
    [groupedWidgets]
  );

  return (
    <>
      {containerIds.map((containerId) => (
        <React.Fragment key={containerId}>
          {getGroupByContainerId(containerId).map((groupObj) => (
            <PopupWithTabs
              key={groupObj.groupId}
              widgets={groupObj.widgets}
              groupId={groupObj.groupId}
              containerId={containerId}
            />
          ))}
        </React.Fragment>
      ))}
    </>
  );
}
