//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import { EWidgetDisplayType, type IWidget } from "@/types/widgets";

export const groupWidgetsById = <T extends IWidget>(
  widgets: T[]
): [T[], T[][]] => {
  const singleWidgets = widgets.filter(
    (widget) => widget.display_type === EWidgetDisplayType.Popup
  );
  const tabWidgets = widgets.filter(
    (widget) => widget.display_type === EWidgetDisplayType.PopupTab
  );
  const groupedWidgets = tabWidgets.reduce(
    (acc, widget) => {
      if (acc[widget.id]) {
        acc[widget.id].push(widget);
      } else {
        acc[widget.id] = [widget];
      }
      return acc;
    },
    {} as Record<string, T[]>
  );
  return [singleWidgets, Object.values(groupedWidgets)];
};
