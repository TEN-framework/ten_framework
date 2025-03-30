//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import { MarkerType } from "@xyflow/react";
import dagre from "dagre";

import { CustomNodeType } from "@/flow/CustomNode";
import { CustomEdgeType } from "@/flow/CustomEdge";
import { retrieveAddons } from "@/api/services/addons";
import type { IExtensionAddon } from "@/types/apps";
import {
  type IBackendNode,
  type IBackendConnection,
  EConnectionType,
  type TConnectionMap,
} from "@/types/graphs";

const NODE_WIDTH = 172;
const NODE_HEIGHT = 48;

export const getLayoutedElements = (
  nodes: CustomNodeType[],
  edges: CustomEdgeType[]
) => {
  const dagreGraph = new dagre.graphlib.Graph();
  dagreGraph.setDefaultEdgeLabel(() => ({}));

  // Find all bidirectional pairs.
  const nodePairs = new Map<string, Set<string>>();
  edges.forEach((edge) => {
    const hasReverse = edges.some(
      (e) => e.source === edge.target && e.target === edge.source
    );
    if (hasReverse) {
      if (!nodePairs.has(edge.source)) {
        nodePairs.set(edge.source, new Set());
      }
      nodePairs.get(edge.source)?.add(edge.target);
    }
  });

  // Set graph to flow top to bottom-right.
  dagreGraph.setGraph({
    rankdir: "TB",
    nodesep: NODE_WIDTH * 2,
    ranksep: NODE_HEIGHT * 2,
  });

  // Add nodes to graph.
  nodes.forEach((node) => {
    dagreGraph.setNode(node.id, { width: NODE_WIDTH, height: NODE_HEIGHT });
  });

  // Process pairs in order of first appearance.
  const processedPairs = new Set<string>();
  let currentX = 0;
  const nodeXPositions = new Map<string, number>();

  edges.forEach((edge) => {
    dagreGraph.setEdge(edge.source, edge.target);

    // Check if this forms a pair and hasn't been processed.
    const pairKey = [edge.source, edge.target].sort().join("-");
    if (
      nodePairs.has(edge.source) &&
      nodePairs.get(edge.source)?.has(edge.target) &&
      !processedPairs.has(pairKey)
    ) {
      processedPairs.add(pairKey);
      nodeXPositions.set(edge.source, currentX);
      nodeXPositions.set(edge.target, currentX + NODE_WIDTH * 2);
      currentX += NODE_WIDTH * 4;
    }
  });

  dagre.layout(dagreGraph);

  const layoutedNodes = nodes.map((node) => {
    const nodeWithPosition = dagreGraph.node(node.id);
    const xPos = nodeXPositions.has(node.id)
      ? nodeXPositions.get(node.id)
      : nodeWithPosition.x;

    return {
      ...node,
      position: {
        x: (xPos ?? nodeWithPosition.x) - NODE_WIDTH / 2,
        y: nodeWithPosition.y - NODE_HEIGHT / 2,
      },
    };
  });

  const edgesWithNewHandles = edges.map((edge) => {
    const type = edge.data?.connectionType;
    if (type) {
      return {
        ...edge,
        sourceHandle: `source-${edge.source}-${type}`,
        targetHandle: `target-${edge.target}-${type}`,
      };
    }
    return edge;
  });

  return { nodes: layoutedNodes, edges: edgesWithNewHandles };
};

export const processNodes = (
  backendNodes: IBackendNode[]
): CustomNodeType[] => {
  return backendNodes.map((n, index) => ({
    id: n.name,
    position: { x: index * 200, y: 100 },
    type: "customNode",
    data: {
      name: `${n.name}`,
      addon: n.addon,
      sourceCmds: [],
      targetCmds: [],
      sourceData: [],
      targetData: [],
      sourceAudioFrame: [],
      targetAudioFrame: [],
      sourceVideoFrame: [],
      targetVideoFrame: [],
    },
  }));
};

