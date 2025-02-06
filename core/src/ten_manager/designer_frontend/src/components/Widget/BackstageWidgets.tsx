//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import { LogViewerBackstageWidget } from "@/components/Widget/LogViewerWidget";
import { useWidgetStore } from "@/store/widget";
import { EWidgetCategory } from "@/types/widgets";
import { TerminalViewerBackstageWidget } from "@/components/Widget/TerminalViewerWidget"; // eslint-disable-line max-len

export function BackstageWidgets() {
  const { backstageWidgets } = useWidgetStore();

  return (
    <>
      {backstageWidgets.map((widget) => {
        switch (widget.category) {
          case EWidgetCategory.LogViewer:
            return <LogViewerBackstageWidget key={widget.id} {...widget} />;
          case EWidgetCategory.TerminalViewer:
            return (
              <TerminalViewerBackstageWidget key={widget.id} {...widget} />
            );
          default:
            return null;
        }
      })}
    </>
  );
}
