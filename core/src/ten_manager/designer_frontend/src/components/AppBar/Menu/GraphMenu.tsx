//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import { useTranslation } from "react-i18next";
import { FolderOpenIcon, MoveIcon, BlocksIcon } from "lucide-react";

import {
  NavigationMenuContent,
  NavigationMenuItem,
  NavigationMenuLink,
  NavigationMenuTrigger,
} from "@/components/ui/NavigationMenu";
import { Button } from "@/components/ui/Button";
import { cn } from "@/lib/utils";
import { useWidgetStore } from "@/store/widget";
import {
  GRAPH_SELECT_POPUP_ID,
  EXTENSION_STORE_POPUP_ID,
} from "@/constants/widgets";
import {
  EDefaultWidgetType,
  EWidgetCategory,
  EWidgetDisplayType,
} from "@/types/widgets";

export function GraphMenu(props: { onAutoLayout: () => void }) {
  const { onAutoLayout } = props;

  const { t } = useTranslation();
  const { appendWidgetIfNotExists } = useWidgetStore();

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

  const onOpenExtensionStore = () => {
    appendWidgetIfNotExists({
      id: EXTENSION_STORE_POPUP_ID,
      category: EWidgetCategory.Default,
      display_type: EWidgetDisplayType.Popup,
      metadata: {
        type: EDefaultWidgetType.ExtensionStore,
      },
    });
  };

  return (
    <NavigationMenuItem>
      <NavigationMenuTrigger className="submenu-trigger">
        {t("header.menu.graph")}
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
            {t("header.menu.openExistingGraph")}
          </Button>
        </NavigationMenuLink>
        <NavigationMenuLink asChild>
          <Button
            className="w-full justify-start"
            variant="ghost"
            onClick={onOpenExtensionStore}
          >
            <BlocksIcon />
            {t("header.menu.openExtensionStore")}
          </Button>
        </NavigationMenuLink>
        <NavigationMenuLink asChild>
          <Button
            className="w-full justify-start"
            variant="ghost"
            onClick={onAutoLayout}
          >
            <MoveIcon />
            {t("header.menu.autoLayout")}
          </Button>
        </NavigationMenuLink>
      </NavigationMenuContent>
    </NavigationMenuItem>
  );
}
