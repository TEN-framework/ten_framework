//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import * as React from "react";
import { toast } from "sonner";
import { useTranslation } from "react-i18next";
import { PlayIcon } from "lucide-react";

import { Popup } from "@/components/Popup/Popup";
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/Select";
import { Label } from "@/components/ui/Label";
import { Button } from "@/components/ui/Button";
import { Input } from "@/components/ui/Input";
import { useWidgetStore, useAppStore, useFlowStore } from "@/store";
import {
  baseDirEntriesToIFMItems,
  EFMItemType,
  fmItemsToFMArray,
} from "@/components/FileManager/utils";
import { FileManager } from "@/components/FileManager/AppFolder";
import { useRetrieveDirList } from "@/api/services/fileSystem";
import { postBaseDir, useApps } from "@/api/services/apps";
import { SpinnerLoading } from "@/components/Status/Loading";
import {
  APP_FOLDER_POPUP_ID,
  APPS_MANAGER_POPUP_ID,
} from "@/constants/widgets";
import { TEN_DEFAULT_BACKEND_WS_ENDPOINT } from "@/constants";
import { AppsManagerWidget } from "@/components/Widget/AppsWidget";
import { TEN_PATH_WS_EXEC } from "@/constants";
import {
  ELogViewerScriptType,
  EWidgetDisplayType,
  EWidgetCategory,
} from "@/types/widgets";

export const AppFolderPopup = () => {
  const [isSaving, setIsSaving] = React.useState<boolean>(false);

  const { t } = useTranslation();

  const { removeWidget } = useWidgetStore();
  const { setNodesAndEdges } = useFlowStore();
  const { folderPath, setFolderPath, fmItems, setFmItems } = useAppStore();

  const { data, error, isLoading } = useRetrieveDirList(folderPath);
  const { mutate: mutateApps } = useApps();

  const handleSetBaseDir = async (folderPath: string) => {
    try {
      await postBaseDir(folderPath.trim());
      setNodesAndEdges([], []); // Clear the contents of the FlowCanvas.
      mutateApps();
      toast.success(t("header.menuApp.loadAppSuccess"));
    } catch (error: unknown) {
      if (error instanceof Error) {
        toast.error(t("popup.default.errorOpenAppFolder"), {
          description: error.message,
        });
      } else {
        toast.error(t("popup.default.errorUnknown"));
      }
      console.error(error);
    }
  };

  const handleSave = async () => {
    setIsSaving(true);
    try {
      if (!folderPath.trim()) {
        toast.error(t("popup.default.errorFolderPathEmpty"));
        throw new Error("errorFolderPathEmpty");
      }
      console.log("[file-menu] folderPath set to", folderPath);
      await handleSetBaseDir(folderPath.trim());
      removeWidget(APP_FOLDER_POPUP_ID);
    } catch (error: unknown) {
      if (error instanceof Error && error.message === "errorFolderPathEmpty") {
        toast.error(t("popup.default.errorFolderPathEmpty"));
      } else {
        toast.error(t("popup.default.errorUnknown"));
      }
      console.error(error);
    } finally {
      setIsSaving(false);
    }
  };

  React.useEffect(() => {
    if (error) {
      toast.error(t("popup.default.errorGetBaseDir"));
      console.error(error);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [error]);

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

  return (
    <Popup
      id={APP_FOLDER_POPUP_ID}
      title={t("header.menuApp.openAppFolder")}
      resizable
      width={600}
      height={400}
    >
      <div className="flex flex-col gap-2 w-full h-full">
        <FileManager
          data={fmItems}
          allowSelectTypes={[EFMItemType.FOLDER]}
          className="w-full h-[calc(100%-3rem)]"
          onSelect={(path) => setFolderPath(path)}
          selectedPath={folderPath}
          isLoading={isLoading}
          colWidth={200}
        />
        <div className="flex justify-end h-fit gap-2">
          <Button
            variant="outline"
            onClick={() => removeWidget(APP_FOLDER_POPUP_ID)}
            disabled={isSaving}
          >
            {t("action.cancel")}
          </Button>
          <Button onClick={handleSave} disabled={isSaving}>
            {isSaving ? (
              <>
                <SpinnerLoading className="w-4 h-4 mr-2" />
                {t("action.ok")}
              </>
            ) : (
              t("action.ok")
            )}
          </Button>
        </div>
      </div>
    </Popup>
  );
};

export const LoadedAppsPopup = () => {
  const { t } = useTranslation();

  return (
    <Popup id={APPS_MANAGER_POPUP_ID} title={t("popup.apps.manager")} resizable>
      <AppsManagerWidget />
    </Popup>
  );
};

export const AppRunPopup = (props: {
  id: string;
  data: { base_dir?: string; scripts?: string[] };
}) => {
  const { id, data = {} } = props;
  const { base_dir: baseDir, scripts } = data;

  const { t } = useTranslation();

  const { removeWidget, appendWidgetIfNotExists } = useWidgetStore();

  const [selectedScript, setSelectedScript] = React.useState<
    string | undefined
  >(scripts?.[0] || undefined);

  const handleRun = () => {
    removeWidget(id);

    appendWidgetIfNotExists({
      id: "app-start-" + Date.now(),
      category: EWidgetCategory.LogViewer,
      display_type: EWidgetDisplayType.Popup,

      metadata: {
        wsUrl: TEN_DEFAULT_BACKEND_WS_ENDPOINT + TEN_PATH_WS_EXEC,
        scriptType: ELogViewerScriptType.RUN_SCRIPT,
        script: {
          type: ELogViewerScriptType.RUN_SCRIPT,
          base_dir: baseDir,
          name: selectedScript,
        },
        onStop: () => {
          console.log("app-start-widget-closed", baseDir, selectedScript);
        },
      },
    });
  };

  if (!baseDir || !scripts || scripts.length === 0) {
    return null;
  }

  return (
    <Popup id={id} title={t("popup.apps.run")} width={600} maxWidth={600}>
      <div className="flex flex-col gap-2 w-full h-full">
        <div className="flex flex-col gap-2">
          <Label htmlFor="runapp_base_dir">{t("popup.apps.baseDir")}</Label>
          <Input id="runapp_base_dir" type="text" value={baseDir} disabled />
        </div>
        <div className="flex flex-col gap-2">
          <Label htmlFor="runapp_script">{t("popup.apps.runScript")}</Label>
          <Select value={selectedScript} onValueChange={setSelectedScript}>
            <SelectTrigger id="runapp_script">
              <SelectValue placeholder={t("popup.apps.selectScript")} />
            </SelectTrigger>
            <SelectContent>
              <SelectGroup>
                {scripts?.map((script) => (
                  <SelectItem key={script} value={script}>
                    {script}
                  </SelectItem>
                ))}
              </SelectGroup>
            </SelectContent>
          </Select>
        </div>
        <div className="flex justify-end gap-2 mt-auto">
          <Button variant="outline" onClick={() => removeWidget(id)}>
            {t("action.cancel")}
          </Button>
          <Button disabled={!selectedScript?.trim()} onClick={handleRun}>
            <PlayIcon className="size-4" />
            {t("action.run")}
          </Button>
        </div>
      </div>
    </Popup>
  );
};
