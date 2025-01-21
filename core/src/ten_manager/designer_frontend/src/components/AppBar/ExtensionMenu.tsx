//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import * as React from "react";
import { BlocksIcon } from "lucide-react";
import { useTranslation } from "react-i18next";

import {
  NavigationMenuContent,
  NavigationMenuItem,
  NavigationMenuLink,
  NavigationMenuTrigger,
} from "@/components/ui/NavigationMenu";
import { Button } from "@/components/ui/Button";
import { cn } from "@/lib/utils";
import { useWidgetStore } from "@/store/widget";
import { EWidgetDisplayType, EWidgetCategory } from "@/types/widgets";

export function ExtensionMenu() {
  const { t } = useTranslation();
  const { appendWidgetIfNotExists } = useWidgetStore();

  const handleManageExtensionsClick = () => {
    appendWidgetIfNotExists({
      id: "extension-manager",
      category: EWidgetCategory.ExtensionManager,
      metadata: {},
      display_type: EWidgetDisplayType.Popup,
    });
  };

  return (
    <NavigationMenuItem>
      <NavigationMenuTrigger className="submenu-trigger">
        {t("header.menu.extensions")}
      </NavigationMenuTrigger>
      <NavigationMenuContent
        className={cn("flex flex-col items-center px-1 py-1.5 gap-1.5")}
      >
        <NavigationMenuLink asChild>
          <Button
            className="w-full justify-start max-w-screen-sm"
            variant="ghost"
            onClick={handleManageExtensionsClick}
          >
            <BlocksIcon />
            {t("header.menu.extensions.manage")}
          </Button>
        </NavigationMenuLink>
      </NavigationMenuContent>
    </NavigationMenuItem>
  );
}
