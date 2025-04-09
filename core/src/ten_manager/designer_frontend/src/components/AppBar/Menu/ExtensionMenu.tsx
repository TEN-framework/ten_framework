//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import { useTranslation } from "react-i18next";
import { BlocksIcon, InfoIcon } from "lucide-react";

import {
  NavigationMenuContent,
  NavigationMenuItem,
  NavigationMenuLink,
  NavigationMenuTrigger,
} from "@/components/ui/NavigationMenu";
import { Separator } from "@/components/ui/Separator";
import { Button } from "@/components/ui/Button";
import { cn } from "@/lib/utils";
import {
  EDefaultWidgetType,
  EWidgetDisplayType,
  EWidgetCategory,
} from "@/types/widgets";
import { useWidgetStore } from "@/store/widget";
import {
  DOC_REF_POPUP_ID,
  EXTENSION_STORE_POPUP_ID,
} from "@/constants/widgets";
import { EDocLinkKey } from "@/types/doc";

export const ExtensionMenu = (props: {
  disableMenuClick?: boolean;
  idx: number;
  triggerListRef?: React.RefObject<HTMLButtonElement[]>;
}) => {
  const { disableMenuClick, idx, triggerListRef } = props;

  const { t } = useTranslation();
  const { appendWidgetIfNotExists, appendTabWidget } = useWidgetStore();

  const onOpenExtensionStore = () => {
    appendWidgetIfNotExists({
      id: EXTENSION_STORE_POPUP_ID,
      category: EWidgetCategory.Default,
      display_type: EWidgetDisplayType.Popup,
      metadata: {
        type: EDefaultWidgetType.ExtensionStore,
      },
    });
  };

  const openAbout = () => {
    appendTabWidget({
      id: DOC_REF_POPUP_ID,
      sub_id: EDocLinkKey.Extension,
      category: EWidgetCategory.Default,
      display_type: EWidgetDisplayType.PopupTab,
      metadata: {
        type: EDefaultWidgetType.DocRef,
        doc_link_key: EDocLinkKey.Extension,
      },
    });
  };

  return (
    <NavigationMenuItem>
      <NavigationMenuTrigger
        className="submenu-trigger"
        ref={(ref) => {
          if (triggerListRef?.current && ref) {
            triggerListRef.current[idx] = ref;
          }
        }}
        onClick={(e) => {
          if (disableMenuClick) {
            e.preventDefault();
          }
        }}
      >
        {t("header.menuExtension.title")}
      </NavigationMenuTrigger>
      <NavigationMenuContent
        className={cn("flex flex-col items-center px-1 py-1.5 gap-1.5")}
      >
        <NavigationMenuLink asChild>
          <Button
            className="w-full justify-start"
            variant="ghost"
            onClick={onOpenExtensionStore}
          >
            <BlocksIcon />
            {t("header.menuExtension.openExtensionStore")}
          </Button>
        </NavigationMenuLink>
        <Separator className="w-full" />
        <NavigationMenuLink asChild>
          <Button
            className="w-full justify-start"
            variant="ghost"
            onClick={openAbout}
          >
            <InfoIcon />
            {t("header.menuExtension.about")}
          </Button>
        </NavigationMenuLink>
      </NavigationMenuContent>
    </NavigationMenuItem>
  );
};
