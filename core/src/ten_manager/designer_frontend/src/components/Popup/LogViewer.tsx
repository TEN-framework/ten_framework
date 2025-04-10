//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import * as React from "react";
import { useTranslation } from "react-i18next";
import { PinIcon, OctagonXIcon } from "lucide-react";

import { PopupBase } from "@/components/Popup/Base";
import { LogViewerFrontStageWidget } from "@/components/Widget/LogViewerWidget";
import {
  EWidgetCategory,
  EWidgetDisplayType,
  type ILogViewerWidget,
} from "@/types/widgets";
import { useWidgetStore } from "@/store/widget";
import { GROUP_LOG_VIEWER_ID, GROUP_TERMINAL_ID } from "@/constants/widgets";
import { CONTAINER_DEFAULT_ID } from "@/constants/widgets";

const DEFAULT_WIDTH = 800;
const DEFAULT_HEIGHT = 400;

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

  const { t } = useTranslation();
  const {
    updateWidgetDisplayType,
    removeWidget,
    backstageWidgets,
    appendBackstageWidgetIfNotExists,
    removeBackstageWidget,
    removeLogViewerHistory,
  } = useWidgetStore();

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

/** @deprecated */
export function LogViewerPopup(props: {
  id: string;
  data: ILogViewerWidget["metadata"];
  onStop?: () => void;
}) {
  const { id, data, onStop } = props;

  const { t } = useTranslation();
  const {
    updateWidgetDisplayType,
    removeWidget,
    backstageWidgets,
    appendBackstageWidgetIfNotExists,
    removeBackstageWidget,
    removeLogViewerHistory,
  } = useWidgetStore();

  const handlePinToDock = () => {
    updateWidgetDisplayType(id, EWidgetDisplayType.Dock);
  };

  const handleClose = () => {
    removeWidget(id);
    removeBackstageWidget(id);
    removeLogViewerHistory(id);
  };

  React.useEffect(() => {
    const targetBackstageWidget = backstageWidgets.find(
      (w) => w.widget_id === id
    ) as ILogViewerWidget | undefined;

    if (!targetBackstageWidget && data?.scriptType && data?.script) {
      appendBackstageWidgetIfNotExists({
        container_id: CONTAINER_DEFAULT_ID,
        group_id: GROUP_TERMINAL_ID,
        widget_id: id,

        category: EWidgetCategory.LogViewer,
        display_type: EWidgetDisplayType.Popup,

        title: <LogViewerPopupTitle title={data?.options?.title} />,
        metadata: data,
      });
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [backstageWidgets, data?.scriptType, data?.script, id]);

  return (
    <PopupBase
      id={id}
      title={
        <div className="flex items-center gap-1.5">
          <span className="font-medium text-foreground/90 font-sans">
            {data?.options?.title || t("popup.logViewer.title")}
          </span>
          {/* <span className="text-foreground/50 text-sm font-sans">
            {hasUnsavedChanges ? `(${t("action.unsaved")})` : ""}
          </span> */}
        </div>
      }
      onClose={handleClose}
      resizable={true}
      width={DEFAULT_WIDTH}
      height={DEFAULT_HEIGHT}
      contentClassName="p-0"
      customActions={[
        {
          id: "pin-to-dock",
          label: t("action.pinToDock"),
          Icon: PinIcon,
          onClick: handlePinToDock,
        },
        ...(onStop
          ? [
              {
                id: "stop",
                label: t("action.stop"),
                Icon: OctagonXIcon,
                onClick: onStop,
              },
            ]
          : []),
      ]}
    >
      <LogViewerFrontStageWidget id={id} options={data?.options} />
    </PopupBase>
  );
}