export const processConnections = (
  backendConnections: IBackendConnection[]
): {
  initialEdges: CustomEdgeType[];
  nodeSourceCmdMap: TConnectionMap;
  nodeSourceDataMap: TConnectionMap;
  nodeSourceAudioFrameMap: TConnectionMap;
  nodeSourceVideoFrameMap: TConnectionMap;
  nodeTargetCmdMap: TConnectionMap;
  nodeTargetDataMap: TConnectionMap;
  nodeTargetAudioFrameMap: TConnectionMap;
  nodeTargetVideoFrameMap: TConnectionMap;
} => {
  const initialEdges: CustomEdgeType[] = [];
  const nodeSourceCmdMap: TConnectionMap = {};
  const nodeSourceDataMap: TConnectionMap = {};
  const nodeSourceAudioFrameMap: TConnectionMap = {};
  const nodeSourceVideoFrameMap: TConnectionMap = {};
  const nodeTargetCmdMap: TConnectionMap = {};
  const nodeTargetDataMap: TConnectionMap = {};
  const nodeTargetAudioFrameMap: TConnectionMap = {};
  const nodeTargetVideoFrameMap: TConnectionMap = {};

  backendConnections.forEach((c) => {
    const sourceNodeId = c.extension;
    const types = [
      EConnectionType.CMD,
      EConnectionType.DATA,
      EConnectionType.AUDIO_FRAME,
      EConnectionType.VIDEO_FRAME,
    ];
    types.forEach((type) => {
      if (c[type as keyof IBackendConnection]) {
        (
          c[type as keyof IBackendConnection] as Array<{
            name: string;
            dest: Array<{ extension: string; app?: string }>;
          }>
        ).forEach((item) => {
          item.dest.forEach((d) => {
            const targetNodeId = d.extension;
            const edgeId = `edge-${sourceNodeId}-${item.name}-${targetNodeId}`;
            const itemName = item.name;

            // Record the item name in the appropriate source map based on type
            let sourceMap: TConnectionMap;
            let targetMap: TConnectionMap;
            switch (type) {
              case EConnectionType.CMD:
                sourceMap = nodeSourceCmdMap;
                targetMap = nodeTargetCmdMap;
                break;
              case EConnectionType.DATA:
                sourceMap = nodeSourceDataMap;
                targetMap = nodeTargetDataMap;
                break;
              case EConnectionType.AUDIO_FRAME:
                sourceMap = nodeSourceAudioFrameMap;
                targetMap = nodeTargetAudioFrameMap;
                break;
              case EConnectionType.VIDEO_FRAME:
                sourceMap = nodeSourceVideoFrameMap;
                targetMap = nodeTargetVideoFrameMap;
                break;
              default:
                console.warn(`Unknown connection type: ${type}`);
                return;
            }

            if (!sourceMap[sourceNodeId]) {
              sourceMap[sourceNodeId] = new Set();
            }
            sourceMap[sourceNodeId].add({
              name: itemName,
              srcApp: c.app ?? "",
              destApp: d.app ?? "",
            });

            // Record the item name in the appropriate target map
            if (!targetMap[targetNodeId]) {
              targetMap[targetNodeId] = new Set();
            }
            targetMap[targetNodeId].add({
              name: itemName,
              srcApp: c.app ?? "",
              destApp: d.app ?? "",
            });

            initialEdges.push({
              id: edgeId,
              source: sourceNodeId,
              target: targetNodeId,
              data: {
                connectionType: type,
                name: itemName,
                srcApp: c.app ?? "",
                destApp: d.app ?? "",
                labelOffsetX: 0,
                labelOffsetY: 0,
              },
              type: "customEdge",
              label: itemName,
              sourceHandle: `source-${sourceNodeId}`,
              targetHandle: `target-${targetNodeId}`,
              markerEnd: {
                type: MarkerType.ArrowClosed,
                width: 20,
                height: 20,
                color: "#FF0072",
              },
            });
          });
        });
      }
    });
  });

  return {
    initialEdges,
    nodeSourceCmdMap,
    nodeSourceDataMap,
    nodeSourceAudioFrameMap,
    nodeSourceVideoFrameMap,
    nodeTargetCmdMap,
    nodeTargetDataMap,
    nodeTargetAudioFrameMap,
    nodeTargetVideoFrameMap,
  };
};

