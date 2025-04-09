//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import {
  FolderPlusIcon,
  FolderCogIcon,
  FolderOpenIcon,
  InfoIcon,
} from "lucide-react";
import { useTranslation } from "react-i18next";

import {
  NavigationMenuContent,
  NavigationMenuItem,
  NavigationMenuLink,
  NavigationMenuTrigger,
} from "@/components/ui/NavigationMenu";
import { Separator } from "@/components/ui/Separator";
import { Button } from "@/components/ui/Button";
import { cn } from "@/lib/utils";
import { useWidgetStore } from "@/store";
import {
  EWidgetCategory,
  EWidgetDisplayType,
  EDefaultWidgetType,
} from "@/types/widgets";
import {
  APP_FOLDER_POPUP_ID,
  APPS_MANAGER_POPUP_ID,
  APP_CREATE_POPUP_ID,
  DOC_REF_POPUP_ID,
} from "@/constants/widgets";
import { EDocLinkKey } from "@/types/doc";

export function AppMenu(props: {
  disableMenuClick?: boolean;
  idx: number;
  triggerListRef?: React.RefObject<HTMLButtonElement[]>;
}) {
  const { disableMenuClick, idx, triggerListRef } = props;

  const { t } = useTranslation();

  const { appendWidgetIfNotExists, appendTabWidget } = useWidgetStore();

  const openAppFolderPopup = () => {
    appendWidgetIfNotExists({
      id: APP_FOLDER_POPUP_ID,
      category: EWidgetCategory.Default,
      display_type: EWidgetDisplayType.Popup,
      metadata: {
        type: EDefaultWidgetType.AppFolder,
      },
    });
  };

  const openAppsManagerPopup = () => {
    appendWidgetIfNotExists({
      id: APPS_MANAGER_POPUP_ID,
      category: EWidgetCategory.Default,
      display_type: EWidgetDisplayType.Popup,
      metadata: {
        type: EDefaultWidgetType.AppsManager,
      },
    });
  };

  const openAppCreatePopup = () => {
    appendWidgetIfNotExists({
      id: APP_CREATE_POPUP_ID,
      category: EWidgetCategory.Default,
      display_type: EWidgetDisplayType.Popup,
      metadata: {
        type: EDefaultWidgetType.AppCreate,
      },
    });
  };

  const openAbout = () => {
    appendTabWidget({
      id: DOC_REF_POPUP_ID,
      sub_id: EDocLinkKey.App,
      category: EWidgetCategory.Default,
      display_type: EWidgetDisplayType.PopupTab,
      metadata: {
        type: EDefaultWidgetType.DocRef,
        doc_link_key: EDocLinkKey.App,
      },
    });
  };

  return (
    <>
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
          {t("header.menuApp.title")}
        </NavigationMenuTrigger>
        <NavigationMenuContent
          className={cn("flex flex-col items-center px-1 py-1.5 gap-1.5")}
        >
          <NavigationMenuLink asChild>
            <Button
              className="w-full justify-start"
              variant="ghost"
              onClick={openAppCreatePopup}
            >
              <FolderPlusIcon className="w-4 h-4 me-2" />
              {t("header.menuApp.createApp")}
            </Button>
          </NavigationMenuLink>
          <NavigationMenuLink asChild>
            <Button
              className="w-full justify-start"
              variant="ghost"
              onClick={openAppFolderPopup}
            >
              <FolderOpenIcon className="w-4 h-4 me-2" />
              {t("header.menuApp.loadApp")}
            </Button>
          </NavigationMenuLink>
          <NavigationMenuLink asChild>
            <Button
              className="w-full justify-start"
              variant="ghost"
              onClick={openAppsManagerPopup}
            >
              <FolderCogIcon className="w-4 h-4 me-2" />
              {t("header.menuApp.manageLoadedApps")}
            </Button>
          </NavigationMenuLink>
          <Separator className="w-full" />
          <NavigationMenuLink asChild>
            <Button
              className="w-full justify-start"
              variant="ghost"
              onClick={openAbout}
            >
              <InfoIcon className="w-4 h-4 me-2" />
              {t("header.menuApp.about")}
            </Button>
          </NavigationMenuLink>
        </NavigationMenuContent>
      </NavigationMenuItem>
    </>
  );
}
