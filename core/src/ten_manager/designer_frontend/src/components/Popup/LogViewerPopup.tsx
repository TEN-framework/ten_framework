import * as React from "react";
import { useTranslation } from "react-i18next";
import { PinIcon } from "lucide-react";

import Popup from "@/components/Popup/Popup";
import LogViewerWidget from "@/components/Widget/LogViewerWidget";
import { EWidgetDisplayType } from "@/types/widgets";
import { useWidgetStore } from "@/store/widget";

const DEFAULT_WIDTH = 800;
const DEFAULT_HEIGHT = 400;

export function LogViewerPopup(props: { id: string }) {
  const { id } = props;

  const { t } = useTranslation();
  const { updateWidgetDisplayType, removeWidget } = useWidgetStore();

  const handlePinToDock = () => {
    updateWidgetDisplayType(id, EWidgetDisplayType.Dock);
  };

  const handleClose = () => {
    removeWidget(id);
  };

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
      ]}
    >
      <LogViewerWidget />
    </Popup>
  );
}
