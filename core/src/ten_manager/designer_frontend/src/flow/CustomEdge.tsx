//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import {
  type EdgeProps,
  type Edge,
  BaseEdge,
  getSmoothStepPath,
  Position,
} from "@xyflow/react";

import { type EConnectionType } from "@/types/graphs";

export type CustomEdgeType = Edge<
  {
    labelOffsetX: number;
    labelOffsetY: number;
    connectionType: EConnectionType;
  },
  "customEdge"
>;

export function CustomEdge({
  sourceX,
  sourceY,
  targetX,
  targetY,
  id,
  style,
  selected,
  markerEnd,
}: EdgeProps<CustomEdgeType>) {
  const [path] = getSmoothStepPath({
    sourceX: sourceX,
    sourceY: sourceY,
    sourcePosition: Position.Right,
    targetX: targetX,
    targetY: targetY,
    targetPosition: Position.Left,
  });

  return (
    <>
      <BaseEdge id={id} path={path} style={style} markerEnd={markerEnd} />
      {selected && (
        <path
          id={id}
          d={path}
          fill="none"
          strokeDasharray="5,5"
          stroke="url(#edge-gradient)"
          strokeWidth="2"
          opacity="0.75"
        >
          <animate
            attributeName="stroke-dashoffset"
            values="10;0"
            dur="0.75s"
            repeatCount="indefinite"
          />
        </path>
      )}
    </>
  );
}

export default CustomEdge;
