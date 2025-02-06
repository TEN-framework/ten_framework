//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import * as React from "react";
import { toast } from "sonner";
import { FolderOpenIcon, CogIcon, PlayIcon } from "lucide-react";
import { useTranslation } from "react-i18next";

import Popup from "@/components/Popup/Popup";
import {
  NavigationMenuContent,
  NavigationMenuItem,
  NavigationMenuLink,
  NavigationMenuTrigger,
} from "@/components/ui/NavigationMenu";
import { Button } from "@/components/ui/Button";
import { cn } from "@/lib/utils";
import { ThreeColumnFileManager } from "@/components/FileManager/AppFolder";
import {
  baseDirEntriesToIFMItems,
  fmItemsToFMArray,
  type IFMItem,
  EFMItemType,
} from "@/components/FileManager/utils";
import { useDirList, getBaseDir } from "@/api/services/fileSystem";
import { useWidgetStore } from "@/store/widget";
import { EWidgetCategory, EWidgetDisplayType } from "@/types/widgets";
import { Input } from "@/components/ui/Input";
import { TEN_DEFAULT_APP_RUN_SCRIPT } from "@/constants";

interface FileMenuProps {
  defaultBaseDir?: string;
  onSetBaseDir: (folderPath: string) => void;
}

export function FileMenu(props: FileMenuProps) {
  const { defaultBaseDir = "/", onSetBaseDir } = props;
  const { t } = useTranslation();

  const [isFolderPathModalOpen, setIsFolderPathModalOpen] =
    React.useState<boolean>(false);
  const [folderPath, setFolderPath] = React.useState<string>(defaultBaseDir);
  const [fmItems, setFmItems] = React.useState<IFMItem[][]>([]);

  const [isPreferencesModalOpen, setIsPreferencesModalOpen] =
    React.useState<boolean>(false);
  const [defaultRunScript, setDefaultRunScript] = React.useState<string>(
    TEN_DEFAULT_APP_RUN_SCRIPT
  );

  const { data, error, isLoading } = useDirList(folderPath);

  const { appendWidgetIfNotExists } = useWidgetStore();

  const handleAppStart = async () => {
    try {
      const baseDirData = await getBaseDir();
      const baseDir = baseDirData?.base_dir;
      if (!baseDir) {
        toast.error("Base directory is not set.");
        return;
      }

      const scriptName = defaultRunScript;

      appendWidgetIfNotExists({
        id: "app-start-" + Date.now(),
        category: EWidgetCategory.LogViewer,
        display_type: EWidgetDisplayType.Popup,
        metadata: {
          wsUrl: "ws://localhost:49483/api/designer/v1/ws/app/start",
          baseDir,
          scriptName,
          supportStop: true,
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
        category: EWidgetCategory.TerminalViewer,
        display_type: EWidgetDisplayType.Popup,
        metadata: {
          wsUrl: "ws://localhost:49483/api/designer/v1/ws/app/install",
          baseDir,
          supportStop: true,
        },
      });
    } catch (err) {
      console.error(err);
      toast.error("Failed to get base directory.");
    }
  };

  const handleManualOk = async () => {
    if (!folderPath.trim()) {
      toast.error("The folder path cannot be empty.");
      return;
    }

    console.log("[file-menu] folderPath set to", folderPath);
    onSetBaseDir(folderPath.trim());
    setIsFolderPathModalOpen(false);
  };

  React.useEffect(() => {
    if (!data?.entries) {
      return;
    }
    const currentFmItems = baseDirEntriesToIFMItems(data.entries);
    const fmArray = fmItemsToFMArray(currentFmItems, fmItems);
    setFmItems(fmArray);
    // Suppress the warning about the dependency array.
    // <fmItems> should not be a dependency.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [data, folderPath]);

  React.useEffect(() => {
    if (error) {
      toast.error("Failed to load the folder.", {
        description: error?.message,
      });
    }
  }, [error]);

  return (
    <>
      <NavigationMenuItem>
        <NavigationMenuTrigger className="submenu-trigger">
          {t("header.menu.file")}
        </NavigationMenuTrigger>
        <NavigationMenuContent
          className={cn("flex flex-col items-center px-1 py-1.5 gap-1.5")}
        >
          <NavigationMenuLink asChild>
            <Button
              className="w-full justify-start"
              variant="ghost"
              onClick={() => setIsFolderPathModalOpen(true)}
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
              <PlayIcon className="w-4 h-4 me-2" />
              {t("header.menu.installAll")}
            </Button>
          </NavigationMenuLink>

          <NavigationMenuLink asChild>
            <Button
              className="w-full justify-start"
              variant="ghost"
              onClick={() => setIsPreferencesModalOpen(true)}
            >
              <CogIcon className="w-4 h-4 me-2" />
              {t("header.menu.preferences")}
            </Button>
          </NavigationMenuLink>
        </NavigationMenuContent>
      </NavigationMenuItem>
      {isFolderPathModalOpen && (
        <Popup
          title={t("header.menu.openAppFolder")}
          onClose={() => setIsFolderPathModalOpen(false)}
          resizable
          initialWidth={600}
          initialHeight={400}
          onCollapseToggle={() => {}}
        >
          <div className="flex flex-col gap-2 w-full h-full">
            <ThreeColumnFileManager
              data={fmItems}
              allowSelectTypes={[EFMItemType.FOLDER]}
              className="w-full h-[calc(100%-3rem)]"
              onSelect={(path) => setFolderPath(path)}
              selectedPath={folderPath}
              isLoading={isLoading}
            />
            <div className="flex justify-end h-fit gap-2">
              <Button
                variant="outline"
                onClick={() => setIsFolderPathModalOpen(false)}
              >
                {t("action.cancel")}
              </Button>
              <Button onClick={handleManualOk}>{t("action.ok")}</Button>
            </div>
          </div>
        </Popup>
      )}
      {isPreferencesModalOpen && (
        <Popup
          title={t("header.menu.preferences")}
          onClose={() => setIsPreferencesModalOpen(false)}
          resizable={false}
          initialWidth={400}
          initialHeight={200}
          onCollapseToggle={() => {}}
          preventFocusSteal={true}
        >
          <div className="flex flex-col gap-2 w-full h-full">
            <label htmlFor="defaultRunScript">
              {t("Default label for app run")}{" "}
            </label>
            <Input
              id="defaultRunScript"
              type="text"
              defaultValue={defaultRunScript}
            />
            <div className="flex justify-end gap-2 mt-auto">
              <Button
                variant="outline"
                onClick={() => setIsPreferencesModalOpen(false)}
              >
                {t("action.cancel")}
              </Button>
              <Button
                onClick={() => {
                  const inputElement = document.getElementById(
                    "defaultRunScript"
                  ) as HTMLInputElement;
                  setDefaultRunScript(inputElement?.value || defaultRunScript);
                  setIsPreferencesModalOpen(false);
                }}
              >
                {t("action.ok")}
              </Button>
            </div>
          </div>
        </Popup>
      )}
    </>
  );
}
