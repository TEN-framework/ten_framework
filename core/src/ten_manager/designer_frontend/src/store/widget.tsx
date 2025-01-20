import { create } from "zustand";

import {
  EWidgetCategory,
  type EWidgetDisplayType,
  type IWidget,
} from "@/types/widgets";

export const useWidgetStore = create<{
  widgets: IWidget[];
  appendWidget: (widget: IWidget) => void;
  appendWidgetIfNotExists: (widget: IWidget) => void;
  removeWidget: (widgetId: string) => void;
  removeWidgets: (widgetIds: string[]) => void;
  updateWidgetDisplayType: (
    widgetId: string,
    displayType: EWidgetDisplayType
  ) => void;
  updateEditorStatus: (widgetId: string, isEditing: boolean) => void;
}>((set) => ({
  widgets: [],
  appendWidget: (widget: IWidget) =>
    set((state) => ({ widgets: [...state.widgets, widget] })),
  appendWidgetIfNotExists: (widget: IWidget) =>
    set((state) => ({
      widgets: state.widgets.find((w) => w.id === widget.id)
        ? state.widgets
        : [...state.widgets, widget],
    })),
  removeWidget: (widgetId: string) =>
    set((state) => ({
      widgets: state.widgets.filter((w) => w.id !== widgetId),
    })),
  removeWidgets: (widgetIds: string[]) =>
    set((state) => ({
      widgets: state.widgets.filter((w) => !widgetIds.includes(w.id)),
    })),
  updateWidgetDisplayType: (
    widgetId: string,
    displayType: EWidgetDisplayType
  ) =>
    set((state) => ({
      widgets: state.widgets.map((w) =>
        w.id === widgetId ? { ...w, display_type: displayType } : w
      ),
    })),
  updateEditorStatus: (widgetId: string, isEditing: boolean) =>
    set((state) => ({
      widgets: state.widgets.map((w) =>
        w.id === widgetId && w.category === EWidgetCategory.Editor
          ? { ...w, isEditing }
          : w
      ),
    })),
}));
