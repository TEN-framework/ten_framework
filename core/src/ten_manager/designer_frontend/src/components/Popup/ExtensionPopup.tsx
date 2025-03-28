//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import { useTranslation } from "react-i18next";
import { PinIcon } from "lucide-react";

import { Popup } from "@/components/Popup/Popup";
import { EXTENSION_STORE_POPUP_ID } from "@/constants/widgets";
import { useWidgetStore } from "@/store/widget";
import {
  ExtensionStoreWidget,
  ExtensionWidget,
} from "@/components/Widget/ExtensionWidget";
import { EWidgetDisplayType } from "@/types/widgets";
import { IListTenCloudStorePackage } from "@/types/extension";
import { getCurrentWindowSize } from "@/utils/popup";

export const ExtensionStorePopup = () => {
  const { removeWidget, updateWidgetDisplayType } = useWidgetStore();
  const { t } = useTranslation();

  const handlePinToDock = () => {
    updateWidgetDisplayType(EXTENSION_STORE_POPUP_ID, EWidgetDisplayType.Dock);
  };

  const windowSize = getCurrentWindowSize();

  return (
    <Popup
      id={EXTENSION_STORE_POPUP_ID}
      title={t("extensionStore.title")}
      onClose={() => removeWidget(EXTENSION_STORE_POPUP_ID)}
      width={340}
      height={windowSize?.height ? windowSize?.height - 100 : 400}
      contentClassName="p-0"
      resizable
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
      height={400}
      width={600}
      contentClassName="p-0"
      resizable
    >
      <ExtensionWidget versions={versions} name={name} />
    </Popup>
  );
};
