//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import {
  HardDriveDownloadIcon,
  FolderMinusIcon,
  FolderPlusIcon,
  FolderSyncIcon,
  FolderTreeIcon,
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
} from "@/constants/widgets";

export function AppMenu() {
  const { t } = useTranslation();

  const { appendWidgetIfNotExists } = useWidgetStore();

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

  return (
    <>
      <NavigationMenuItem>
        <NavigationMenuTrigger className="submenu-trigger">
          {t("header.menu.app")}
        </NavigationMenuTrigger>
        <NavigationMenuContent
          className={cn("flex flex-col items-center px-1 py-1.5 gap-1.5")}
        >
          <NavigationMenuLink asChild>
            <Button
              className="w-full justify-start"
              variant="ghost"
              onClick={openAppFolderPopup}
            >
              <FolderPlusIcon className="w-4 h-4 me-2" />
              {t("header.menuApp.loadApp")}
            </Button>
          </NavigationMenuLink>
          <NavigationMenuLink asChild>
            <Button
              className="w-full justify-start"
              variant="ghost"
              onClick={openAppsManagerPopup}
            >
              <FolderMinusIcon className="w-4 h-4 me-2" />
              {t("header.menuApp.unloadApp")}
            </Button>
          </NavigationMenuLink>
          <NavigationMenuLink asChild>
            <Button
              className="w-full justify-start"
              variant="ghost"
              onClick={openAppsManagerPopup}
            >
              <FolderSyncIcon className="w-4 h-4 me-2" />
              {t("header.menuApp.reloadApp")}
            </Button>
          </NavigationMenuLink>
          <NavigationMenuLink asChild>
            <Button
              className="w-full justify-start"
              variant="ghost"
              onClick={openAppsManagerPopup}
            >
              <FolderTreeIcon className="w-4 h-4 me-2" />
              {t("header.menuApp.listAllLoadedApps")}
            </Button>
          </NavigationMenuLink>
          <Separator className="w-full" />
          <NavigationMenuLink asChild>
            <Button
              className="w-full justify-start"
              variant="ghost"
              // onClick={handleAppInstall}
            >
              <HardDriveDownloadIcon className="w-4 h-4 me-2" />
              {t("header.menu.installAll")}
            </Button>
          </NavigationMenuLink>
        </NavigationMenuContent>
      </NavigationMenuItem>
    </>
  );
}
