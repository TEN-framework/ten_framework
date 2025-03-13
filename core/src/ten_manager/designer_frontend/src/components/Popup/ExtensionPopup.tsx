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
import {
  ExtensionStoreWidget,
  ExtensionWidget,
} from "@/components/Widget/ExtensionWidget";
import { EWidgetDisplayType } from "@/types/widgets";
import { IListTenCloudStorePackage } from "@/types/extension";

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

export const ExtensionPopup = (props: {
  id: string;
  name: string;
  versions: IListTenCloudStorePackage[];
}) => {
  const { id, name, versions } = props;
  const { removeWidget } = useWidgetStore();
  const { t } = useTranslation();

  return (
    <Popup
      id={id}
      title={t("extensionStore.extensionTitle", { name })}
      onClose={() => removeWidget(id)}
      initialHeight={400}
      initialWidth={600}
      contentClassName="p-0"
      resizable
      preventFocusSteal
    >
      <ExtensionWidget versions={versions} name={name} />
    </Popup>
  );
};
