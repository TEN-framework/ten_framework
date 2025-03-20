//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import * as React from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import {
  FolderMinusIcon,
  FolderPlusIcon,
  FolderSyncIcon,
  HardDriveDownloadIcon,
} from "lucide-react";

import {
  Table,
  TableBody,
  TableCaption,
  TableCell,
  TableFooter,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/Table";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { Button } from "@/components/ui/Button";
import { cn } from "@/lib/utils";
import { useApps } from "@/api/services/apps";
import { SpinnerLoading } from "@/components/Status/Loading";
import { useWidgetStore } from "@/store/widget";
import {
  EDefaultWidgetType,
  EWidgetDisplayType,
  EWidgetCategory,
} from "@/types/widgets";
import { APP_FOLDER_POPUP_ID } from "@/constants/widgets";
import { postReloadApps, postUnloadApps } from "@/api/services/apps";

export const AppsManagerWidget = (props: { className?: string }) => {
  const [isUnloading, setIsUnloading] = React.useState<boolean>(false);
  const [isReloading, setIsReloading] = React.useState<boolean>(false);

  const { t } = useTranslation();
  const { data: loadedApps, isLoading, error, mutate } = useApps();
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

  const handleUnloadApp = async (baseDir: string) => {
    try {
      setIsUnloading(true);
      await postUnloadApps(baseDir);
      toast.success(t("header.menuApp.unloadAppSuccess"));
    } catch (error) {
      console.error(error);
      toast.error(
        t("header.menuApp.unloadAppFailed", {
          description:
            error instanceof Error ? error.message : t("popup.apps.error"),
        })
      );
    } finally {
      mutate();
      setIsUnloading(false);
    }
  };

  const handleReloadApp = async (baseDir?: string) => {
    try {
      setIsReloading(true);
      await postReloadApps(baseDir);
      if (baseDir) {
        toast.success(t("header.menuApp.reloadAppSuccess"));
      } else {
        toast.success(t("header.menuApp.reloadAllAppsSuccess"));
      }
    } catch (error) {
      console.error(error);
      if (baseDir) {
        toast.error(
          t("header.menuApp.reloadAppFailed", {
            description:
              error instanceof Error ? error.message : t("popup.apps.error"),
          })
        );
      } else {
        toast.error(
          t("header.menuApp.reloadAllAppsFailed", {
            description:
              error instanceof Error ? error.message : t("popup.apps.error"),
          })
        );
      }
    } finally {
      mutate();
      setIsReloading(false);
    }
  };

  const handleAppInstall = async () => {
    // try {
    //   setIsInstalling(true);
    //   await postInstallApps();
    // } catch (error) {
    //   console.error(error);
    //   toast.error(
    //     t("header.menuApp.installAllFailed", {
    //       description:
    //         error instanceof Error ? error.message : t("popup.apps.error"),
    //     })
    //   );
    // } finally {
    //   setIsInstalling(false);
    // }
  };

  const isLoadingMemo = React.useMemo(() => {
    return isUnloading || isReloading || isLoading;
  }, [isUnloading, isReloading, isLoading]);

  React.useEffect(() => {
    if (error) {
      toast.error(t("popup.apps.error"));
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [error]);

  return (
    <div className={cn("flex flex-col gap-2 w-full h-full", props.className)}>
      <Table>
        <TableCaption>{t("popup.apps.tableCaption")}</TableCaption>
        <TableHeader>
          <TableRow>
            <TableHead className="w-14">{t("dataTable.no")}</TableHead>
            <TableHead>{t("dataTable.name")}</TableHead>
            <TableHead className="text-right">
              {t("dataTable.actions")}
            </TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {isLoading ? (
            <TableRow>
              <TableCell colSpan={3}>
                <SpinnerLoading />
              </TableCell>
            </TableRow>
          ) : (
            loadedApps?.base_dirs?.map((baseDir, index) => (
              <TableRow key={baseDir}>
                <TableCell className="font-medium">{index + 1}</TableCell>
                <TableCell>
                  <span className="text-xs rounded-md p-1 bg-muted px-2">
                    {baseDir}
                  </span>
                </TableCell>
                <TableCell className="text-right space-x-2">
                  <TooltipProvider>
                    <Tooltip>
                      <TooltipTrigger asChild>
                        <Button
                          variant="outline"
                          size="icon"
                          disabled={isLoadingMemo}
                          onClick={() => handleUnloadApp(baseDir)}
                        >
                          <FolderMinusIcon className="w-4 h-4" />
                          <span className="sr-only">
                            {t("header.menuApp.unloadApp")}
                          </span>
                        </Button>
                      </TooltipTrigger>
                      <TooltipContent>
                        <p>{t("header.menuApp.unloadApp")}</p>
                      </TooltipContent>
                    </Tooltip>
                  </TooltipProvider>

                  <TooltipProvider>
                    <Tooltip>
                      <TooltipTrigger asChild>
                        <Button
                          variant="outline"
                          size="icon"
                          disabled={isLoadingMemo}
                          onClick={() => handleReloadApp(baseDir)}
                        >
                          <FolderSyncIcon className="w-4 h-4" />
                          <span className="sr-only">
                            {t("header.menuApp.reloadApp")}
                          </span>
                        </Button>
                      </TooltipTrigger>
                      <TooltipContent>
                        <p>{t("header.menuApp.reloadApp")}</p>
                      </TooltipContent>
                    </Tooltip>
                  </TooltipProvider>

                  <TooltipProvider>
                    <Tooltip>
                      <TooltipTrigger asChild>
                        <Button
                          variant="outline"
                          size="icon"
                          disabled={isLoadingMemo}
                          onClick={handleAppInstall}
                        >
                          <HardDriveDownloadIcon className="w-4 h-4" />
                          <span className="sr-only">
                            {t("header.menuApp.installAll")}
                          </span>
                        </Button>
                      </TooltipTrigger>
                      <TooltipContent>
                        <p>{t("header.menuApp.installAll")}</p>
                      </TooltipContent>
                    </Tooltip>
                  </TooltipProvider>
                </TableCell>
              </TableRow>
            ))
          )}
        </TableBody>
        <TableFooter className="bg-transparent">
          <TableRow>
            <TableCell colSpan={3} className="text-right space-x-2">
              <Button
                variant="outline"
                size="sm"
                onClick={openAppFolderPopup}
                disabled={isLoadingMemo}
              >
                <FolderPlusIcon className="w-4 h-4" />
                {t("header.menuApp.loadApp")}
              </Button>
              <Button
                variant="outline"
                size="sm"
                disabled={isLoadingMemo}
                onClick={() => handleReloadApp()}
              >
                <FolderSyncIcon className="w-4 h-4" />
                <span>{t("header.menuApp.reloadAllApps")}</span>
              </Button>
            </TableCell>
          </TableRow>
        </TableFooter>
      </Table>
    </div>
  );
};