export const fetchAddonInfoForNodes = async (
  baseDir: string,
  nodes: CustomNodeType[]
): Promise<CustomNodeType[]> => {
  return await Promise.all(
    nodes.map(async (node) => {
      try {
        const addonInfoList: IExtensionAddon[] = await retrieveAddons({
          base_dir: baseDir,
          addon_name: node.data.addon,
        });
        const addonInfo: IExtensionAddon | undefined = addonInfoList?.[0];
        if (!addonInfo) {
          throw new Error();
        }
        console.log(`URL for addon '${node.data.addon}': ${addonInfo.url}`);
        return {
          ...node,
          data: {
            ...node.data,
            url: addonInfo.url,
          },
        };
      } catch (addonError: unknown) {
        console.error(
          `Failed to fetch addon info for '${node.data.addon}': ` +
            `${(addonError as Error).message}`
        );
        return node;
      }
    })
  );
};

export const enhanceNodesWithCommands = (
  nodes: CustomNodeType[],
  options: {
    nodeSourceCmdMap: TConnectionMap;
    nodeTargetCmdMap: TConnectionMap;
    nodeSourceDataMap: TConnectionMap;
    nodeSourceAudioFrameMap: TConnectionMap;
    nodeSourceVideoFrameMap: TConnectionMap;
    nodeTargetDataMap: TConnectionMap;
    nodeTargetAudioFrameMap: TConnectionMap;
    nodeTargetVideoFrameMap: TConnectionMap;
  }
): CustomNodeType[] => {
  const {
    nodeSourceCmdMap = {},
    nodeTargetCmdMap = {},
    nodeSourceDataMap = {},
    nodeSourceAudioFrameMap = {},
    nodeSourceVideoFrameMap = {},
    nodeTargetDataMap = {},
    nodeTargetAudioFrameMap = {},
    nodeTargetVideoFrameMap = {},
  } = options;

  return nodes.map((node) => {
    const sourceCmds = nodeSourceCmdMap[node.id]
      ? Array.from(nodeSourceCmdMap[node.id])
      : [];
    const targetCmds = nodeTargetCmdMap[node.id]
      ? Array.from(nodeTargetCmdMap[node.id])
      : [];

    const sourceData = nodeSourceDataMap[node.id]
      ? Array.from(nodeSourceDataMap[node.id])
      : [];
    const targetData = nodeTargetDataMap[node.id]
      ? Array.from(nodeTargetDataMap[node.id])
      : [];

    const sourceAudioFrame = nodeSourceAudioFrameMap[node.id]
      ? Array.from(nodeSourceAudioFrameMap[node.id])
      : [];
    const targetAudioFrame = nodeTargetAudioFrameMap[node.id]
      ? Array.from(nodeTargetAudioFrameMap[node.id])
      : [];

    const sourceVideoFrame = nodeSourceVideoFrameMap[node.id]
      ? Array.from(nodeSourceVideoFrameMap[node.id])
      : [];
    const targetVideoFrame = nodeTargetVideoFrameMap[node.id]
      ? Array.from(nodeTargetVideoFrameMap[node.id])
      : [];

    return {
      ...node,
      type: "customNode",
      data: {
        ...node.data,
        label: node.data.name || `${node.id}`,
        sourceCmds,
        targetCmds,
        sourceData,
        targetData,
        sourceAudioFrame,
        targetAudioFrame,
        sourceVideoFrame,
        targetVideoFrame,
      },
    };
  });
};
