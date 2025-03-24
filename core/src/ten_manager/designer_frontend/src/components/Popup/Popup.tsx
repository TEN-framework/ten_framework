//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import * as React from "react";
import { ChevronDown, ChevronUp, X } from "lucide-react";
import {
  motion,
  useDragControls,
  useMotionValue,
  type PanInfo,
} from "motion/react";

import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/Button";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/Tooltip";
import { useWidgetStore } from "@/store";

const POPUP_MIN_HEIGHT = 100;
const POPUP_MIN_WIDTH = 100;

export const Popup = (props: {
  id: string;
  title: string | React.ReactNode;
  children: React.ReactNode;
  className?: string;
  resizable?: boolean;
  collapsible?: boolean;
  /** @deprecated */
  autoFocus?: boolean;
  customActions?: {
    id: string;
    label: string;
    Icon: (props: React.SVGProps<SVGSVGElement>) => React.ReactNode;
    onClick: () => void;
  }[];
  contentClassName?: string;
  width?: number;
  height?: number;
  maxWidth?: number;
  maxHeight?: number;
  onClose?: () => Promise<void> | void;
  onCollapseToggle?: (isCollapsed: boolean) => void;
}) => {
  const {
    id,
    children,
    className,
    title,
    resizable = false,
    collapsible = true,
    autoFocus = false,
    customActions,
    contentClassName,
    width,
    height,
    maxWidth = Infinity,
    maxHeight = Infinity,
    onClose,
    onCollapseToggle,
  } = props;

  const [isCollapsed, setIsCollapsed] = React.useState(false);
  const [isResized, setIsResized] = React.useState(false);

  const popupRef = React.useRef<HTMLDivElement>(null);

  const { removeWidget } = useWidgetStore();
  const dragControls = useDragControls();
  const popupHeight = useMotionValue(
    height ||
      popupRef.current?.getBoundingClientRect().height ||
      POPUP_MIN_HEIGHT
  );
  const popupWidth = useMotionValue(
    width || popupRef.current?.getBoundingClientRect().width || POPUP_MIN_WIDTH
  );

  const preResizeCallback = React.useCallback(() => {
    if (isResized) return;
    setIsResized(true);
    const currentHeight = popupRef.current?.getBoundingClientRect().height;
    const currentWidth = popupRef.current?.getBoundingClientRect().width;
    popupHeight.set(currentHeight || POPUP_MIN_HEIGHT);
    popupWidth.set(currentWidth || POPUP_MIN_WIDTH);
  }, [isResized, popupHeight, popupWidth]);

  const handleResize = React.useCallback(
    (mode: "x" | "y" | "all") =>
      (_event: MouseEvent | TouchEvent | PointerEvent, info: PanInfo) => {
        preResizeCallback();

        if (mode === "x" || mode === "all") {
          const newWidth = popupWidth.get() + info.delta.x;
          if (newWidth >= POPUP_MIN_WIDTH && newWidth <= maxWidth) {
            popupWidth.set(newWidth);
          }
        }

        if (mode === "y" || mode === "all") {
          const newHeight = popupHeight.get() + info.delta.y;
          if (newHeight >= POPUP_MIN_HEIGHT && newHeight <= maxHeight) {
            popupHeight.set(newHeight);
          }
        }
      },
    [preResizeCallback, popupWidth, popupHeight, maxWidth, maxHeight]
  );

  const handleResizeX = handleResize("x");
  const handleResizeY = handleResize("y");
  const handleResizeXY = handleResize("all");

  const handleClose = React.useCallback(() => {
    if (onClose) {
      onClose();
    } else {
      removeWidget(id);
    }
  }, [onClose, id, removeWidget]);

  const handleBringToFront = React.useCallback(() => {
    const highestZIndex = Math.max(
      ...Array.from(document.querySelectorAll(".popup")).map(
        (el) => parseInt(window.getComputedStyle(el).zIndex) || 0
      )
    );
    if (popupRef.current) {
      popupRef.current.style.zIndex = (highestZIndex + 1).toString();
    }
  }, []);

  React.useEffect(() => {
    if (popupRef.current) {
      const { innerWidth, innerHeight } = window;
      const rect = popupRef.current.getBoundingClientRect();
      popupRef.current.style.left = `${(innerWidth - rect.width) / 2}px`;
      popupRef.current.style.top = `${(innerHeight - rect.height) / 2}px`;
      handleBringToFront();
      if (autoFocus) {
        popupRef.current.focus();
      }
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [autoFocus]);

  return (
    <motion.div
      ref={popupRef}
      id={id}
      drag
      dragMomentum={false}
      dragControls={dragControls}
      dragListener={false}
      onMouseDown={handleBringToFront}
      className={cn(
        "popup",
        "fixed text-sm overflow-hidden",
        "backdrop-blur-xs shadow-xl ",
        "bg-slate-50/80 dark:bg-gray-900/90",
        "border border-slate-100/50 dark:border-gray-900/50",
        "ring-1 ring-slate-100/50 dark:ring-gray-900/50",
        "text-foreground rounded-lg focus:outline-hidden flex flex-col",
        "transition-opacity duration-200 ease-in-out",
        className
      )}
      style={{
        ...(isResized
          ? {
              height: isCollapsed ? "auto" : popupHeight,
              width: popupWidth,
            }
          : {
              height: (!isCollapsed && height) || "auto",
              width: width || "auto",
            }),
        ...(maxWidth !== Infinity && { maxWidth }),
        ...(maxHeight !== Infinity && { maxHeight }),
      }}
    >
      <motion.div
        onPointerDown={(event) => dragControls.start(event)}
        className={cn(
          "p-2.5 flex justify-between items-center cursor-move select-none",
          "bg-slate-100/80 dark:bg-gray-900/80",
          "rounded-t-lg",
          {
            ["border-b border-border/50"]: collapsible && !isCollapsed,
          }
        )}
      >
        {typeof title === "string" ? (
          <span className="font-medium text-foreground/90 font-sans">
            {title}
          </span>
        ) : (
          title
        )}
        <div className="flex items-center gap-1.5">
          {customActions?.map((action) => (
            <TooltipProvider>
              <Tooltip>
                <TooltipTrigger asChild>
                  <Button
                    variant="ghost"
                    size="sm"
                    className="h-auto p-1.5 cursor-pointer"
                    onClick={action.onClick}
                  >
                    <action.Icon className="opacity-70" />
                  </Button>
                </TooltipTrigger>
                <TooltipContent>
                  <p>{action.label}</p>
                </TooltipContent>
              </Tooltip>
            </TooltipProvider>
          ))}
          {collapsible && (
            <Button
              variant="ghost"
              size="sm"
              className="h-auto p-1.5 transition-colors cursor-pointer"
              onClick={() => {
                setIsCollapsed(!isCollapsed);
                onCollapseToggle?.(isCollapsed);
              }}
            >
              {isCollapsed ? (
                <ChevronDown className="opacity-70" />
              ) : (
                <ChevronUp className="opacity-70" />
              )}
            </Button>
          )}
          <Button
            variant="ghost"
            size="sm"
            className="h-auto p-1.5 transition-colors cursor-pointer"
            onClick={handleClose}
          >
            <X className="opacity-70" />
          </Button>
        </div>
      </motion.div>
      <motion.div
        className={cn(
          "p-2.5 overflow-hidden flex w-full",
          {
            ["py-0 h-0 opacity-0"]: collapsible && isCollapsed,
            ["h-full opacity-100"]: !isCollapsed,
          },
          contentClassName
        )}
      >
        {children}
      </motion.div>
      {/* bottom resize handler */}
      {resizable && (
        <motion.div
          className={cn(
            "absolute bottom-0 left-0 right-1",
            "h-0.5 cursor-ns-resize bg-transparent"
          )}
          drag="y"
          dragConstraints={{ top: 0, left: 0, right: 0, bottom: 0 }}
          dragElastic={0}
          dragMomentum={false}
          onDrag={handleResizeY}
        />
      )}
      {/* right resize handler */}
      {resizable && (
        <motion.div
          className={cn(
            "absolute right-0 top-0 bottom-1",
            "w-0.5 cursor-ew-resize bg-transparent"
          )}
          drag="x"
          dragConstraints={{ top: 0, left: 0, right: 0, bottom: 0 }}
          dragElastic={0}
          onDrag={handleResizeX}
        />
      )}
      {/* right bottom resize handler */}
      {resizable && (
        <motion.div
          className={cn(
            "absolute right-0 bottom-0",
            "size-1 cursor-se-resize bg-transparent"
          )}
          drag="x"
          dragConstraints={{ top: 0, left: 0, right: 0, bottom: 0 }}
          dragElastic={0}
          onDrag={handleResizeXY}
        />
      )}
    </motion.div>
  );
};
