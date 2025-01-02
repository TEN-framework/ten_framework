import * as React from "react";

import type { CustomNodeType } from "@/flow/CustomNode";
import type { CustomEdgeType } from "@/flow/CustomEdge";
import { EConnectionType } from "@/types/graphs";

export type TReactFlowDataContext = {
  nodes: CustomNodeType[];
  edges: CustomEdgeType[];
};

export const ReactFlowDataContext = React.createContext<TReactFlowDataContext>({
  nodes: [],
  edges: [],
});

// --- hooks ---
export const useCDAVInfoByEdgeId = (edgeId: string) => {
  const { nodes, edges } = React.useContext(ReactFlowDataContext);
  console.log("[useCDAVInfoByEdgeId] nodes === ", nodes);
  console.log("[useCDAVInfoByEdgeId] edges === ", edges);
  const edge = edges.find((e) => e.id === edgeId);
  if (!edge) return null;

  console.log("[useCDAVInfoByEdgeId] edge === ", edge);

  const source = nodes.find((n) => n.id === edge.source);
  const target = nodes.find((n) => n.id === edge.target);
  if (!source || !target) return null;

  // Find all edges between the same source and target nodes
  const relatedEdges = edges.filter(
    (e) => e.source === edge.source && e.target === edge.target
  );

  // Count connections between source and target nodes
  let cmdCount = 0;
  let dataCount = 0;
  let audioCount = 0;
  let videoCount = 0;

  // Count all connection types from related edges
  relatedEdges.forEach((e) => {
    switch (e.data?.connectionType) {
      case EConnectionType.CMD:
        cmdCount++;
        break;
      case EConnectionType.DATA:
        dataCount++;
        break;
      case EConnectionType.AUDIO_FRAME:
        audioCount++;
        break;
      case EConnectionType.VIDEO_FRAME:
        videoCount++;
        break;
    }
  });

  return {
    source,
    target,
    relatedEdges,
    connectionCounts: {
      cmd: cmdCount,
      data: dataCount,
      audio: audioCount,
      video: videoCount,
    },
  };
};
