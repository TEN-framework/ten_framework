//
// Copyright © 2024 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import {
  useEffect,
  useState,
  useCallback,
  forwardRef,
  useImperativeHandle,
  MouseEvent,
} from "react";
import {
  ReactFlow,
  addEdge,
  MiniMap,
  Controls,
  Edge,
  Node,
  applyNodeChanges,
  applyEdgeChanges,
  Connection,
  MarkerType,
  BuiltInNode,
} from "@xyflow/react";
import dagre from "dagre";
import CustomNode, { CustomNodeType } from "./CustomNode";

interface ApiResponse<T> {
  status: string;
  data: T;
  meta?: any;
}

interface BackendNode {
  addon: string;
  name: string;
  extension_group: string;
  app: string;
  property: any;
  api?: any;
}

interface BackendConnection {
  app: string;
  extension_group: string;
  extension: string;
  cmd?: {
    name: string;
    dest: {
      app: string;
      extension_group: string;
      extension: string;
    }[];
  }[];
}

const nodeWidth = 172;
const nodeHeight = 36;

const getLayoutedElements = (
  nodes: MyNodeType[],
  edges: Edge[],
  direction = "TB"
) => {
  const dagreGraph = new dagre.graphlib.Graph();
  dagreGraph.setDefaultEdgeLabel(() => ({}));
  dagreGraph.setGraph({ rankdir: direction });

  nodes.forEach((node) => {
    dagreGraph.setNode(node.id, { width: nodeWidth, height: nodeHeight });
  });

  edges.forEach((edge) => {
    dagreGraph.setEdge(edge.source, edge.target);
  });

  dagre.layout(dagreGraph);

  const layoutedNodes = nodes.map((node) => {
    const nodeWithPosition = dagreGraph.node(node.id);
    return {
      ...node,
      position: {
        x: nodeWithPosition.x - nodeWidth / 2,
        y: nodeWithPosition.y - nodeHeight / 2,
      },
    };
  });

  return { nodes: layoutedNodes, edges };
};

export interface FlowCanvasRef {
  performAutoLayout: () => void;
}

export type MyNodeType = BuiltInNode | CustomNodeType;

