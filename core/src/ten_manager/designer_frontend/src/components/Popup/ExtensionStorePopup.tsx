//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import { useTranslation } from "react-i18next";
import { PinIcon } from "lucide-react";

import Popup from "@/components/Popup/Popup";
import { EXTENSION_STORE_POPUP_ID } from "@/constants/widgets";
import { useWidgetStore } from "@/store/widget";
import { ExtensionStoreWidget } from "@/components/Widget/ExtensionStoreWidget";
import { EWidgetDisplayType } from "@/types/widgets";

export const ExtensionStorePopup = () => {
  const { removeWidget, updateWidgetDisplayType } = useWidgetStore();
  const { t } = useTranslation();

  const handlePinToDock = () => {
    updateWidgetDisplayType(EXTENSION_STORE_POPUP_ID, EWidgetDisplayType.Dock);
  };

  return (
    <Popup
      id={EXTENSION_STORE_POPUP_ID}
      title={t("extensionStore.title")}
      onClose={() => removeWidget(EXTENSION_STORE_POPUP_ID)}
      initialHeight={400}
      initialWidth={340}
      contentClassName="p-0"
      resizable
      preventFocusSteal
      customActions={[
        {
          id: "pin-to-dock",
          label: t("action.pinToDock"),
          Icon: PinIcon,
          onClick: handlePinToDock,
        },
      ]}
    >
      <ExtensionStoreWidget />
    </Popup>
  );
};
