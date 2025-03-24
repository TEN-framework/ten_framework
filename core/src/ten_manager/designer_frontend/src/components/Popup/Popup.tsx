//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import React, { useState, useEffect, useRef } from "react";
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

const DEFAULT_WIDTH = 800;
const DEFAULT_HEIGHT = 400;

interface PopupProps {
  id?: string;
  title: string | React.ReactNode;
  children: React.ReactNode;
  className?: string;
  resizable?: boolean;

  // When a popup appears, it takes keyboard focus, allowing the popup to be
  // closed directly by pressing "Esc." However, in some scenarios, the popup
  // should not take keyboard focus. For example, in the case of xterm, if the
  // popup takes keyboard focus, xterm will continuously fail to regain keyboard
  //  focus, making it impossible to receive input from the keyboard.
  preventFocusSteal?: boolean;

  initialWidth?: number;
  initialHeight?: number;

  onClose: () => Promise<void> | void;
  onCollapseToggle?: (isCollapsed: boolean) => void;
  contentClassName?: string;
  customActions?: {
    id: string;
    label: string;
    Icon: (props: React.SVGProps<SVGSVGElement>) => React.ReactNode;
    onClick: () => void;
  }[];
}

const PopupLegacy: React.FC<PopupProps> = ({
  title,
  children,
  className,
  contentClassName,
  resizable = false, // Default to non-resizable.
  preventFocusSteal = false,
  initialWidth,
  initialHeight,
  onClose,
  onCollapseToggle,
  customActions,
}) => {
  const popupRef = useRef<HTMLDivElement>(null);
  const headerRef = useRef<HTMLDivElement>(null);

  const [isCollapsed, setIsCollapsed] = useState(false);
  const [isVisible, setIsVisible] = useState(false);

  const [position, setPosition] = useState({ x: 0, y: 0 });
  const [size, setSize] = useState<{ width?: number; height?: number }>({
    width: initialWidth,
    height: initialHeight,
  });
  const [prevHeight, setPrevHeight] = useState<number | null>(null);
  const [headerHeight, setHeaderHeight] = useState<number>(0);
  const [isResizing, setIsResizing] = useState(false);
  const [resizeStart, setResizeStart] = useState<{
    x: number;
    y: number;
    width: number;
    height: number;
  } | null>(null);

  const [isDragging, setIsDragging] = useState(false);
  const [dragStart, setDragStart] = useState<{ x: number; y: number } | null>(
    null
  );

  // Center the popup on mount.
  useEffect(() => {
    if (popupRef.current) {
      const { innerWidth, innerHeight } = window;
      const rect = popupRef.current.getBoundingClientRect();
      setPosition({
        x: (innerWidth - rect.width) / 2,
        y: (innerHeight - rect.height) / 2,
      });
      if (!preventFocusSteal) {
        popupRef.current.focus();
      }
    }
    setIsVisible(true);
  }, [preventFocusSteal]);

  // If an initial size is not provided, the actual size will be determined
  // after the popup becomes visible.
  useEffect(() => {
    if (
      isVisible &&
      popupRef.current &&
      size.width === undefined &&
      size.height === undefined
    ) {
      const rect = popupRef.current.getBoundingClientRect();
      setSize({ width: rect.width, height: rect.height });
    }
  }, [isVisible, size.width, size.height]);

  // Handle keydown for ESC.
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        onClose();
      }
    };

    // Only add listener if this popup should take focus.
    if (!preventFocusSteal && popupRef.current) {
      const current = popupRef.current;
      current.addEventListener("keydown", handleKeyDown);

      // Cleanup function uses the captured reference.
      return () => {
        current.removeEventListener("keydown", handleKeyDown);
      };
    }

    // If the condition isn't met, no cleanup is necessary.
    return undefined;
  }, [onClose, preventFocusSteal]);

  const handleMouseDown = (e: React.MouseEvent) => {
    // Bring to front immediately on mouse down.
    bringToFront();

    setIsDragging(true);
    setDragStart({ x: e.clientX - position.x, y: e.clientY - position.y });
  };

  useEffect(() => {
    const handleMouseMove = (e: MouseEvent) => {
      if (isDragging && dragStart) {
        setPosition({ x: e.clientX - dragStart.x, y: e.clientY - dragStart.y });
      }

      if (isResizing && resizeStart) {
        const newWidth = Math.max(e.clientX - resizeStart.x, 300);
        const newHeight = Math.max(e.clientY - resizeStart.y, 200);
        setSize({
          width: newWidth,
          height: newHeight,
        });
      }
    };

    const handleMouseUp = () => {
      setIsDragging(false);
      setDragStart(null);

      setIsResizing(false);
      setResizeStart(null);
    };

    if (isDragging || isResizing) {
      window.addEventListener("mousemove", handleMouseMove);
      window.addEventListener("mouseup", handleMouseUp);
    } else {
      window.removeEventListener("mousemove", handleMouseMove);
      window.removeEventListener("mouseup", handleMouseUp);
    }

    return () => {
      window.removeEventListener("mousemove", handleMouseMove);
      window.removeEventListener("mouseup", handleMouseUp);
    };
  }, [isDragging, isResizing, dragStart, resizeStart]);

  const handleResizeMouseDown = (e: React.MouseEvent) => {
    e.stopPropagation(); // Prevent from triggering dragging.
    setIsResizing(true);
    setResizeStart({
      x: position.x,
      y: position.y,
      width: size.width ?? DEFAULT_WIDTH,
      height: size.height ?? DEFAULT_HEIGHT,
    });
  };

  useEffect(() => {
    if (headerRef.current) {
      const height = headerRef.current.getBoundingClientRect().height;
      setHeaderHeight(height);
    }
    bringToFront();
  }, []);

  const toggleCollapse = () => {
    setIsCollapsed((prev) => {
      const newState = !prev;
      if (newState) {
        // Store the current height before collapsing.
        setPrevHeight(size.height ?? null);

        // Set the height to the header's height to collapse.
        setSize((prevSize) => ({ ...prevSize, height: headerHeight }));
      } else if (prevHeight !== null) {
        // Restore the previous height when un-collapsing.
        setSize((prevSize) => ({ ...prevSize, height: prevHeight }));

        setPrevHeight(null);
      }
      onCollapseToggle?.(newState);
      return newState;
    });
  };

  // Handle focus and bring to front when clicked.
  const handleClick = () => {
    if (!preventFocusSteal) {
      popupRef.current?.focus();
      bringToFront();
    }
  };

  const bringToFront = () => {
    const highestZIndex = Math.max(
      ...Array.from(document.querySelectorAll(".popup")).map(
        (el) => parseInt(window.getComputedStyle(el).zIndex) || 0
      )
    );
    if (popupRef.current) {
      popupRef.current.style.zIndex = (highestZIndex + 1).toString();
    }
  };

  return (
    <div
      className={cn(
        "fixed text-sm overflow-hidden",
        "backdrop-blur-xs shadow-xl ",
        "bg-slate-50/80 dark:bg-gray-900/90",
        "border border-slate-100/50 dark:border-gray-900/50",
        "ring-1 ring-slate-100/50 dark:ring-gray-900/50",
        "text-foreground rounded-lg focus:outline-hidden flex flex-col popup",
        "transition-opacity duration-200 ease-in-out",
        isVisible ? "opacity-100" : "opacity-0",
        isResizing && "select-none",
        isDragging && "select-none cursor-move",
        className
      )}
      style={{
        top: position.y,
        left: position.x,
        zIndex: 1001,
        ...(size.width !== undefined && size.height !== undefined
          ? { width: size.width, height: size.height }
          : {}),
      }}
      ref={popupRef}
      tabIndex={-1}
      onClick={handleClick}
      onMouseDownCapture={() => {
        if (!preventFocusSteal) {
          popupRef.current?.focus();
        }
        bringToFront();
      }}
    >
      <div
        className={cn(
          "p-2.5 flex justify-between items-center cursor-move select-none",
          "bg-slate-100/80 dark:bg-gray-900/80",
          "rounded-t-lg",
          {
            ["border-b border-border/50"]: !isCollapsed,
          }
        )}
        onMouseDown={handleMouseDown}
        ref={headerRef}
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
                    className="h-auto p-1.5"
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
          <Button
            variant="ghost"
            size="sm"
            className="h-auto p-1.5 transition-colors"
            onClick={toggleCollapse}
          >
            {isCollapsed ? (
              <ChevronDown className="opacity-70" />
            ) : (
              <ChevronUp className="opacity-70" />
            )}
          </Button>
          <Button
            variant="ghost"
            size="sm"
            className="h-auto p-1.5 transition-colors"
            onClick={onClose}
          >
            <X className="opacity-70" />
          </Button>
        </div>
      </div>

      <div
        className={cn(
          "p-2.5 overflow-hidden flex w-full h-full",
          {
            ["invisible h-0 py-0"]: isCollapsed,
          },
          contentClassName
        )}
      >
        {children}
      </div>

      {resizable && !isCollapsed && (
        <div
          className={cn(
            "w-[5px] h-[5px] bg-transparent cursor-se-resize",
            "absolute right-0 bottom-0 z-1002"
          )}
          onMouseDown={handleResizeMouseDown}
        />
      )}
    </div>
  );
};

