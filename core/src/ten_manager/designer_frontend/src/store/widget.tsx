//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import { create } from "zustand";
import { devtools } from "zustand/middleware";

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
  backstageWidgets: IWidget[];
  appendBackstageWidget: (widget: IWidget) => void;
  appendBackstageWidgetIfNotExists: (widget: IWidget) => void;
  removeBackstageWidget: (widgetId: string) => void;
  removeBackstageWidgets: (widgetIds: string[]) => void;
  logViewerHistory: {
    [id: string]: {
      history: string[];
    };
  };
  appendLogViewerHistory: (id: string, history: string[]) => void;
  removeLogViewerHistory: (id: string) => void;
  removeLogViewerHistories: (ids: string[]) => void;
  terminalViewerHistory: {
    [id: string]: {
      history: string[];
    };
  };
  appendTerminalViewerHistory: (id: string, history: string[]) => void;
  removeTerminalViewerHistory: (id: string) => void;
  removeTerminalViewerHistories: (ids: string[]) => void;
}>()(
  devtools((set) => ({
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
    backstageWidgets: [],
    appendBackstageWidget: (widget: IWidget) =>
      set((state) => ({
        backstageWidgets: [...state.backstageWidgets, widget],
      })),
    appendBackstageWidgetIfNotExists: (widget: IWidget) =>
      set((state) => ({
        backstageWidgets: state.backstageWidgets.find((w) => w.id === widget.id)
          ? state.backstageWidgets
          : [...state.backstageWidgets, widget],
      })),
    removeBackstageWidget: (widgetId: string) =>
      set((state) => ({
        backstageWidgets: state.backstageWidgets.filter(
          (w) => w.id !== widgetId
        ),
      })),
    removeBackstageWidgets: (widgetIds: string[]) =>
      set((state) => ({
        backstageWidgets: state.backstageWidgets.filter(
          (w) => !widgetIds.includes(w.id)
        ),
      })),
    logViewerHistory: {},
    appendLogViewerHistory: (id: string, history: string[], override = false) =>
      set((state) => ({
        logViewerHistory: {
          ...state.logViewerHistory,
          [id]: {
            history: override
              ? history
              : [...(state.logViewerHistory[id]?.history || []), ...history],
          },
        },
      })),
    removeLogViewerHistory: (id: string) =>
      set((state) => ({
        logViewerHistory: Object.fromEntries(
          Object.entries(state.logViewerHistory).filter(([key]) => key !== id)
        ),
      })),
    removeLogViewerHistories: (ids: string[]) =>
      set((state) => ({
        logViewerHistory: Object.fromEntries(
          Object.entries(state.logViewerHistory).filter(
            ([key]) => !ids.includes(key)
          )
        ),
      })),
    terminalViewerHistory: {},
    appendTerminalViewerHistory: (
      id: string,
      history: string[],
      override = false
    ) =>
      set((state) => ({
        terminalViewerHistory: {
          ...state.terminalViewerHistory,
          [id]: {
            history: override
              ? history
              : [
                  ...(state.terminalViewerHistory[id]?.history || []),
                  ...history,
                ],
          },
        },
      })),
    removeTerminalViewerHistory: (id: string) =>
      set((state) => ({
        terminalViewerHistory: Object.fromEntries(
          Object.entries(state.terminalViewerHistory).filter(
            ([key]) => key !== id
          )
        ),
      })),
    removeTerminalViewerHistories: (ids: string[]) =>
      set((state) => ({
        terminalViewerHistory: Object.fromEntries(
          Object.entries(state.terminalViewerHistory).filter(
            ([key]) => !ids.includes(key)
          )
        ),
      })),
  }))
);
