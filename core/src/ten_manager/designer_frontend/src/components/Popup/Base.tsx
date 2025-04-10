//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import * as React from "react";
import { ChevronDown, ChevronUp, X, XIcon } from "lucide-react";
import {
  motion,
  useDragControls,
  useMotionValue,
  type PanInfo,
} from "motion/react";

import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/Button";
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuTrigger,
  ContextMenuSeparator,
} from "@/components/ui/ContextMenu";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/Tooltip";
import { useWidgetStore } from "@/store";
import { ECustomEventName } from "@/utils/popup";
import { TWidgetCustomAction } from "@/types/widgets";

const POPUP_MIN_HEIGHT = 200;
const POPUP_MIN_WIDTH = 340;

export interface IPopupBaseProps {
  id: string;
  title?: string | React.ReactNode;
  children: React.ReactNode;
  className?: string;
  contentClassName?: string;
  defaultWidth?: number;
  defaultHeight?: number;
  maxWidth?: number;
  maxHeight?: number;
  initialPosition?:
    | "center"
    | "top-left"
    | "top-right"
    | "bottom-left"
    | "bottom-right";
  customActions?: TWidgetCustomAction[];
  onResizing?: () => void;
  onResized?: () => void;
  onClose?: () => Promise<void> | void;
  onCollapseToggle?: (isCollapsed: boolean) => void;
}

export const PopupTabsBar = (props: {
  children?: React.ReactNode;
  className?: string;
}) => {
  const { children, className } = props;

  return (
    <ul
      className={cn(
        "w-full h-8 flex items-center overflow-x-auto overflow-y-hidden",
        "scroll-p-1",
        "bg-border dark:bg-popover",
        className
      )}
    >
      {children}
    </ul>
  );
};

export const PopupTabsBarItem = (props: {
  id: string;
  children?: React.ReactNode | string;
  className?: string;
  isActive?: boolean;
  onSelect?: (id: string) => void;
  onClose?: (id: string) => void;
  actions?: TWidgetCustomAction[];
}) => {
  const { id, children, onClose, isActive, actions, className, onSelect } =
    props;

  return (
    <ContextMenu>
      <ContextMenuTrigger asChild>
        <li
          className={cn(
            "w-fit flex items-center gap-2 px-2 py-1 text-xs cursor-pointer",
            "border-b-2 border-transparent",
            {
              "text-primary border-purple-900": isActive,
            },
            "hover:text-primary",
            className
          )}
          onClick={() => onSelect?.(id)}
        >
          <div className={cn("truncate max-w-[150px]")}>{children}</div>
          {onClose && (
            <XIcon
              className="size-3 ml-1 text-foreground hover:text-destructive"
              onClick={(e) => {
                e.stopPropagation();
                onClose?.(id);
              }}
            />
          )}
        </li>
      </ContextMenuTrigger>
      <ContextMenuContent>
        {actions?.map((action) => (
          <ContextMenuItem key={action.id} inset onClick={action.onClick}>
            {action.label}
          </ContextMenuItem>
        ))}
        {actions && actions.length > 0 && onClose && <ContextMenuSeparator />}
        {onClose && (
          <ContextMenuItem inset onClick={() => onClose?.(id)}>
            Close
          </ContextMenuItem>
        )}
      </ContextMenuContent>
    </ContextMenu>
  );
};

export const PopupTabsBarContent = (props: {
  children?: React.ReactNode;
  className?: string;
  isActive?: boolean;
}) => {
  const { children, className, isActive } = props;

  return (
    <div
      className={cn(
        "w-full h-[calc(100%-32px)]",
        {
          ["hidden"]: !isActive,
        },
        className
      )}
    >
      {children}
    </div>
  );
};

