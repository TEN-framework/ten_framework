//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import * as React from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { CheckIcon } from "lucide-react";

import { PopupBase } from "@/components/Popup/Base";
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectLabel,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/Select";
import { Label } from "@/components/ui/Label";
import { Button } from "@/components/ui/Button";
import { SpinnerLoading } from "@/components/Status/Loading";
import { useGraphs } from "@/api/services/graphs";
import { useApps } from "@/api/services/apps";
import { useWidgetStore, useFlowStore, useAppStore } from "@/store";
import { GRAPH_SELECT_POPUP_ID } from "@/constants/widgets";
// eslint-disable-next-line max-len
import { resetNodesAndEdgesByGraphName } from "@/components/Widget/GraphsWidget";

export function GraphSelectPopup() {
  const { t } = useTranslation();

  const {
    data: loadedApps,
    isLoading: isLoadingApps,
    error: errorApps,
  } = useApps();
  const { removeWidget } = useWidgetStore();
  const { setNodesAndEdges } = useFlowStore();
  const { currentWorkspace, updateCurrentWorkspace } = useAppStore();

  const [selectedApp, setSelectedApp] = React.useState<string | null>(
    currentWorkspace.baseDir ?? loadedApps?.app_info?.[0]?.base_dir ?? null
  );
  const [selectedAppUri, setSelectedAppUri] = React.useState<string | null>(
    currentWorkspace.appUri ?? loadedApps?.app_info?.[0]?.app_uri ?? null
  );

  const { graphs = [], error, isLoading } = useGraphs(selectedApp);

  const [tmpSelectedGraph, setTmpSelectedGraph] = React.useState<string | null>(
    currentWorkspace.graphName ?? null
  );

  const handleSelectApp = (app?: string | null) => {
    setSelectedApp(app ?? null);
    setSelectedAppUri(
      loadedApps?.app_info?.find((appInfo) => appInfo.base_dir === app)
        ?.app_uri ?? null
    );

    setTmpSelectedGraph(null);
  };

  const handleSelectGraph =
    (graphName: string, baseDir: string | null, appUri: string | null) =>
    async () => {
      updateCurrentWorkspace({
        baseDir,
        graphName,
        appUri,
      });
      try {
        const { nodes: layoutedNodes, edges: layoutedEdges } =
          await resetNodesAndEdgesByGraphName(graphName, baseDir);

        setNodesAndEdges(layoutedNodes, layoutedEdges);
      } catch (err: unknown) {
        console.error(err);
        toast.error("Failed to load graph.");
      } finally {
        removeWidget(GRAPH_SELECT_POPUP_ID);
      }
    };

  const handleCancel = () => {
    removeWidget(GRAPH_SELECT_POPUP_ID);
    handleSelectApp(currentWorkspace.baseDir);
    setTmpSelectedGraph(currentWorkspace.graphName);
  };

  const handleSave = async () => {
    if (!tmpSelectedGraph) {
      updateCurrentWorkspace({
        baseDir: selectedApp,
        graphName: null,
      });
      setNodesAndEdges([], []);
      toast.success(t("popup.selectGraph.updateSuccess"), {
        description: (
          <>
            <p>{`${t("popup.selectGraph.app")}: ${selectedApp}`}</p>
          </>
        ),
      });
    } else {
      await handleSelectGraph(tmpSelectedGraph, selectedApp, selectedAppUri)();
      toast.success(t("popup.selectGraph.updateSuccess"), {
        description: (
          <>
            <p>{`${t("popup.selectGraph.app")}: ${selectedApp}`}</p>
            <p>{`${t("popup.selectGraph.graph")}: ${tmpSelectedGraph}`}</p>
          </>
        ),
      });
    }
    removeWidget(GRAPH_SELECT_POPUP_ID);
  };

  React.useEffect(() => {
    if (error instanceof Error) {
      toast.error(`Failed to fetch graphs: ${error.message}`);
    } else if (error) {
      toast.error("An unknown error occurred.");
    }
    if (errorApps instanceof Error) {
      toast.error(`Failed to fetch apps: ${errorApps.message}`);
    } else if (errorApps) {
      toast.error("An unknown error occurred.");
    }
  }, [error, errorApps]);

  return (
    <PopupBase
      id={GRAPH_SELECT_POPUP_ID}
      title={t("popup.selectGraph.title")}
      resizable={false}
      width={400}
    >
      <div className="flex flex-col gap-2 w-full h-full">
        <Label>{t("popup.selectGraph.app")}</Label>
        {isLoadingApps ? (
          <SpinnerLoading
            className="w-full h-full"
            svgProps={{ className: "size-10" }}
          />
        ) : (
          <Select
            onValueChange={(value) => handleSelectApp(value)}
            value={selectedApp ?? undefined}
          >
            <SelectTrigger className="w-full">
              <SelectValue
                placeholder={t("header.menuGraph.selectLoadedApp")}
              />
            </SelectTrigger>
            <SelectContent>
              <SelectGroup>
                <SelectLabel>
                  {t("header.menuGraph.selectLoadedApp")}
                </SelectLabel>
                {loadedApps?.app_info?.map((app) => (
                  <SelectItem key={app.base_dir} value={app.base_dir}>
                    {app.base_dir}
                  </SelectItem>
                ))}
              </SelectGroup>
            </SelectContent>
          </Select>
        )}
        <Label>{t("popup.selectGraph.graph")}</Label>
        {isLoading ? (
          <>
            <SpinnerLoading
              className="w-full h-full"
              svgProps={{ className: "size-10" }}
            />
          </>
        ) : (
          <ul className="flex flex-col gap-1 h-full w-full">
            <Button
              asChild
              key={"null"}
              className="justify-start cursor-pointer"
              variant="ghost"
              onClick={() => setTmpSelectedGraph(null)}
            >
              <li className="w-full">
                {tmpSelectedGraph === null ? (
                  <CheckIcon className="size-4" />
                ) : (
                  <div className="size-4" />
                )}
                <span className="text-sm">
                  {t("popup.selectGraph.unspecified")}
                </span>
              </li>
            </Button>
            {graphs?.map((graph) => (
              <Button
                asChild
                key={graph.name}
                className="justify-start cursor-pointer"
                variant="ghost"
                onClick={() => setTmpSelectedGraph(graph.name)}
              >
                <li className="w-full">
                  {tmpSelectedGraph === graph.name ? (
                    <CheckIcon className="size-4" />
                  ) : (
                    <div className="size-4" />
                  )}
                  <span className="text-sm">{graph.name}</span>
                  {graph.auto_start ? (
                    <span className="text-xs">({t("action.autoStart")})</span>
                  ) : null}
                  {selectedApp === currentWorkspace.baseDir &&
                  graph.name === currentWorkspace.graphName ? (
                    <span className="text-xs">({t("action.current")})</span>
                  ) : null}
                </li>
              </Button>
            ))}
          </ul>
        )}
        <div className="flex justify-end gap-2">
          <Button variant="outline" onClick={handleCancel}>
            {t("action.cancel")}
          </Button>
          <Button onClick={handleSave}>{t("action.save")}</Button>
        </div>
      </div>
    </PopupBase>
  );
}
