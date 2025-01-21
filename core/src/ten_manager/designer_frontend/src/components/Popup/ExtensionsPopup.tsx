//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import React from "react";
import { useTranslation } from "react-i18next";
import { PinIcon } from "lucide-react";

import Popup from "@/components/Popup/Popup";
import { ExtensionsWidget } from "@/components/Widget/ExtensionsWidget";
import { useWidgetStore } from "@/store/widget";
import { EWidgetDisplayType } from "@/types/widgets";

const DEFAULT_WIDTH = 400;
const DEFAULT_HEIGHT = 600;

export default function ExtensionsPopup(props: { id: string }) {
  const { t } = useTranslation();
  const { updateWidgetDisplayType } = useWidgetStore();

  const handlePinToDock = () => {
    updateWidgetDisplayType(props.id, EWidgetDisplayType.Dock);
  };

  return (
    <Popup
      id={props.id}
      preventFocusSteal={true}
      resizable={true}
      initialWidth={DEFAULT_WIDTH}
      initialHeight={DEFAULT_HEIGHT}
      title={t("header.menu.extensions")}
      onClose={() => {}}
      customActions={[
        {
          id: "pin-to-dock",
          label: t("action.pinToDock"),
          Icon: PinIcon,
          onClick: handlePinToDock,
        },
      ]}
    >
      <ExtensionsWidget />
    </Popup>
  );
}
