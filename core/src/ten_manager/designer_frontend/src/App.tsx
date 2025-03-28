//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import React, { useState, useCallback, useRef } from "react";
import {
  addEdge,
  applyEdgeChanges,
  applyNodeChanges,
  EdgeChange,
  NodeChange,
} from "@xyflow/react";
import { toast } from "sonner";

import { ThemeProvider } from "@/components/ThemeProvider";
import AppBar from "@/components/AppBar";
import StatusBar from "@/components/StatusBar";
import FlowCanvas, { type FlowCanvasRef } from "@/flow/FlowCanvas";
import { CustomNodeType } from "@/flow/CustomNode";
import { CustomEdgeType } from "@/flow/CustomEdge";
import { getLayoutedElements } from "@/flow/graph";
import {
  ResizableHandle,
  ResizablePanel,
  ResizablePanelGroup,
} from "@/components/ui/Resizable";
import { GlobalDialogs } from "@/components/GlobalDialogs";
import Dock from "@/components/Dock";
import { useWidgetStore, useFlowStore, useAppStore } from "@/store";
import { EWidgetDisplayType } from "@/types/widgets";
import { GlobalPopups } from "@/components/Popup/GlobalPopups";
import { BackstageWidgets } from "@/components/Widget/BackstageWidgets";
import { cn } from "@/lib/utils";
import { usePreferences } from "@/api/services/common";
import { SpinnerLoading } from "@/components/Status/Loading";
import { PREFERENCES_SCHEMA_LOG } from "@/types/apps";

const App: React.FC = () => {
  const { nodes, setNodes, edges, setEdges, setNodesAndEdges } = useFlowStore();
  const [resizablePanelMode, setResizablePanelMode] = useState<
    "left" | "bottom" | "right"
  >("bottom");

  const {
    data: remotePreferences,
    isLoading: isLoadingPreferences,
    error: errorPreferences,
  } = usePreferences();

  const { widgets } = useWidgetStore();
  const { setPreferences } = useAppStore();

  const flowCanvasRef = useRef<FlowCanvasRef | null>(null);

  const dockWidgetsMemo = React.useMemo(
    () =>
      widgets.filter(
        (widget) => widget.display_type === EWidgetDisplayType.Dock
      ),
    [widgets]
  );

  const performAutoLayout = useCallback(() => {
    const { nodes: layoutedNodes, edges: layoutedEdges } = getLayoutedElements(
      nodes,
      edges
    );
    setNodesAndEdges(layoutedNodes, layoutedEdges);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [nodes, edges]);

  const handleNodesChange = useCallback(
    (changes: NodeChange<CustomNodeType>[]) => {
      const newNodes = applyNodeChanges(changes, nodes);
      setNodes(newNodes);
    },
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [nodes]
  );

  const handleEdgesChange = useCallback(
    (changes: EdgeChange<CustomEdgeType>[]) => {
      const newEdges = applyEdgeChanges(changes, edges);
      setEdges(newEdges);
    },
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [edges]
  );

  // init preferences
  React.useEffect(() => {
    if (
      remotePreferences &&
      remotePreferences?.preferences?.logviewer_line_size
    ) {
      const parsedValues = PREFERENCES_SCHEMA_LOG.safeParse(
        remotePreferences.preferences
      );
      if (!parsedValues.success) {
        throw new Error("Invalid values");
      }
      setPreferences(
        "logviewer_line_size",
        parsedValues.data.logviewer_line_size
      );
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [remotePreferences]);

  React.useEffect(() => {
    if (errorPreferences) {
      console.error(errorPreferences);
      toast.error(errorPreferences.message);
    }
  }, [errorPreferences]);

  if (isLoadingPreferences) {
    return (
      <div className="flex items-center justify-center h-screen w-full">
        <SpinnerLoading />
      </div>
    );
  }

  return (
    <ThemeProvider defaultTheme="system" storageKey="vite-ui-theme">
      <AppBar onAutoLayout={performAutoLayout} className="z-9997" />

      <ResizablePanelGroup
        key={`resizable-panel-group-${resizablePanelMode}`}
        direction={resizablePanelMode === "bottom" ? "vertical" : "horizontal"}
        className={cn("w-screen h-screen", "min-h-screen min-w-screen")}
      >
        {resizablePanelMode === "left" && dockWidgetsMemo.length > 0 && (
          <>
            <ResizablePanel defaultSize={40}>
              {/* Global dock widgets. */}
              <Dock
                position={resizablePanelMode}
                onPositionChange={
                  setResizablePanelMode as (position: string) => void
                }
              />
            </ResizablePanel>
            <ResizableHandle />
          </>
        )}
        <ResizablePanel defaultSize={dockWidgetsMemo.length > 0 ? 60 : 100}>
          <FlowCanvas
            ref={flowCanvasRef}
            nodes={nodes}
            edges={edges}
            onNodesChange={handleNodesChange}
            onEdgesChange={handleEdgesChange}
            onConnect={(connection) => {
              const newEdges = addEdge(connection, edges);
              setEdges(newEdges);
            }}
            className="w-full h-[calc(100dvh-60px)] mt-10"
          />
        </ResizablePanel>
        {resizablePanelMode !== "left" && dockWidgetsMemo.length > 0 && (
          <>
            <ResizableHandle />
            <ResizablePanel defaultSize={40}>
              {/* Global dock widgets. */}
              <Dock
                position={resizablePanelMode}
                onPositionChange={
                  setResizablePanelMode as (position: string) => void
                }
              />
            </ResizablePanel>
          </>
        )}
      </ResizablePanelGroup>

      {/* Global popups. */}
      <GlobalPopups />

      {/* Global dialogs. */}
      <GlobalDialogs />

      {/* [invisible] Global backstage widgets. */}
      <BackstageWidgets />

      <StatusBar className="z-9997" />
    </ThemeProvider>
  );
};

export default App;
