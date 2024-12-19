import * as React from "react";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import { ChevronDown, ChevronUp, X } from "lucide-react"; // Import icons

const DEFAULT_WIDTH = 800;
const DEFAULT_HEIGHT = 400;

interface CommonDialogProps {
  title: string;
  children: React.ReactNode;
  className?: string;
  resizable?: boolean;
  preventFocusSteal?: boolean;
  initialWidth?: number;
  initialHeight?: number;
  onClose: () => void;
  onCollapseToggle?: (isCollapsed: boolean) => void;
}

export function CommonDialog({
  title,
  children,
  className,
  resizable = false,
  preventFocusSteal = false,
  initialWidth = DEFAULT_WIDTH,
  initialHeight = DEFAULT_HEIGHT,
  onClose,
  onCollapseToggle,
}: CommonDialogProps) {
  const dialogRef = React.useRef<HTMLDivElement>(null);
  const headerRef = React.useRef<HTMLDivElement>(null);

  const [isCollapsed, setIsCollapsed] = React.useState(false);
  const [position, setPosition] = React.useState({ x: 0, y: 0 });
  const [size, setSize] = React.useState<{ width: number; height: number }>({
    width: initialWidth,
    height: initialHeight,
  });
  const [prevHeight, setPrevHeight] = React.useState<number | null>(null);
  const [headerHeight, setHeaderHeight] = React.useState<number>(0);
  const [isDragging, setIsDragging] = React.useState(false);
  const [isResizing, setIsResizing] = React.useState(false);
  const [dragStart, setDragStart] = React.useState<{
    x: number;
    y: number;
  } | null>(null);
  const [resizeStart, setResizeStart] = React.useState<{
    x: number;
    y: number;
    width: number;
    height: number;
  } | null>(null);
  const [isVisible, setIsVisible] = React.useState(false);

  const bringToFront = React.useCallback(() => {
    const highestZIndex = Math.max(
      ...Array.from(document.querySelectorAll(".dialog-content")).map(
        (el) => parseInt(window.getComputedStyle(el).zIndex) || 0,
      ),
    );
    if (dialogRef.current) {
      dialogRef.current.style.zIndex = (highestZIndex + 1).toString();
    }
  }, []);

  const handleClick = React.useCallback(() => {
    if (!preventFocusSteal) {
      dialogRef.current?.focus();
      bringToFront();
    }
  }, [preventFocusSteal, bringToFront]);

  React.useEffect(() => {
    if (dialogRef.current) {
      const { innerWidth, innerHeight } = window;
      const rect = dialogRef.current.getBoundingClientRect();
      setPosition({
        x: (innerWidth - rect.width) / 2,
        y: (innerHeight - rect.height) / 2,
      });
      if (!preventFocusSteal) {
        dialogRef.current.focus();
      }
    }
    setIsVisible(true);
    bringToFront();
  }, [preventFocusSteal, bringToFront]);

  React.useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        onClose();
      }
    };

    if (!preventFocusSteal && dialogRef.current) {
      dialogRef.current.addEventListener("keydown", handleKeyDown);
    }

    return () => {
      if (!preventFocusSteal && dialogRef.current) {
        dialogRef.current.removeEventListener("keydown", handleKeyDown);
      }
    };
  }, [onClose, preventFocusSteal]);

  const handleMouseMove = React.useCallback(
    (e: MouseEvent) => {
      if (isDragging && dragStart) {
        setPosition({
          x: e.clientX - dragStart.x,
          y: e.clientY - dragStart.y,
        });
      }

      if (isResizing && resizeStart) {
        const newWidth = Math.max(e.clientX - resizeStart.x, 300);
        const newHeight = Math.max(e.clientY - resizeStart.y, 200);
        setSize({ width: newWidth, height: newHeight });
      }
    },
    [isDragging, dragStart, isResizing, resizeStart],
  );

  const handleMouseUp = React.useCallback(() => {
    setIsDragging(false);
    setDragStart(null);
    setIsResizing(false);
    setResizeStart(null);
  }, []);

  React.useEffect(() => {
    if (isDragging || isResizing) {
      window.addEventListener("mousemove", handleMouseMove);
      window.addEventListener("mouseup", handleMouseUp);
    }
    return () => {
      window.removeEventListener("mousemove", handleMouseMove);
      window.removeEventListener("mouseup", handleMouseUp);
    };
  }, [isDragging, isResizing, handleMouseMove, handleMouseUp]);

  const handleHeaderMouseDown = (e: React.MouseEvent) => {
    setIsDragging(true);
    setDragStart({
      x: e.clientX - position.x,
      y: e.clientY - position.y,
    });
  };

  const handleResizeMouseDown = (e: React.MouseEvent) => {
    e.stopPropagation();
    setIsResizing(true);
    setResizeStart({
      x: position.x,
      y: position.y,
      width: size.width,
      height: size.height,
    });
  };

  const toggleCollapse = () => {
    setIsCollapsed((prev) => {
      const newState = !prev;
      if (newState) {
        setPrevHeight(size.height);
        setSize((s) => ({ ...s, height: headerHeight }));
      } else if (prevHeight) {
        setSize((s) => ({ ...s, height: prevHeight }));
        setPrevHeight(null);
      }
      onCollapseToggle?.(newState);
      return newState;
    });
  };

  return (
    <Dialog open onOpenChange={() => onClose()}>
      <DialogContent
        ref={dialogRef}
        className={cn(
          "fixed p-0 border shadow-lg text-sm rounded focus:outline-none flex flex-col dialog-content",
          isVisible ? "opacity-100" : "opacity-0",
          isResizing && "select-none",
          isDragging && "select-none cursor-move",
          className,
        )}
        style={{
          top: position.y,
          left: position.x,
          width: size.width,
          height: size.height,
          transform: "none",
          zIndex: 1001,
        }}
        onClick={handleClick}
        onMouseDownCapture={() => {
          if (!preventFocusSteal) {
            dialogRef.current?.focus();
          }
          bringToFront();
        }}
      >
        <DialogHeader
          ref={headerRef}
          className="bg-[var(--popup-header-bg)] p-2.5 flex justify-between items-center cursor-move select-none border-b"
          onMouseDown={handleHeaderMouseDown}
        >
          <DialogTitle className="font-bold">{title}</DialogTitle>
          <div className="flex items-center">
            <Button
              variant="ghost"
              size="sm"
              className="h-auto p-1.5"
              onClick={toggleCollapse}
            >
              {isCollapsed ? <ChevronDown /> : <ChevronUp />}
            </Button>
            <Button
              variant="ghost"
              size="sm"
              className="h-auto p-1.5"
              onClick={onClose}
            >
              <X />
            </Button>
          </div>
        </DialogHeader>

        <div
          className={cn(
            "p-2.5 overflow-hidden flex w-full h-full",
            isCollapsed ? "invisible h-0 py-0" : "visible",
          )}
        >
          {children}
        </div>

        {resizable && !isCollapsed && (
          <div
            className="w-[5px] h-[5px] bg-transparent absolute right-0 bottom-0 cursor-se-resize z-[1002]"
            onMouseDown={handleResizeMouseDown}
          />
        )}
      </DialogContent>
    </Dialog>
  );
}
