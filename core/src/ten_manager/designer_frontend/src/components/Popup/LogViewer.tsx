//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import * as React from "react";
import { useTranslation } from "react-i18next";

import { LogViewerFrontStageWidget } from "@/components/Widget/LogViewerWidget";
import {
  EWidgetCategory,
  EWidgetDisplayType,
  type ILogViewerWidget,
} from "@/types/widgets";
import { useWidgetStore } from "@/store/widget";
import { GROUP_LOG_VIEWER_ID } from "@/constants/widgets";
import { CONTAINER_DEFAULT_ID } from "@/constants/widgets";

export const LogViewerPopupTitle = (props: {
  title?: string | React.ReactNode;
}) => {
  const { title } = props;
  const { t } = useTranslation();
  return (
    <div className="flex items-center gap-1.5">
      <span className="font-medium text-foreground/90 font-sans">
        {title || t("popup.logViewer.title")}
      </span>
    </div>
  );
};

export const LogViewerPopupContent = (props: { widget: ILogViewerWidget }) => {
  const { widget } = props;

  const { backstageWidgets, appendBackstageWidgetIfNotExists } =
    useWidgetStore();

  React.useEffect(() => {
    const targetBackstageWidget = backstageWidgets.find(
      (w) => w.widget_id === widget.widget_id
    ) as ILogViewerWidget | undefined;

    if (
      !targetBackstageWidget &&
      widget.metadata.scriptType &&
      widget.metadata.script
    ) {
      appendBackstageWidgetIfNotExists({
        container_id: CONTAINER_DEFAULT_ID,
        group_id: GROUP_LOG_VIEWER_ID,
        widget_id: widget.widget_id,

        category: EWidgetCategory.LogViewer,
        display_type: EWidgetDisplayType.Popup,

        title: <LogViewerPopupTitle title={widget.metadata.options?.title} />,
        metadata: widget.metadata,
      });
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [backstageWidgets, widget.metadata.scriptType, widget.metadata.script]);

  return (
    <LogViewerFrontStageWidget
      id={widget.widget_id}
      options={widget.metadata.options}
    />
  );
};
