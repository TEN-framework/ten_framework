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

import { PopupBase } from "@/components/Popup/Base";
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
import { AppFileManager } from "@/components/FileManager/AppFolder";
import { postLoadDir, useApps } from "@/api/services/apps";
import {
  APP_FOLDER_POPUP_ID,
  APPS_MANAGER_POPUP_ID,
  APP_CREATE_POPUP_ID,
  CONTAINER_DEFAULT_ID,
  GROUP_LOG_VIEWER_ID,
} from "@/constants/widgets";
import { TEN_DEFAULT_BACKEND_WS_ENDPOINT } from "@/constants";
import {
  AppsManagerWidget,
  AppTemplateWidget,
} from "@/components/Widget/AppsWidget";
import { TEN_PATH_WS_EXEC } from "@/constants";
import {
  ELogViewerScriptType,
  EWidgetDisplayType,
  EWidgetCategory,
  IWidget,
  IDefaultWidgetData,
  IDefaultWidget,
} from "@/types/widgets";
import { LogViewerPopupTitle } from "../LogViewer";

export const AppFolderPopupTitle = () => {
  const { t } = useTranslation();
  return t("popup.default.openAppFolder");
};

export const AppFolderPopupContent = (props: { widget: IWidget }) => {
  const { widget } = props;
  const [isSaving, setIsSaving] = React.useState<boolean>(false);

  const { t } = useTranslation();

  const { removeWidget } = useWidgetStore();
  const { setNodesAndEdges } = useFlowStore();
  const { folderPath, updateCurrentWorkspace } = useAppStore();

  const { mutate: mutateApps } = useApps();

  const handleSetBaseDir = async (folderPath: string) => {
    try {
      const { app_uri = null } = await postLoadDir(folderPath.trim());
      setNodesAndEdges([], []); // Clear the contents of the FlowCanvas.
      updateCurrentWorkspace({
        baseDir: folderPath.trim(),
        graphName: null,
        appUri: app_uri,
      });
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
      removeWidget(widget.widget_id);
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

  return (
    <AppFileManager
      isSaveLoading={isSaving}
      onSave={handleSave}
      onCancel={() => removeWidget(widget.widget_id)}
    />
  );
};

/** @deprecated */
export const AppFolderPopup = () => {
  const [isSaving, setIsSaving] = React.useState<boolean>(false);

  const { t } = useTranslation();

  const { removeWidget } = useWidgetStore();
  const { setNodesAndEdges } = useFlowStore();
  const { folderPath, updateCurrentWorkspace } = useAppStore();

  const { mutate: mutateApps } = useApps();

  const handleSetBaseDir = async (folderPath: string) => {
    try {
      const { app_uri = null } = await postLoadDir(folderPath.trim());
      setNodesAndEdges([], []); // Clear the contents of the FlowCanvas.
      updateCurrentWorkspace({
        baseDir: folderPath.trim(),
        graphName: null,
        appUri: app_uri,
      });
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

  return (
    <PopupBase
      id={APP_FOLDER_POPUP_ID}
      title={t("header.menuApp.openAppFolder")}
      resizable
      width={600}
      height={400}
    >
      <AppFileManager
        isSaveLoading={isSaving}
        onSave={handleSave}
        onCancel={() => removeWidget(APP_FOLDER_POPUP_ID)}
      />
    </PopupBase>
  );
};

export const LoadedAppsPopupTitle = () => {
  const { t } = useTranslation();
  return t("popup.apps.manager");
};

// eslint-disable-next-line @typescript-eslint/no-unused-vars
export const LoadedAppsPopupContent = (_props: { widget: IWidget }) => {
  return <AppsManagerWidget />;
};

/** @deprecated */
export const LoadedAppsPopup = () => {
  const { t } = useTranslation();

  return (
    <PopupBase
      id={APPS_MANAGER_POPUP_ID}
      title={t("popup.apps.manager")}
      resizable
    >
      <AppsManagerWidget />
    </PopupBase>
  );
};

export const AppRunPopupTitle = () => {
  const { t } = useTranslation();
  return t("popup.apps.run");
};

export const AppRunPopupContent = (props: { widget: IDefaultWidget }) => {
  const { widget } = props;
  const { base_dir: baseDir, scripts } = widget.metadata as IDefaultWidgetData;

  const { t } = useTranslation();

  const { removeWidget, appendWidgetIfNotExists } = useWidgetStore();

  const [selectedScript, setSelectedScript] = React.useState<
    string | undefined
  >(scripts?.[0] || undefined);

  const handleRun = () => {
    removeWidget(widget.widget_id);

    appendWidgetIfNotExists({
      container_id: CONTAINER_DEFAULT_ID,
      group_id: GROUP_LOG_VIEWER_ID,
      widget_id: "app-start-" + Date.now(),

      category: EWidgetCategory.LogViewer,
      display_type: EWidgetDisplayType.Popup,

      title: <LogViewerPopupTitle />,
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
        <Button
          variant="outline"
          onClick={() => removeWidget(widget.widget_id)}
        >
          {t("action.cancel")}
        </Button>
        <Button disabled={!selectedScript?.trim()} onClick={handleRun}>
          <PlayIcon className="size-4" />
          {t("action.run")}
        </Button>
      </div>
    </div>
  );
};

/** @deprecated */
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
    <PopupBase id={id} title={t("popup.apps.run")} width={600} maxWidth={600}>
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
    </PopupBase>
  );
};

export const AppCreatePopupTitle = () => {
  const { t } = useTranslation();
  return t("popup.apps.create");
};

export const AppCreatePopupContent = (props: { widget: IWidget }) => {
  const { widget } = props;

  const { removeWidget } = useWidgetStore();

  const handleCreated = () => {
    removeWidget(widget.widget_id);
  };

  return (
    <AppTemplateWidget onCreated={handleCreated} className="w-full max-w-sm" />
  );
};

/** @deprecated */
export const AppCreatePopup = (props: { id?: string }) => {
  const { id } = props;

  const { t } = useTranslation();
  const { removeWidget } = useWidgetStore();

  const handleCreated = () => {
    removeWidget(id || APP_CREATE_POPUP_ID);
  };

  return (
    <PopupBase
      id={APP_CREATE_POPUP_ID}
      title={t("popup.apps.create")}
      width={384}
    >
      <AppTemplateWidget
        onCreated={handleCreated}
        className="w-full max-w-sm"
      />
    </PopupBase>
  );
};
