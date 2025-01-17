import * as React from "react";
import {
  XIcon,
  EllipsisVerticalIcon,
  PanelRightIcon,
  PanelLeftIcon,
  PanelBottomIcon,
  SquareArrowOutUpRightIcon,
} from "lucide-react";

import { cn } from "@/lib/utils";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuLabel,
  DropdownMenuRadioGroup,
  DropdownMenuRadioItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/DropdownMenu";
import {
  ContextMenu,
  ContextMenuCheckboxItem,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuLabel,
  ContextMenuRadioGroup,
  ContextMenuRadioItem,
  ContextMenuSeparator,
  ContextMenuShortcut,
  ContextMenuSub,
  ContextMenuSubContent,
  ContextMenuSubTrigger,
  ContextMenuTrigger,
} from "@/components/ui/ContextMenu";
import { useWidgetStore } from "@/store/widget";
import { EWidgetCategory, EWidgetDisplayType, IWidget } from "@/types/widgets";
import TerminalWidget from "@/components/Widget/TerminalWidget";
import EditorWidget from "@/components/Widget/EditorWidget";

export default function DockContainer(props: {
  position?: string;
  onPositionChange?: (position: string) => void;
  className?: string;
}) {
  const { position = "bottom", onPositionChange, className } = props;

  const [selectedWidgetId, setSelectedWidgetId] = React.useState<string | null>(
    null
  );

  const { widgets, removeWidget, removeWidgets, updateWidgetDisplayType } =
    useWidgetStore();

  const dockWidgetsMemo = React.useMemo(
    () =>
      widgets.filter(
        (widget) => widget.display_type === EWidgetDisplayType.Dock
      ),
    [widgets]
  );

  const selectedWidgetMemo = React.useMemo(
    () => dockWidgetsMemo.find((widget) => widget.id === selectedWidgetId),
    [dockWidgetsMemo, selectedWidgetId]
  );

  React.useEffect(() => {
    const selectedWidget = dockWidgetsMemo.find(
      (widget) => widget.id === selectedWidgetId
    );
    if (dockWidgetsMemo.length > 0 && !selectedWidget) {
      setSelectedWidgetId(dockWidgetsMemo[0].id);
    }
  }, [dockWidgetsMemo, selectedWidgetId]);

  const handlePopout = (id: string) => {
    updateWidgetDisplayType(id, EWidgetDisplayType.Popup);
  };

  return (
    <div
      className={cn("w-full h-full bg-muted text-muted-foreground", className)}
    >
      <DockHeader
        className="text-primary"
        position={position}
        onPositionChange={onPositionChange}
        onClose={() => {
          removeWidgets(dockWidgetsMemo.map((w) => w.id));
        }}
      >
        <ul className="w-full overflow-x-auto flex items-center gap-2">
          {dockWidgetsMemo.map((widget) => (
            <li key={widget.id} className="w-fit">
              <DockerHeaderTabElement
                widget={widget}
                selected={selectedWidgetId === widget.id}
                onSelect={setSelectedWidgetId}
                onPopout={handlePopout}
                onClose={() => removeWidget(widget.id)}
              />
            </li>
          ))}
        </ul>
      </DockHeader>

      {!selectedWidgetMemo && (
        <div className={cn("w-full h-full flex items-center justify-center")}>
          Not Tab Selected
        </div>
      )}

      {selectedWidgetMemo && (
        <div
          className={cn(
            "w-full h-[calc(100%-24px)] flex items-center justify-center"
          )}
        >
          {/* {selectedWidgetMemo.id} */}
          {selectedWidgetMemo.category === EWidgetCategory.Terminal && (
            <TerminalWidget
              id={selectedWidgetMemo.id}
              data={selectedWidgetMemo.metadata}
              onClose={() => {
                removeWidget(selectedWidgetMemo.id);
              }}
            />
          )}
          {selectedWidgetMemo.category === EWidgetCategory.Editor && (
            <EditorWidget
              id={selectedWidgetMemo.id}
              data={selectedWidgetMemo.metadata}
            />
          )}
        </div>
      )}
    </div>
  );
}

function DockHeader(props: {
  position?: string;
  className?: string;
  onPositionChange?: (position: string) => void;
  children?: React.ReactNode;
  onClose?: () => void;
}) {
  const {
    position = "bottom",
    className,
    onPositionChange,
    children,
    onClose,
  } = props;

  return (
    <div
      className={cn(
        "w-full h-6 flex items-center justify-between",
        "px-4",
        "bg-border dark:bg-popover",
        className
      )}
    >
      {children}
      {/* Action Bar */}
      <div className="w-fit flex items-center gap-2">
        <DropdownMenu>
          <DropdownMenuTrigger>
            <EllipsisVerticalIcon className="w-4 h-4" />
          </DropdownMenuTrigger>
          <DropdownMenuContent>
            <DropdownMenuLabel>Dock Side</DropdownMenuLabel>
            <DropdownMenuSeparator />
            <DropdownMenuRadioGroup
              value={position}
              onValueChange={onPositionChange}
            >
              <DropdownMenuRadioItem value="left">
                <PanelLeftIcon className="w-4 h-4 me-2" />
                Left
              </DropdownMenuRadioItem>
              <DropdownMenuRadioItem value="right">
                <PanelRightIcon className="w-4 h-4 me-2" />
                Right
              </DropdownMenuRadioItem>
              <DropdownMenuRadioItem value="bottom">
                <PanelBottomIcon className="w-4 h-4 me-2" />
                Bottom
              </DropdownMenuRadioItem>
            </DropdownMenuRadioGroup>
          </DropdownMenuContent>
        </DropdownMenu>
        {onClose && (
          <XIcon
            className="w-4 h-4 cursor-pointer"
            type="button"
            tabIndex={1}
            onClick={onClose}
          />
        )}
      </div>
    </div>
  );
}

function DockerHeaderTabElement(props: {
  widget: IWidget;
  selected?: boolean;
  onClose?: (id: string) => void;
  onPopout?: (id: string) => void;
  onSelect?: (id: string) => void;
}) {
  const { widget, selected, onClose, onPopout, onSelect } = props;
  const category = widget.category;
  return (
    <ContextMenu>
      <ContextMenuTrigger>
        <div
          className={cn(
            "w-fit px-2 py-1 text-xs text-muted-foreground",
            "flex items-center gap-1 cursor-pointer",
            "border-b-2 border-transparent",
            {
              "text-primary": selected,
              "border-purple-900": selected,
            },
            "hover:text-primary hover:border-purple-950"
          )}
          onClick={() => onSelect?.(widget.id)}
        >
          {category}
          {onClose && (
            <XIcon className="size-3" onClick={() => onClose(widget.id)} />
          )}
        </div>
      </ContextMenuTrigger>
      <ContextMenuContent>
        <ContextMenuItem
          onClick={() => {
            onPopout?.(widget.id);
          }}
        >
          Popout
          <ContextMenuShortcut>
            <SquareArrowOutUpRightIcon className="size-3" />
          </ContextMenuShortcut>
        </ContextMenuItem>
        <ContextMenuSeparator />
        <ContextMenuItem onClick={() => onClose?.(widget.id)}>
          Close
        </ContextMenuItem>
      </ContextMenuContent>
    </ContextMenu>
  );
}