export default PopupLegacy;

const POPUP_MIN_HEIGHT = 100;
const POPUP_MIN_WIDTH = 100;

export const Popup = (props: {
  id: string;
  title: string | React.ReactNode;
  children: React.ReactNode;
  className?: string;
  resizable?: boolean;
  collapsible?: boolean;
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

  const [isCollapsed, setIsCollapsed] = useState(false);
  const [isResized, setIsResized] = useState(false);

  const popupRef = useRef<HTMLDivElement>(null);

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

  React.useEffect(() => {
    if (popupRef.current) {
      const { innerWidth, innerHeight } = window;
      const rect = popupRef.current.getBoundingClientRect();
      popupRef.current.style.left = `${(innerWidth - rect.width) / 2}px`;
      popupRef.current.style.top = `${(innerHeight - rect.height) / 2}px`;
      if (autoFocus) {
        popupRef.current.focus();
      }
    }
  }, [autoFocus]);

  return (
    <motion.div
      ref={popupRef}
      id={id}
      drag
      dragMomentum={false}
      dragControls={dragControls}
      dragListener={false}
      className={cn(
        "fixed text-sm overflow-hidden",
        "backdrop-blur-xs shadow-xl ",
        "bg-slate-50/80 dark:bg-gray-900/90",
        "border border-slate-100/50 dark:border-gray-900/50",
        "ring-1 ring-slate-100/50 dark:ring-gray-900/50",
        "text-foreground rounded-lg focus:outline-hidden flex flex-col popup",
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
              height: "auto",
              width: "auto",
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
          "p-2.5 overflow-hidden flex w-full h-full",
          {
            ["py-0"]: collapsible && isCollapsed,
          },
          contentClassName
        )}
        initial={{
          height: "auto",
          opacity: 1,
        }}
        animate={{
          height: isCollapsed ? 0 : "auto",
          opacity: isCollapsed ? 0 : 1,
        }}
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
