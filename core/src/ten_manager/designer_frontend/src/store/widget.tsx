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
  EWidgetDisplayType,
  TWidgetCustomAction,
  type IWidget,
} from "@/types/widgets";
import { getZodDefaults } from "@/utils";
import { PREFERENCES_SCHEMA_LOG } from "@/types/apps";
import { dispatchBringToFrontPopup } from "@/utils/popup";

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
  updateWidgetDisplayTypeBulk: (
    widgetIds: string[],
    displayType: EWidgetDisplayType
  ) => void;
  appendWidgetCustomAction: (
    widgetId: string,
    action: TWidgetCustomAction
  ) => void;

  // editor ---
  updateEditorStatus: (widgetId: string, isContentChanged: boolean) => void;

  // backstage(ws) ---
  backstageWidgets: IWidget[];
  appendBackstageWidget: (widget: IWidget) => void;
  appendBackstageWidgetIfNotExists: (widget: IWidget) => void;
  removeBackstageWidget: (widgetId: string) => void;
  removeBackstageWidgets: (widgetIds: string[]) => void;

  // log viewer ---
  logViewerHistory: {
    [id: string]: {
      history: string[];
      maxLength: number;
    };
  };
  appendLogViewerHistory: (
    id: string,
    history: string[],
    options?: { override?: boolean; maxLength?: number }
  ) => void;
  removeLogViewerHistory: (id: string) => void;
  removeLogViewerHistories: (ids: string[]) => void;

  // extension store ---
  extSearch: string;
  setExtSearch: (search: string) => void;
  extFilter: {
    showUninstalled: boolean;
    showInstalled: boolean;
    sort: "default" | "name" | "name-desc";
    type: string[];
  };
  updateExtFilter: (filter: {
    showUninstalled?: boolean;
    showInstalled?: boolean;
    sort?: "default" | "name" | "name-desc";
    type?: string[];
  }) => void;
}>()(
  devtools((set) => ({
    widgets: [],
    appendWidget: (widget: IWidget) =>
      set((state) => ({ widgets: [...state.widgets, widget] })),
    appendWidgetIfNotExists: (widget: IWidget) => {
      set((state) => ({
        widgets: state.widgets.find(
          (w) =>
            w.container_id === widget.container_id &&
            w.group_id === widget.group_id &&
            w.widget_id === widget.widget_id
        )
          ? state.widgets
          : [...state.widgets, widget],
      }));
      dispatchBringToFrontPopup(widget.widget_id, widget.group_id);
    },
    removeWidget: (widgetId: string) =>
      set((state) => ({
        widgets: state.widgets.filter((w) => w.widget_id !== widgetId),
      })),
    removeWidgets: (widgetIds: string[]) =>
      set((state) => ({
        widgets: state.widgets.filter((w) => !widgetIds.includes(w.widget_id)),
      })),
    updateWidgetDisplayType: (
      widgetId: string,
      displayType: EWidgetDisplayType
    ) =>
      set((state) => ({
        widgets: state.widgets.map((w) =>
          w.widget_id === widgetId ? { ...w, display_type: displayType } : w
        ),
      })),
    updateWidgetDisplayTypeBulk: (
      widgetIds: string[],
      displayType: EWidgetDisplayType
    ) =>
      set((state) => ({
        widgets: state.widgets.map((w) =>
          widgetIds.includes(w.widget_id)
            ? { ...w, display_type: displayType }
            : w
        ),
      })),
    appendWidgetCustomAction: (widgetId: string, action: TWidgetCustomAction) =>
      set((state) => ({
        widgets: state.widgets.map((w) => {
          if (w.widget_id === widgetId) {
            if (w.actions) {
              w.actions.custom_actions = [
                ...(w.actions.custom_actions || []),
                action,
              ];
            } else {
              w.actions = {
                custom_actions: [action],
              };
            }
          }
          return w;
        }),
      })),

    // editor ---
    updateEditorStatus: (widgetId: string, isContentChanged: boolean) =>
      set((state) => ({
        widgets: state.widgets.map((w) =>
          w.widget_id === widgetId && w.category === EWidgetCategory.Editor
            ? { ...w, metadata: { ...w.metadata, isContentChanged } }
            : w
        ),
      })),

    // backstage(ws) ---
    backstageWidgets: [],
    appendBackstageWidget: (widget: IWidget) =>
      set((state) => ({
        backstageWidgets: [...state.backstageWidgets, widget],
      })),
    appendBackstageWidgetIfNotExists: (widget: IWidget) =>
      set((state) => ({
        backstageWidgets: state.backstageWidgets.find(
          (w) => w.widget_id === widget.widget_id
        )
          ? state.backstageWidgets
          : [...state.backstageWidgets, widget],
      })),
    removeBackstageWidget: (widgetId: string) =>
      set((state) => ({
        backstageWidgets: state.backstageWidgets.filter(
          (w) => w.widget_id !== widgetId
        ),
      })),
    removeBackstageWidgets: (widgetIds: string[]) =>
      set((state) => ({
        backstageWidgets: state.backstageWidgets.filter(
          (w) => !widgetIds.includes(w.widget_id)
        ),
      })),

    // log viewer ---
    logViewerHistory: {},
    appendLogViewerHistory: (
      id: string,
      history: string[],
      options?: { override?: boolean; maxLength?: number }
    ) =>
      set((state) => ({
        logViewerHistory: {
          ...state.logViewerHistory,
          [id]: {
            history: (() => {
              const maxLength =
                options?.maxLength ||
                getZodDefaults(PREFERENCES_SCHEMA_LOG).logviewer_line_size;
              const newHistory = options?.override
                ? history
                : [...(state.logViewerHistory[id]?.history || []), ...history];
              return newHistory.slice(-maxLength);
            })(),
            maxLength:
              options?.maxLength ||
              getZodDefaults(PREFERENCES_SCHEMA_LOG).logviewer_line_size,
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

    // extension store ---
    extSearch: "",
    setExtSearch: (search: string) => set({ extSearch: search }),
    extFilter: {
      showUninstalled: true,
      showInstalled: true,
      sort: "default",
      type: [],
    },
    updateExtFilter: (filter: {
      showUninstalled?: boolean;
      showInstalled?: boolean;
      sort?: "default" | "name" | "name-desc";
      type?: string[];
    }) => set((state) => ({ extFilter: { ...state.extFilter, ...filter } })),
  }))
);
