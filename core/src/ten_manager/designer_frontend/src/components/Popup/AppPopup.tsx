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

import Popup from "@/components/Popup/Popup";
import { Label } from "@/components/ui/Label";
import { Button } from "@/components/ui/Button";
import { Input } from "@/components/ui/Input";
import { useWidgetStore, useAppStore, useFlowStore } from "@/store";
import {
  baseDirEntriesToIFMItems,
  EFMItemType,
  fmItemsToFMArray,
} from "@/components/FileManager/utils";
import { ThreeColumnFileManager } from "@/components/FileManager/AppFolder";
import { useRetrieveDirList } from "@/api/services/fileSystem";
import { postBaseDir } from "@/api/services/apps";
import { SpinnerLoading } from "@/components/Status/Loading";
import {
  APP_FOLDER_POPUP_ID,
  APP_PREFERENCES_POPUP_ID,
  APPS_MANAGER_POPUP_ID,
} from "@/constants/widgets";
import { AppsManagerWidget } from "@/components/Widget/AppsWidget";

export const AppFolderPopup = () => {
  const [isSaving, setIsSaving] = React.useState<boolean>(false);

  const { t } = useTranslation();

  const { removeWidget } = useWidgetStore();
  const { setNodesAndEdges } = useFlowStore();
  const { folderPath, setFolderPath, fmItems, setFmItems } = useAppStore();

  const { data, error, isLoading } = useRetrieveDirList(folderPath);

  const handleSetBaseDir = async (folderPath: string) => {
    try {
      await postBaseDir(folderPath.trim());
      setNodesAndEdges([], []); // Clear the contents of the FlowCanvas.
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
      title={t("header.menu.openAppFolder")}
      onClose={() => removeWidget(APP_FOLDER_POPUP_ID)}
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

/** @deprecated */
export const PreferencesPopup = () => {
  const { t } = useTranslation();

  const { removeWidget } = useWidgetStore();
  const { runScript, setRunScript } = useAppStore();

  const [inputRunScript, setInputRunScript] = React.useState<string>(runScript);

  return (
    <Popup
      id={APP_PREFERENCES_POPUP_ID}
      title={t("header.menu.preferences")}
      onClose={() => removeWidget(APP_PREFERENCES_POPUP_ID)}
      resizable={false}
      initialWidth={400}
      initialHeight={200}
      onCollapseToggle={() => {}}
      preventFocusSteal={true}
    >
      <div className="flex flex-col gap-2 w-full h-full">
        <label htmlFor="defaultRunScript">
          {t("popup.default.defaultLabelForAppRun")}
        </label>
        <Input
          id="defaultRunScript"
          type="text"
          defaultValue={runScript}
          value={inputRunScript}
          onChange={(e) => setInputRunScript(e.target.value)}
        />
        <div className="flex justify-end gap-2 mt-auto">
          <Button
            variant="outline"
            onClick={() => removeWidget(APP_PREFERENCES_POPUP_ID)}
          >
            {t("action.cancel")}
          </Button>
          <Button
            disabled={!inputRunScript?.trim()}
            onClick={() => {
              setRunScript(inputRunScript || runScript);
              removeWidget(APP_PREFERENCES_POPUP_ID);
            }}
          >
            {t("action.ok")}
          </Button>
        </div>
      </div>
    </Popup>
  );
};

export const LoadedAppsPopup = () => {
  const { t } = useTranslation();

  const { removeWidget } = useWidgetStore();

  return (
    <Popup
      id={APPS_MANAGER_POPUP_ID}
      title={t("popup.apps.manager")}
      onClose={() => removeWidget(APPS_MANAGER_POPUP_ID)}
      resizable
      initialWidth={600}
      initialHeight={400}
      onCollapseToggle={() => {}}
    >
      <AppsManagerWidget />
    </Popup>
  );
};

export const AppRunPopup = (props: {
  id: string;
  data: { base_dir?: string };
}) => {
  const { id, data = {} } = props;
  const { base_dir: baseDir } = data;

  const { t } = useTranslation();

  const { removeWidget } = useWidgetStore();
  const { runScript, setRunScript } = useAppStore();

  const [inputRunScript, setInputRunScript] = React.useState<string>(runScript);

  const handleRun = () => {
    setRunScript(inputRunScript || runScript);
    removeWidget(id);
  };

  if (!baseDir) {
    return null;
  }

  return (
    <Popup
      id={id}
      title={t("popup.apps.run")}
      onClose={() => removeWidget(id)}
      resizable
      initialWidth={600}
      initialHeight={400}
      onCollapseToggle={() => {}}
      preventFocusSteal
    >
      <div className="flex flex-col gap-2 w-full h-full">
        <div className="flex flex-col gap-2">
          <Label htmlFor="runapp_base_dir">{t("popup.apps.baseDir")}</Label>
          <Input
            id="runapp_base_dir"
            type="text"
            defaultValue={baseDir}
            value={baseDir}
            disabled
          />
        </div>
        <div className="flex flex-col gap-2">
          <Label htmlFor="runapp_script">{t("popup.apps.runScript")}</Label>
          <Input
            id="runapp_script"
            type="text"
            defaultValue={runScript}
            value={inputRunScript}
            onChange={(e) => setInputRunScript(e.target.value)}
          />
        </div>
        <div className="flex justify-end gap-2 mt-auto">
          <Button variant="outline" onClick={() => removeWidget(id)}>
            {t("action.cancel")}
          </Button>
          <Button disabled={!inputRunScript?.trim()} onClick={handleRun}>
            <PlayIcon className="size-4" />
            {t("action.run")}
          </Button>
        </div>
      </div>
    </Popup>
  );
};
