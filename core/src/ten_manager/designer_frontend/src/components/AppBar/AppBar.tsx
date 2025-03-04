//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import * as React from "react";
import { useTranslation, Trans } from "react-i18next";

import {
  NavigationMenu,
  NavigationMenuList,
} from "@/components/ui/NavigationMenu";
import { ModeToggle } from "@/components/ModeToggle";
import { LanguageToggle } from "@/components/LangSwitch";
import { Badge } from "@/components/ui/Badge";
import { AppMenu } from "@/components/AppBar/Menu/AppMenu";
import { EditMenu } from "@/components/AppBar/Menu/EditMenu";
import { HelpMenu } from "@/components/AppBar/Menu/HelpMenu";
import { AppStatus } from "@/components/AppBar/AppStatus";
import { cn } from "@/lib/utils";

interface AppBarProps {
  // The current version of tman.
  version?: string;

  onOpenExistingGraph: () => void;
  onAutoLayout: () => void;
  onSetBaseDir: (folderPath: string) => void;
}

const AppBar: React.FC<AppBarProps> = ({
  version,
  onOpenExistingGraph,
  onAutoLayout,
  onSetBaseDir,
}) => {
  const { t } = useTranslation();

  const onNavChange = () => {
    setTimeout(() => {
      const triggers = document.querySelectorAll(
        '.submenu-trigger[data-state="open"]'
      );
      if (triggers.length === 0) return;

      const firstTrigger = triggers[0] as HTMLElement;

      document.documentElement.style.setProperty(
        "--menu-left-position",
        `${firstTrigger.offsetLeft}px`
      );
    });
  };

  return (
    <div
      className={cn(
        "flex justify-between items-center h-10 px-5 text-sm select-none",
        "bg-[var(--app-bar-bg)] text-[var(--app-bar-fg)]",
        "border-b border-[#e5e7eb] dark:border-[#374151]"
      )}
    >
      <NavigationMenu onValueChange={onNavChange}>
        <NavigationMenuList>
          <AppMenu onSetBaseDir={onSetBaseDir} />
          <EditMenu
            onAutoLayout={onAutoLayout}
            onOpenExistingGraph={onOpenExistingGraph}
          />
          <HelpMenu />
        </NavigationMenuList>
      </NavigationMenu>

      {/* Middle part is the status bar. */}
      <AppStatus
        className={cn(
          "flex-1 flex justify-center items-center",
          "text-xs text-muted-foreground"
        )}
      />

      {/* Right part is the logo. */}
      <div className="flex items-center gap-1.5">
        <LanguageToggle />
        <ModeToggle />
        <div
          className={cn(
            "text-xs text-muted-foreground flex items-center gap-2 relative"
          )}
        >
          <div>
            <Trans
              components={[
                <PoweredByTenFramework className="font-bold text-foreground" />,
              ]}
              t={t}
              i18nKey="header.poweredByTenFramework"
            />
          </div>
          <Badge variant="secondary">{version}</Badge>
        </div>
      </div>
    </div>
  );
};

export function PoweredByTenFramework(props: {
  className?: string;
  children?: React.ReactNode;
}) {
  const { t } = useTranslation();
  return (
    <span className={cn("font-bold text-foreground", props.className)}>
      {t("tenFramework")}
    </span>
  );
}

export default AppBar;