const FlowCanvas = forwardRef<FlowCanvasRef, {}>((props, ref) => {
  const [nodes, setNodes] = useState<MyNodeType[]>([]);
  const [edges, setEdges] = useState<Edge[]>([]);
  const [error, setError] = useState<string>("");

  const [contextMenu, setContextMenu] = useState<{
    visible: boolean;
    x: number;
    y: number;
    type?: "node" | "edge";
    data?: any;
  }>({ visible: false, x: 0, y: 0 });

  // Close the context menu.
  const closeContextMenu = useCallback(() => {
    setContextMenu({ visible: false, x: 0, y: 0 });
  }, []);

  // Click on the blank space to close the context menu.
  useEffect(() => {
    const handleClick = () => {
      closeContextMenu();
    };
    window.addEventListener("click", handleClick);
    return () => window.removeEventListener("click", handleClick);
  }, [closeContextMenu]);

  // Export performAutoLayout function.
  const performAutoLayout = useCallback(() => {
    const { nodes: layoutedNodes, edges: layoutedEdges } = getLayoutedElements(
      nodes,
      edges
    );
    setNodes(layoutedNodes);
    setEdges(layoutedEdges);
  }, [nodes, edges]);

  useImperativeHandle(ref, () => ({
    performAutoLayout,
  }));

  useEffect(() => {
    const fetchData = async () => {
      try {
        const nodesRes = await fetch(
          `/api/dev-server/v1/graphs/http_service/nodes`
        );
        if (!nodesRes.ok) {
          throw new Error(`Failed to fetch nodes: ${nodesRes.status}`);
        }
        const nodesPayload: ApiResponse<BackendNode[]> = await nodesRes.json();

        const connectionsRes = await fetch(
          `/api/dev-server/v1/graphs/http_service/connections`
        );
        if (!connectionsRes.ok) {
          throw new Error(
            `Failed to fetch connections: ${connectionsRes.status}`
          );
        }
        const connectionsPayload: ApiResponse<BackendConnection[]> =
          await connectionsRes.json();

        // Create initial node.
        let initialNodes: MyNodeType[] = nodesPayload.data.map((n, index) => ({
          id: n.name,
          position: { x: index * 200, y: 100 },
          type: "customNode",
          data: {
            label: `${n.name}`,
            sourceCmds: [],
            targetCmds: [],
          },
        }));

        // Analyze edges, collect cmds for all nodes, so that we can generate
        // corresponding handles later.
        let initialEdges: Edge[] = [];
        const nodeSourceCmdMap: Record<string, Set<string>> = {};
        const nodeTargetCmdMap: Record<string, Set<string>> = {};

        connectionsPayload.data.forEach((c) => {
          const sourceNodeId = c.extension;
          if (c.cmd) {
            c.cmd.forEach((cmdItem) => {
              cmdItem.dest.forEach((d) => {
                const targetNodeId = d.extension;
                const edgeId = `edge-${sourceNodeId}-${cmdItem.name}-${targetNodeId}`;
                const cmdName = cmdItem.name;

                // Record the cmd name of the source node.
                if (!nodeSourceCmdMap[sourceNodeId]) {
                  nodeSourceCmdMap[sourceNodeId] = new Set();
                }
                nodeSourceCmdMap[sourceNodeId].add(cmdName);

                // Record the cmd name of the target node.
                if (!nodeTargetCmdMap[targetNodeId]) {
                  nodeTargetCmdMap[targetNodeId] = new Set();
                }
                nodeTargetCmdMap[targetNodeId].add(cmdName);

                initialEdges.push({
                  id: edgeId,
                  source: sourceNodeId,
                  target: targetNodeId,
                  type: "default",
                  label: cmdName,
                  sourceHandle: `source-${cmdName}`,
                  targetHandle: `target-${cmdName}`,
                  markerEnd: {
                    type: MarkerType.ArrowClosed,
                  },
                });
              });
            });
          }
        });

        // Write cmds back to Nodes's data to let CustomNode dynamically
        // generate handles.
        initialNodes = initialNodes.map((node) => {
          const sourceCmds = nodeSourceCmdMap[node.id]
            ? Array.from(nodeSourceCmdMap[node.id])
            : [];
          const targetCmds = nodeTargetCmdMap[node.id]
            ? Array.from(nodeTargetCmdMap[node.id])
            : [];
          return {
            ...node,
            type: "customNode",
            data: {
              ...node.data,
              label: node.data.label || `${node.id}`,
              sourceCmds,
              targetCmds,
            },
          };
        });

        const { nodes: layoutedNodes, edges: layoutedEdges } =
          getLayoutedElements(initialNodes, initialEdges);

        setNodes(layoutedNodes);
        setEdges(layoutedEdges);
      } catch (err: any) {
        console.error(err);
        setError("Failed to fetch workflow data.");
      }
    };
    fetchData();
  }, []);

  const onConnect = useCallback(
    (params: Connection | Edge) => {
      setEdges((eds) => addEdge(params, eds));
    },
    [setEdges]
  );

  // Right-click Node.
  const onNodeContextMenu = useCallback((event: MouseEvent, node: Node) => {
    event.preventDefault();
    setContextMenu({
      visible: true,
      x: event.clientX,
      y: event.clientY,
      type: "node",
      data: node,
    });
  }, []);

  // Right-click Edge.
  const onEdgeContextMenu = useCallback((event: MouseEvent, edge: Edge) => {
    event.preventDefault();
    setContextMenu({
      visible: true,
      x: event.clientX,
      y: event.clientY,
      type: "edge",
      data: edge,
    });
  }, []);

  return (
    <div
      className="flow-container"
      style={{ width: "100%", height: "calc(100vh - 40px)" }}
    >
      <ReactFlow
        nodes={nodes}
        edges={edges}
        nodeTypes={{
          customNode: CustomNode,
        }}
        onNodesChange={(changes) =>
          setNodes((nds) => applyNodeChanges(changes, nds))
        }
        onEdgesChange={(changes) =>
          setEdges((eds) => applyEdgeChanges(changes, eds))
        }
        onConnect={(p) => onConnect(p)}
        fitView
        nodesDraggable={true}
        edgesFocusable={true}
        style={{ width: "100%", height: "100%" }}
        onNodeContextMenu={onNodeContextMenu}
        onEdgeContextMenu={onEdgeContextMenu}
      >
        <Controls />
        <MiniMap />
      </ReactFlow>
      {error && <div style={{ color: "red" }}>{error}</div>}

      {contextMenu.visible && (
        <div
          className="context-menu"
          style={{
            position: "fixed",
            left: contextMenu.x,
            top: contextMenu.y,
            backgroundColor: "#fff",
            border: "1px solid #ccc",
            padding: "5px",
            zIndex: 9999,
          }}
        >
          <div>Fake Menu Item</div>
        </div>
      )}
    </div>
  );
});

export default FlowCanvas;
