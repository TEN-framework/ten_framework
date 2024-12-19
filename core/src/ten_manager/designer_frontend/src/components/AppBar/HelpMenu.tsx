//
// Copyright © 2024 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import * as React from "react";

import AboutPopup from "@/components/Popup/AboutPopup";
import {
  NavigationMenuContent,
  NavigationMenuItem,
  NavigationMenuLink,
  NavigationMenuTrigger,
} from "@/components/ui/navigation-menu";
import { Button } from "@/components/ui/button";
import { InfoIcon } from "lucide-react";

export function HelpMenu() {
  const [isAboutOpen, setIsAboutOpen] = React.useState(false);

  const openAbout = () => {
    setIsAboutOpen(true);
  };

  const closeAbout = () => {
    setIsAboutOpen(false);
  };

  return (
    <>
      <NavigationMenuItem>
        <NavigationMenuTrigger className="submenu-trigger">
          Help
        </NavigationMenuTrigger>
        <NavigationMenuContent className="flex flex-col items-center px-1 py-1.5 gap-1.5">
          <NavigationMenuLink asChild>
            <Button
              className="w-full justify-start max-w-screen-sm"
              variant="ghost"
              onClick={openAbout}
            >
              <InfoIcon />
              About
            </Button>
          </NavigationMenuLink>
        </NavigationMenuContent>
      </NavigationMenuItem>

      {isAboutOpen && <AboutPopup onClose={closeAbout} />}
    </>
  );
}
