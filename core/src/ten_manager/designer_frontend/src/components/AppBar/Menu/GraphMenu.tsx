//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import { useTranslation } from "react-i18next";
import { FolderOpenIcon, MoveIcon, PackagePlusIcon } from "lucide-react";

import {
  NavigationMenuContent,
  NavigationMenuItem,
  NavigationMenuLink,
  NavigationMenuTrigger,
} from "@/components/ui/NavigationMenu";
import { Separator } from "@/components/ui/Separator";
import { Button } from "@/components/ui/Button";
import { cn } from "@/lib/utils";
import { useWidgetStore, useAppStore } from "@/store";
import { GRAPH_SELECT_POPUP_ID } from "@/constants/widgets";
import {
  EDefaultWidgetType,
  EWidgetCategory,
  EWidgetDisplayType,
} from "@/types/widgets";
import { ENodeActions } from "@/types/graphs";

export function GraphMenu(props: {
  onAutoLayout: () => void;
  disableMenuClick?: boolean;
  idx: number;
  triggerListRef?: React.RefObject<HTMLButtonElement[]>;
}) {
  const { onAutoLayout, disableMenuClick, idx, triggerListRef } = props;

  const { t } = useTranslation();
  const { appendWidgetIfNotExists } = useWidgetStore();
  const { currentWorkspace } = useAppStore();

  const onOpenExistingGraph = () => {
    appendWidgetIfNotExists({
      id: GRAPH_SELECT_POPUP_ID,
      category: EWidgetCategory.Default,
      display_type: EWidgetDisplayType.Popup,
      metadata: {
        type: EDefaultWidgetType.GraphSelect,
      },
    });
  };

  const onAddNode = () => {
    if (!currentWorkspace?.baseDir) return;
    appendWidgetIfNotExists({
      id:
        `node-add-popup-` +
        `${currentWorkspace.baseDir}-${currentWorkspace?.graphName}`,
      category: EWidgetCategory.Node,
      display_type: EWidgetDisplayType.Popup,
      metadata: {
        type: ENodeActions.ADD,
        base_dir: currentWorkspace.baseDir,
        graph_name: currentWorkspace?.graphName,
      },
    });
  };

  return (
    <NavigationMenuItem>
      <NavigationMenuTrigger
        className="submenu-trigger"
        ref={(ref) => {
          if (triggerListRef?.current && ref) {
            triggerListRef.current[idx] = ref;
          }
        }}
        onClick={(e) => {
          if (disableMenuClick) {
            e.preventDefault();
          }
        }}
      >
        {t("header.menuGraph.title")}
      </NavigationMenuTrigger>
      <NavigationMenuContent
        className={cn("flex flex-col items-center px-1 py-1.5 gap-1.5")}
      >
        <NavigationMenuLink asChild>
          <Button
            className="w-full justify-start"
            variant="ghost"
            onClick={onOpenExistingGraph}
          >
            <FolderOpenIcon />
            {t("header.menuGraph.loadGraph")}
          </Button>
        </NavigationMenuLink>
        <NavigationMenuLink asChild>
          <Button
            className="w-full justify-start"
            variant="ghost"
            onClick={onAutoLayout}
          >
            <MoveIcon />
            {t("header.menuGraph.autoLayout")}
          </Button>
        </NavigationMenuLink>
        <Separator className="w-full" />
        <NavigationMenuLink asChild>
          <Button
            className="w-full justify-start"
            variant="ghost"
            disabled={!currentWorkspace || !currentWorkspace.baseDir}
            onClick={onAddNode}
          >
            <PackagePlusIcon />
            {t("header.menuGraph.addNode")}
          </Button>
        </NavigationMenuLink>
      </NavigationMenuContent>
    </NavigationMenuItem>
  );
}
