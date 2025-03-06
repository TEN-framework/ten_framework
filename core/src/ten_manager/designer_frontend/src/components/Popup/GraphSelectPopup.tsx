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
  enhanceNodesWithCommands,
  fetchAddonInfoForNodes,
  getLayoutedElements,
  processConnections,
  processNodes,
} from "@/flow/graph";
import Popup from "@/components/Popup/Popup";
import { Button } from "@/components/ui/Button";
import { SpinnerLoading } from "@/components/Status/Loading";
import {
  getGraphNodes,
  getGraphConnections,
  useGraphs,
} from "@/api/services/graphs";
import { useWidgetStore, useFlowStore } from "@/store";

import type { CustomNodeType } from "@/flow/CustomNode";

export const GRAPH_SELECT_POPUP_ID = "graph-select-popup";

export function GraphSelectPopup() {
  const { t } = useTranslation();
  const { graphs = [], error, isLoading } = useGraphs();
  const { removeWidget } = useWidgetStore();
  const { setNodesAndEdges } = useFlowStore();

  const handleSelectGraph = (graphName: string) => async () => {
    try {
      const backendNodes = await getGraphNodes(graphName);
      const backendConnections = await getGraphConnections(graphName);

      let initialNodes: CustomNodeType[] = processNodes(backendNodes);

      const {
        initialEdges,
        nodeSourceCmdMap,
        nodeSourceDataMap,
        nodeSourceAudioFrameMap,
        nodeSourceVideoFrameMap,
        nodeTargetCmdMap,
        nodeTargetDataMap,
        nodeTargetAudioFrameMap,
        nodeTargetVideoFrameMap,
      } = processConnections(backendConnections);

      // Write back the cmd information to nodes, so that CustomNode could
      // generate corresponding handles.
      initialNodes = enhanceNodesWithCommands(initialNodes, {
        nodeSourceCmdMap,
        nodeTargetCmdMap,
        nodeSourceDataMap,
        nodeTargetDataMap,
        nodeSourceAudioFrameMap,
        nodeSourceVideoFrameMap,
        nodeTargetAudioFrameMap,
        nodeTargetVideoFrameMap,
      });

      // Fetch additional addon information for each node.
      const nodesWithAddonInfo = await fetchAddonInfoForNodes(initialNodes);

      // Auto-layout the nodes and edges.
      const { nodes: layoutedNodes, edges: layoutedEdges } =
        getLayoutedElements(nodesWithAddonInfo, initialEdges);

      setNodesAndEdges(layoutedNodes, layoutedEdges);
    } catch (err: unknown) {
      console.error(err);
      toast.error("Failed to load graph.");
    } finally {
      removeWidget(GRAPH_SELECT_POPUP_ID);
    }
  };

  React.useEffect(() => {
    if (error instanceof Error) {
      toast.error(`Failed to fetch graphs: ${error.message}`);
    } else if (error) {
      toast.error("An unknown error occurred.");
    }
  }, [error]);

  return (
    <Popup
      id={GRAPH_SELECT_POPUP_ID}
      title={t("popup.selectGraph.title")}
      onClose={() => removeWidget(GRAPH_SELECT_POPUP_ID)}
      resizable={false}
      initialWidth={400}
      initialHeight={300}
      onCollapseToggle={() => {}}
    >
      {isLoading ? (
        <>
          <SpinnerLoading
            className="w-full h-full"
            svgProps={{ className: "size-10" }}
          />
        </>
      ) : (
        <ul className="flex flex-col gap-2 h-full w-full">
          {graphs.map((graph) => (
            <Button
              asChild
              key={graph.name}
              className="justify-start cursor-pointer"
              variant="ghost"
              onClick={handleSelectGraph(graph.name)}
            >
              <li className="w-full">
                <span className="text-sm">{graph.name}</span>
                {graph.auto_start ? (
                  <span className="text-xs">({t("action.autoStart")})</span>
                ) : null}
              </li>
            </Button>
          ))}
        </ul>
      )}
    </Popup>
  );
}
