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
  CONTAINER_DEFAULT_ID,
  APP_FOLDER_WIDGET_ID,
  APP_CREATE_WIDGET_ID,
  APPS_MANAGER_WIDGET_ID,
  DOC_REF_WIDGET_ID,
  GROUP_DOC_REF_ID,
} from "@/constants/widgets";
import { EDocLinkKey } from "@/types/doc";
import {
  AppCreatePopupTitle,
  AppFolderPopupTitle,
  LoadedAppsPopupTitle,
} from "@/components/Popup/Default/App";
import { DocRefPopupTitle } from "@/components/Popup/Default/DocRef";

export function AppMenu(props: {
  disableMenuClick?: boolean;
  idx: number;
  triggerListRef?: React.RefObject<HTMLButtonElement[]>;
}) {
  const { disableMenuClick, idx, triggerListRef } = props;

  const { t } = useTranslation();

  const { appendWidgetIfNotExists } = useWidgetStore();

  const openAppFolderPopup = () => {
    appendWidgetIfNotExists({
      container_id: CONTAINER_DEFAULT_ID,
      group_id: APP_FOLDER_WIDGET_ID,
      widget_id: APP_FOLDER_WIDGET_ID,

      category: EWidgetCategory.Default,
      display_type: EWidgetDisplayType.Popup,

      title: <AppFolderPopupTitle />,
      metadata: {
        type: EDefaultWidgetType.AppFolder,
      },
    });
  };

  const openAppsManagerPopup = () => {
    appendWidgetIfNotExists({
      container_id: CONTAINER_DEFAULT_ID,
      group_id: APPS_MANAGER_WIDGET_ID,
      widget_id: APPS_MANAGER_WIDGET_ID,

      category: EWidgetCategory.Default,
      display_type: EWidgetDisplayType.Popup,

      title: <LoadedAppsPopupTitle />,
      metadata: {
        type: EDefaultWidgetType.AppsManager,
      },
    });
  };

  const openAppCreatePopup = () => {
    appendWidgetIfNotExists({
      container_id: CONTAINER_DEFAULT_ID,
      group_id: APP_CREATE_WIDGET_ID,
      widget_id: APP_CREATE_WIDGET_ID,

      category: EWidgetCategory.Default,
      display_type: EWidgetDisplayType.Popup,

      title: <AppCreatePopupTitle />,
      metadata: {
        type: EDefaultWidgetType.AppCreate,
      },
    });
  };

  const openAbout = () => {
    appendWidgetIfNotExists({
      container_id: CONTAINER_DEFAULT_ID,
      group_id: GROUP_DOC_REF_ID,
      widget_id: DOC_REF_WIDGET_ID + "-" + EDocLinkKey.App,

      category: EWidgetCategory.Default,
      display_type: EWidgetDisplayType.Popup,

      title: <DocRefPopupTitle name={EDocLinkKey.App} />,
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
