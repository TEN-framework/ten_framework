//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import {
  type EdgeProps,
  getBezierPath,
  type Edge,
  BaseEdge,
  EdgeLabelRenderer,
} from "@xyflow/react";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/Button";

export type CustomEdgeType = Edge<
  { labelOffsetX: number; labelOffsetY: number },
  "customEdge"
>;

export function CustomEdge({
  sourceX,
  sourceY,
  targetX,
  targetY,
  sourcePosition,
  targetPosition,
  data,
  id,
  style,
  label,
  selected,
}: EdgeProps<CustomEdgeType>) {
  const [edgePath, labelX, labelY] = getBezierPath({
    sourceX,
    sourceY,
    sourcePosition,
    targetX,
    targetY,
    targetPosition,
  });

  // const onEdgeClick = () => {
  //   console.log("onEdgeClick === ", id, data);
  // };

  return (
    <>
      <BaseEdge id={id} path={edgePath} style={style} />
      {selected && (
        <path
          id={id}
          d={edgePath}
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
      <EdgeLabelRenderer>
        <div
          className="absolute pointer-events-all origin-center nodrag nopan"
          style={{
            // eslint-disable-next-line max-len
            transform: `translate(-50%, -50%) translate(${labelX}px,${labelY}px)`,
          }}
        >
          <div
            className={cn(
              "cursor-pointer rounded-lg border bg-background",
              "hover:bg-muted text-xs pt-0 px-2 py-1"
            )}
            // onClick={onEdgeClick}
          >
            <div className="flex flex-col gap-0.5">
              <div className="flex gap-1">
                <span className="text-muted-foreground">C:</span>2
              </div>
              <div className="flex gap-1">
                <span className="text-muted-foreground">D:</span>1
              </div>
              <div className="flex gap-1">
                <span className="text-muted-foreground">A:</span>0
              </div>
              <div className="flex gap-1">
                <span className="text-muted-foreground">V:</span>5
              </div>
            </div>
          </div>
        </div>
      </EdgeLabelRenderer>
    </>
  );
}

export default CustomEdge;
