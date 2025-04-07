//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import {
  useEffect,
  useState,
  useCallback,
  forwardRef,
  MouseEvent,
  useContext,
} from "react";
import {
  ReactFlow,
  MiniMap,
  Controls,
  Connection,
  type NodeChange,
  type EdgeChange,
} from "@xyflow/react";

import CustomNode from "@/flow/CustomNode";
import CustomEdge from "@/flow/CustomEdge";
import NodeContextMenu from "@/flow/ContextMenu/NodeContextMenu";
import EdgeContextMenu from "@/flow/ContextMenu/EdgeContextMenu";
import { TerminalData } from "@/components/Popup/TerminalPopup";
import { ThemeProviderContext } from "@/components/theme-context";
import { cn } from "@/lib/utils";
import { useWidgetStore, useAppStore } from "@/store";
import {
  EWidgetDisplayType,
  EWidgetCategory,
  type EditorData,
  ELogViewerScriptType,
} from "@/types/widgets";
import { EConnectionType } from "@/types/graphs";
import { ECustomEventName } from "@/utils/popup";

import type { TCustomEdge, TCustomNode } from "@/types/flow";

// Import react-flow style.
import "@xyflow/react/dist/style.css";
import "@/flow/reactflow.css";

export interface FlowCanvasRef {
  performAutoLayout: () => void;
}

interface FlowCanvasProps {
  nodes: TCustomNode[];
  edges: TCustomEdge[];
  onNodesChange: (changes: NodeChange<TCustomNode>[]) => void;
  onEdgesChange: (changes: EdgeChange<TCustomEdge>[]) => void;
  onConnect: (connection: Connection) => void;
  className?: string;
}

