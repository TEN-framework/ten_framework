//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import React, { useRef } from "react";
import { PinIcon } from "lucide-react";
import { useTranslation } from "react-i18next";

import Popup from "@/components/Popup/Popup";
import TerminalWidget from "@/components/Widget/TerminalWidget";
import { EWidgetDisplayType } from "@/types/widgets";
import { useWidgetStore } from "@/store/widget";

const DEFAULT_WIDTH = 800;
const DEFAULT_HEIGHT = 400;

export interface TerminalData {
  title: string;
  url?: string;
}

interface TerminalPopupProps {
  id: string;
  data: TerminalData;
  onClose: () => void;
}

const TerminalPopup: React.FC<TerminalPopupProps> = ({ id, data, onClose }) => {
  const terminalRef = useRef<unknown>(null);

  const { t } = useTranslation();
  const { updateWidgetDisplayType } = useWidgetStore();

  const handleCollapseToggle = (isCollapsed: boolean) => {
    if (terminalRef.current) {
      (
        terminalRef.current as {
          handleCollapseToggle: (isCollapsed: boolean) => void;
        }
      ).handleCollapseToggle(isCollapsed);
    }
  };

  const handlePinToDock = () => {
    updateWidgetDisplayType(id, EWidgetDisplayType.Dock);
  };

  return (
    <Popup
      id={id}
      title={data.title}
      onClose={onClose}
      preventFocusSteal={true}
      className="flex-1 p-0"
      resizable={true}
      onCollapseToggle={handleCollapseToggle}
      initialWidth={DEFAULT_WIDTH}
      initialHeight={DEFAULT_HEIGHT}
      customActions={[
        {
          id: "pin-to-dock",
          label: t("action.pinToDock"),
          Icon: PinIcon,
          onClick: handlePinToDock,
        },
      ]}
    >
      <TerminalWidget
        ref={terminalRef}
        key={id}
        id={id}
        data={data}
        onClose={onClose}
      />
    </Popup>
  );
};

export default TerminalPopup;