export const PopupBase = (props: IPopupBaseProps) => {
  const {
    id,
    children,
    className,
    title = null,
    contentClassName,
    defaultWidth = POPUP_MIN_WIDTH,
    defaultHeight = POPUP_MIN_HEIGHT,
    maxWidth = Infinity,
    maxHeight = Infinity,
    customActions,
    initialPosition = "center",
    onClose,
    onCollapseToggle,
    onResizing,
    onResized,
  } = props;

  const [isCollapsed, setIsCollapsed] = React.useState(false);
  const [isResized, setIsResized] = React.useState(false);
  const [isResizing, setIsResizing] = React.useState(false);

  const popupRef = React.useRef<HTMLDivElement>(null);

  const { removeWidget } = useWidgetStore();
  const dragControls = useDragControls();
  const popupHeight = useMotionValue(
    defaultHeight ||
      popupRef.current?.getBoundingClientRect().height ||
      POPUP_MIN_HEIGHT
  );
  const popupWidth = useMotionValue(
    defaultWidth ||
      popupRef.current?.getBoundingClientRect().width ||
      POPUP_MIN_WIDTH
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

  const handleBringToFront = React.useCallback((tab_id?: string) => {
    const highestZIndex = Math.max(
      ...Array.from(document.querySelectorAll(".popup")).map(
        (el) => parseInt(window.getComputedStyle(el).zIndex) || 0
      )
    );
    // if (tab_id) {
    //   onTabIdUpdate?.(tab_id);
    // }
    if (popupRef.current) {
      if (highestZIndex === parseInt(popupRef.current.style.zIndex)) return;
      popupRef.current.style.zIndex = (highestZIndex + 1).toString();
    }
  }, []);

  React.useEffect(() => {
    if (popupRef.current) {
      const { innerWidth, innerHeight } = window;
      const rect = popupRef.current.getBoundingClientRect();
      if (initialPosition === "center") {
        popupRef.current.style.left = `${(innerWidth - rect.width) / 2}px`;
        popupRef.current.style.top = `${(innerHeight - rect.height) / 2}px`;
      } else if (initialPosition === "top-left") {
        popupRef.current.style.left = `40px`;
        popupRef.current.style.top = `60px`;
      } else if (initialPosition === "top-right") {
        popupRef.current.style.left = `${innerWidth - rect.width - 40}px`;
        popupRef.current.style.top = `60px`;
      } else if (initialPosition === "bottom-left") {
        popupRef.current.style.left = `40px`;
        popupRef.current.style.top = `${innerHeight - rect.height - 40}px`;
      } else if (initialPosition === "bottom-right") {
        popupRef.current.style.left = `${innerWidth - rect.width - 40}px`;
        popupRef.current.style.top = `${innerHeight - rect.height - 40}px`;
      }
      handleBringToFront();
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [initialPosition]);

  React.useEffect(() => {
    const bringToFrontEvent = (event: CustomEvent) => {
      if (event.detail.id === id) {
        handleBringToFront(event.detail.tab_id);
      }
    };

    window.addEventListener(
      ECustomEventName.BringToFrontPopup,
      bringToFrontEvent as EventListener
    );

    return () => {
      window.removeEventListener(
        ECustomEventName.BringToFrontPopup,
        bringToFrontEvent as EventListener
      );
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return (
    <motion.div
      ref={popupRef}
      id={id}
      drag
      dragMomentum={false}
      dragControls={dragControls}
      dragListener={false}
      onMouseDown={() => {
        handleBringToFront();
      }}
      className={cn(
        "popup",
        "fixed text-sm overflow-hidden",
        "backdrop-blur-xs shadow-xl ",
        "bg-slate-50/80 dark:bg-gray-800/90",
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
              height: (!isCollapsed && defaultHeight) || "auto",
              width: defaultWidth || "auto",
            }),
        ...(maxWidth !== Infinity && { maxWidth }),
        ...(maxHeight !== Infinity && { maxHeight }),
      }}
    >
      <motion.div
        onPointerDown={(event) => dragControls.start(event)}
        className={cn(
          "px-2.5 py-1",
          "flex justify-between items-center cursor-move select-none",
          "bg-slate-100/80 dark:bg-gray-900/80",
          "rounded-t-lg",
          {
            ["border-b border-border/50"]: !isCollapsed,
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
        <div className="flex items-center gap-1.5 ml-auto">
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
            ["py-0 h-0 opacity-0"]: isCollapsed,
            ["h-full opacity-100"]: !isCollapsed,
          },
          contentClassName
        )}
        style={{ pointerEvents: isResizing ? "none" : "auto" }}
      >
        {children}
      </motion.div>

      {/* bottom resize handler */}
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
        onMouseDown={() => {
          setIsResizing(true);
          onResizing?.();
        }}
        onDragEnd={() => {
          setIsResizing(false);
          onResized?.();
        }}
      />

      {/* right resize handler */}
      <motion.div
        className={cn(
          "absolute right-0 top-0 bottom-1",
          "w-0.5 cursor-ew-resize bg-transparent"
        )}
        drag="x"
        dragConstraints={{ top: 0, left: 0, right: 0, bottom: 0 }}
        dragElastic={0}
        onDrag={handleResizeX}
        onMouseDown={() => {
          setIsResizing(true);
          onResizing?.();
        }}
        onDragEnd={() => {
          setIsResizing(false);
          onResized?.();
        }}
      />

      {/* right bottom resize handler */}
      <motion.div
        className={cn(
          "absolute right-0 bottom-0",
          "size-1 cursor-se-resize bg-transparent"
        )}
        drag="x"
        dragConstraints={{ top: 0, left: 0, right: 0, bottom: 0 }}
        dragElastic={0}
        onDrag={handleResizeXY}
        onMouseDown={() => {
          setIsResizing(true);
          onResizing?.();
        }}
        onDragEnd={() => {
          setIsResizing(false);
          onResized?.();
        }}
      />
    </motion.div>
  );
};
