//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import { InfoIcon, SettingsIcon } from "lucide-react";
import { useTranslation } from "react-i18next";

import {
  NavigationMenuContent,
  NavigationMenuItem,
  NavigationMenuLink,
  NavigationMenuTrigger,
} from "@/components/ui/NavigationMenu";
import { Separator } from "@/components/ui/Separator";
import { Button } from "@/components/ui/Button";
import { cn } from "@/lib/utils";
import { useWidgetStore } from "@/store/widget";
import {
  EDefaultWidgetType,
  EWidgetCategory,
  EWidgetDisplayType,
} from "@/types/widgets";
import { ABOUT_POPUP_ID, PREFERENCES_POPUP_ID } from "@/constants/widgets";

export function DesignerMenu() {
  const { t } = useTranslation();

  const { appendWidgetIfNotExists } = useWidgetStore();

  const openAbout = () => {
    appendWidgetIfNotExists({
      id: ABOUT_POPUP_ID,
      category: EWidgetCategory.Default,
      display_type: EWidgetDisplayType.Popup,
      metadata: {
        type: EDefaultWidgetType.About,
      },
    });
  };

  const openPreferences = () => {
    appendWidgetIfNotExists({
      id: PREFERENCES_POPUP_ID,
      category: EWidgetCategory.Default,
      display_type: EWidgetDisplayType.Popup,
      metadata: {
        type: EDefaultWidgetType.Preferences,
      },
    });
  };

  return (
    <>
      <NavigationMenuItem>
        <NavigationMenuTrigger className="submenu-trigger font-bold">
          {t("header.menuDesigner.title")}
        </NavigationMenuTrigger>
        <NavigationMenuContent
          className={cn("flex flex-col items-center px-1 py-1.5 gap-1.5")}
        >
          <NavigationMenuLink asChild>
            <Button
              className="w-full justify-start max-w-(--breakpoint-sm)"
              variant="ghost"
              onClick={openAbout}
            >
              <InfoIcon />
              {t("header.menuDesigner.about")}
            </Button>
          </NavigationMenuLink>
          <Separator className="w-full" />
          <NavigationMenuLink asChild>
            <Button
              className="w-full justify-start max-w-(--breakpoint-sm)"
              variant="ghost"
              onClick={openPreferences}
            >
              <SettingsIcon />
              {t("header.menuDesigner.preferences")}
            </Button>
          </NavigationMenuLink>
        </NavigationMenuContent>
      </NavigationMenuItem>
    </>
  );
}