const FlowCanvas = forwardRef<FlowCanvasRef, FlowCanvasProps>(
  ({ nodes, edges, onNodesChange, onEdgesChange, onConnect, className }) => {
    const { appendWidget, appendWidgetIfNotExists } = useWidgetStore();
    const { currentWorkspace } = useAppStore();

    const [contextMenu, setContextMenu] = useState<{
      visible: boolean;
      x: number;
      y: number;
      type?: "node" | "edge";
      edge?: TCustomEdge;
      node?: TCustomNode;
    }>({ visible: false, x: 0, y: 0 });

    const launchTerminal = (data: TerminalData) => {
      const newPopup = { id: `${data.title}-${Date.now()}`, data };
      appendWidget({
        id: newPopup.id,
        category: EWidgetCategory.Terminal,
        metadata: newPopup.data,
        display_type: EWidgetDisplayType.Popup,
      });
    };

    const launchEditor = (data: EditorData) => {
      appendWidgetIfNotExists({
        id: `${data.url}-${Date.now()}`,
        category: EWidgetCategory.Editor,
        metadata: data,
        display_type: EWidgetDisplayType.Popup,
      });
    };

    const launchConnPopup = (
      source: string,
      target?: string,
      metadata?: {
        filters?: {
          type?: EConnectionType;
          source?: boolean;
          target?: boolean;
        };
      }
    ) => {
      const id = `${source}-${target ?? ""}`;
      const filters = metadata?.filters;
      console.log("filters", filters);
      appendWidgetIfNotExists({
        id,
        category: EWidgetCategory.CustomConnection,
        metadata: { id, source, target, filters },
        display_type: EWidgetDisplayType.Popup,
      });
    };

    const launchLogViewer = () => {
      appendWidgetIfNotExists({
        id: `logViewer-${Date.now()}`,
        category: EWidgetCategory.LogViewer,
        metadata: {
          wsUrl: "",
          scriptType: ELogViewerScriptType.DEFAULT,
          script: {},
        },
        display_type: EWidgetDisplayType.Popup,
      });
    };

    const renderContextMenu = () => {
      if (contextMenu.type === "node" && contextMenu.node) {
        return (
          <NodeContextMenu
            visible={contextMenu.visible}
            x={contextMenu.x}
            y={contextMenu.y}
            node={contextMenu.node}
            baseDir={currentWorkspace?.baseDir}
            graphName={currentWorkspace?.graphName}
            onClose={closeContextMenu}
            onLaunchTerminal={launchTerminal}
            onLaunchEditor={launchEditor}
            onLaunchLogViewer={launchLogViewer}
          />
        );
      } else if (contextMenu.type === "edge" && contextMenu.edge) {
        return (
          <EdgeContextMenu
            visible={contextMenu.visible}
            x={contextMenu.x}
            y={contextMenu.y}
            edge={contextMenu.edge}
            onClose={closeContextMenu}
          />
        );
      }
      return null;
    };

    // Right click nodes.
    const clickNodeContextMenu = useCallback(
      (event: MouseEvent, node: TCustomNode) => {
        event.preventDefault();
        setContextMenu({
          visible: true,
          x: event.clientX,
          y: event.clientY,
          type: "node",
          node: node,
        });
      },
      []
    );

    // Right click Edges.
    const clickEdgeContextMenu = useCallback(
      (event: MouseEvent, edge: TCustomEdge) => {
        event.preventDefault();
        setContextMenu({
          visible: true,
          x: event.clientX,
          y: event.clientY,
          type: "edge",
          edge: edge,
        });
      },
      []
    );

    // Close context menu.
    const closeContextMenu = useCallback(() => {
      setContextMenu({ visible: false, x: 0, y: 0 });
    }, []);

    // Click empty space to close context menu.
    useEffect(() => {
      const handleClick = () => {
        closeContextMenu();
      };
      const handleCustomNodeAction = (event: CustomEvent) => {
        switch (event.detail.action) {
          case "connections":
            launchConnPopup(
              event.detail.source,
              event.detail.target,
              event.detail.metadata
            );
            break;
          default:
            break;
        }
      };
      window.addEventListener("click", handleClick);
      window.addEventListener(
        ECustomEventName.CustomNodeAction,
        handleCustomNodeAction as EventListener
      );
      return () => {
        window.removeEventListener("click", handleClick);
        window.removeEventListener(
          ECustomEventName.CustomNodeAction,
          handleCustomNodeAction as EventListener
        );
      };
      // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [closeContextMenu]);

    const { theme } = useContext(ThemeProviderContext);

    return (
      <div
        className={cn("flow-container w-full h-[calc(100vh-40px)]", className)}
      >
        <ReactFlow
          colorMode={theme}
          nodes={nodes}
          edges={edges}
          edgeTypes={{
            customEdge: CustomEdge,
          }}
          nodeTypes={{
            customNode: CustomNode,
          }}
          onNodesChange={onNodesChange}
          onEdgesChange={onEdgesChange}
          onConnect={(p) => onConnect(p)}
          fitView
          nodesDraggable={true}
          edgesFocusable={true}
          style={{ width: "100%", height: "100%" }}
          onNodeContextMenu={clickNodeContextMenu}
          onEdgeContextMenu={clickEdgeContextMenu}
          // onEdgeClick={(e, edge) => {
          //   console.log("clicked", e, edge);
          // }}
        >
          <Controls />
          <MiniMap zoomable pannable />
          <svg className="">
            <defs>
              <linearGradient id="edge-gradient">
                <stop offset="0%" stopColor="#ae53ba" />
                <stop offset="100%" stopColor="#2a8af6" />
              </linearGradient>

              <marker
                id="edge-circle"
                viewBox="-5 -5 10 10"
                refX="0"
                refY="0"
                markerUnits="strokeWidth"
                markerWidth="10"
                markerHeight="10"
                orient="auto"
              >
                <circle
                  stroke="#2a8af6"
                  strokeOpacity="0.75"
                  r="2"
                  cx="0"
                  cy="0"
                />
              </marker>
            </defs>
          </svg>
        </ReactFlow>

        {renderContextMenu()}
      </div>
    );
  }
);

export default FlowCanvas;
