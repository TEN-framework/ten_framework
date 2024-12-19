//
// Copyright © 2024 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import * as React from "react";
import {
  NavigationMenu,
  NavigationMenuList,
} from "@/components/ui/navigation-menu";
import { ModeToggle } from "@/components/mode-toggle";
import { LanguageToggle } from "@/components/lang-switch";
import { Badge } from "@/components/ui/badge";

import { FileMenu } from "@/components/AppBar/FileMenu";
import { EditMenu } from "@/components/AppBar/EditMenu";
import { HelpMenu } from "@/components/AppBar/HelpMenu";

interface AppBarProps {
  // The current version of tman.
  version: string;

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
  const onNavChange = () => {
    setTimeout(() => {
      const triggers = document.querySelectorAll(
        '.submenu-trigger[data-state="open"]',
      );
      if (triggers.length === 0) return;

      const firstTrigger = triggers[0] as HTMLElement;

      document.documentElement.style.setProperty(
        "--menu-left-position",
        `${firstTrigger.offsetLeft}px`,
      );
    });
  };

  return (
    <div className="flex justify-between items-center bg-[var(--app-bar-bg)] text-[var(--app-bar-fg)] h-10 px-5 text-sm select-none">
      <NavigationMenu onValueChange={onNavChange}>
        <NavigationMenuList>
          <FileMenu onSetBaseDir={onSetBaseDir} />
          <EditMenu
            onAutoLayout={onAutoLayout}
            onOpenExistingGraph={onOpenExistingGraph}
          />
          <HelpMenu />
        </NavigationMenuList>
      </NavigationMenu>

      {/* Right part is the logo. */}
      <div className="ml-auto flex items-center gap-1.5">
        <LanguageToggle />
        <ModeToggle />
        <div className="text-xs text-muted-foreground flex items-center gap-2 relative">
          <div>
            Powered by{" "}
            <span className="font-bold text-foreground">TEN Framework</span>
          </div>
          <Badge variant="secondary">{version}</Badge>
        </div>
      </div>
    </div>
  );
};

export default AppBar;
