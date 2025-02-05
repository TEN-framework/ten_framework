//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import * as React from "react";
import { useTranslation } from "react-i18next";
import { PinIcon, OctagonXIcon } from "lucide-react";

import Popup from "@/components/Popup/Popup";
import {
  LogViewerFrontStageWidget,
  type ILogViewerWidgetProps,
} from "@/components/Widget/LogViewerWidget";
import {
  EWidgetCategory,
  EWidgetDisplayType,
  type ILogViewerWidget,
} from "@/types/widgets";
import { useWidgetStore } from "@/store/widget";

const DEFAULT_WIDTH = 800;
const DEFAULT_HEIGHT = 400;

export interface ILogViewerPopupProps extends ILogViewerWidgetProps {
  supportStop?: boolean;
}

export function LogViewerPopup(props: ILogViewerPopupProps) {
  const { id, data, supportStop = false } = props;

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
    const targetBackstageWidget = backstageWidgets.find((w) => w.id === id) as
      | ILogViewerWidget
      | undefined;

    if (
      !targetBackstageWidget &&
      data?.wsUrl &&
      data?.baseDir &&
      data?.scriptName
    ) {
      appendBackstageWidgetIfNotExists({
        id,
        category: EWidgetCategory.LogViewer,
        display_type: EWidgetDisplayType.Popup,
        metadata: {
          wsUrl: data.wsUrl,
          baseDir: data.baseDir,
          scriptName: data.scriptName,
        },
      });
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [backstageWidgets, data?.baseDir, data?.scriptName, data?.wsUrl, id]);

  return (
    <Popup
      id={id}
      title={
        <div className="flex items-center gap-1.5">
          <span className="font-medium text-foreground/90 font-sans">
            {t("popup.logViewer.title")}
          </span>
          {/* <span className="text-foreground/50 text-sm font-sans">
            {hasUnsavedChanges ? `(${t("action.unsaved")})` : ""}
          </span> */}
        </div>
      }
      onClose={handleClose}
      preventFocusSteal={true}
      resizable={true}
      initialWidth={DEFAULT_WIDTH}
      initialHeight={DEFAULT_HEIGHT}
      contentClassName="p-0"
      customActions={[
        {
          id: "pin-to-dock",
          label: t("action.pinToDock"),
          Icon: PinIcon,
          onClick: handlePinToDock,
        },
        ...(supportStop
          ? [
              {
                id: "stop",
                label: t("action.stop"),
                Icon: OctagonXIcon,
                onClick: () => {
                  console.log("stop");
                },
              },
            ]
          : []),
      ]}
    >
      <LogViewerFrontStageWidget id={id} data={data} />
    </Popup>
  );
}
