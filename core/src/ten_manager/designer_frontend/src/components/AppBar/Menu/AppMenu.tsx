//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import { toast } from "sonner";
import {
  FolderOpenIcon,
  CogIcon,
  PlayIcon,
  HardDriveDownloadIcon,
} from "lucide-react";
import { useTranslation } from "react-i18next";

import {
  NavigationMenuContent,
  NavigationMenuItem,
  NavigationMenuLink,
  NavigationMenuTrigger,
} from "@/components/ui/NavigationMenu";
import { Button } from "@/components/ui/Button";
import { cn } from "@/lib/utils";
import { getBaseDir } from "@/api/services/fileSystem";
import { useWidgetStore, useAppStore } from "@/store";
import {
  ELogViewerScriptType,
  EWidgetCategory,
  EWidgetDisplayType,
  EDefaultWidgetType,
} from "@/types/widgets";
import {
  TEN_DEFAULT_BACKEND_WS_ENDPOINT,
  TEN_PATH_WS_RUN_SCRIPT,
  TEN_PATH_WS_BUILTIN_FUNCTION,
} from "@/constants";
import {
  APP_FOLDER_POPUP_ID,
  APP_PREFERENCES_POPUP_ID,
} from "@/components/Popup/AppPopup";

export function AppMenu() {
  const { t } = useTranslation();

  const { appendWidgetIfNotExists } = useWidgetStore();
  const { runScript } = useAppStore();

  const handleAppStart = async () => {
    try {
      const baseDirData = await getBaseDir();
      const baseDir = baseDirData?.base_dir;
      if (!baseDir) {
        toast.error("Base directory is not set.");
        return;
      }

      const scriptName = runScript;

      appendWidgetIfNotExists({
        id: "app-start-" + Date.now(),
        category: EWidgetCategory.LogViewer,
        display_type: EWidgetDisplayType.Popup,

        metadata: {
          wsUrl: TEN_DEFAULT_BACKEND_WS_ENDPOINT + TEN_PATH_WS_RUN_SCRIPT,
          scriptType: ELogViewerScriptType.START,
          script: {
            type: ELogViewerScriptType.START,
            base_dir: baseDir,
            name: scriptName,
          },
          onStop: () => {
            console.log("app-start-widget-closed", baseDir, scriptName);
          },
        },
      });
    } catch (err) {
      console.error(err);
      toast.error("Failed to get base directory.");
    }
  };

  const handleAppInstall = async () => {
    try {
      const baseDirData = await getBaseDir();
      const baseDir = baseDirData?.base_dir;
      if (!baseDir) {
        toast.error("Base directory is not set.");
        return;
      }

      appendWidgetIfNotExists({
        id: "app-install-" + Date.now(),
        category: EWidgetCategory.LogViewer,
        display_type: EWidgetDisplayType.Popup,
        metadata: {
          wsUrl: TEN_DEFAULT_BACKEND_WS_ENDPOINT + TEN_PATH_WS_BUILTIN_FUNCTION,
          scriptType: ELogViewerScriptType.INSTALL_ALL,
          script: {
            type: ELogViewerScriptType.INSTALL_ALL,
            base_dir: baseDir,
          },
          options: {
            disableSearch: true,
            title: t("popup.logViewer.appInstall"),
          },
          // onStop: () => {
          //   console.log("app-install-widget-closed", baseDir);
          // },
        },
      });
    } catch (err) {
      console.error(err);
      toast.error("Failed to get base directory.");
    }
  };

  const openPreferencesPopup = () => {
    appendWidgetIfNotExists({
      id: APP_PREFERENCES_POPUP_ID,
      category: EWidgetCategory.Default,
      display_type: EWidgetDisplayType.Popup,
      metadata: {
        type: EDefaultWidgetType.Preferences,
      },
    });
  };

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
              <FolderOpenIcon className="w-4 h-4 me-2" />
              {t("header.menu.openAppFolder")}
            </Button>
          </NavigationMenuLink>

          <NavigationMenuLink asChild>
            <Button
              className="w-full justify-start"
              variant="ghost"
              onClick={handleAppStart}
            >
              <PlayIcon className="w-4 h-4 me-2" />
              {t("header.menu.start")}
            </Button>
          </NavigationMenuLink>

          <NavigationMenuLink asChild>
            <Button
              className="w-full justify-start"
              variant="ghost"
              onClick={handleAppInstall}
            >
              <HardDriveDownloadIcon className="w-4 h-4 me-2" />
              {t("header.menu.installAll")}
            </Button>
          </NavigationMenuLink>

          <NavigationMenuLink asChild>
            <Button
              className="w-full justify-start"
              variant="ghost"
              onClick={openPreferencesPopup}
            >
              <CogIcon className="w-4 h-4 me-2" />
              {t("header.menu.preferences")}
            </Button>
          </NavigationMenuLink>
        </NavigationMenuContent>
      </NavigationMenuItem>
    </>
  );
}
