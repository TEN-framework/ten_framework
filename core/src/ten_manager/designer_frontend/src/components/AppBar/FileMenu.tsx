//
// Copyright © 2024 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import * as React from "react";
import { toast } from "sonner";
import { FolderOpenIcon } from "lucide-react";

import Popup from "@/components/Popup/Popup";
import {
  NavigationMenuContent,
  NavigationMenuItem,
  NavigationMenuLink,
  NavigationMenuTrigger,
} from "@/components/ui/NavigationMenu";
import { Button } from "@/components/ui/Button";
import { cn } from "@/lib/utils";

import { ThreeColumnFileManager } from "@/components/FileManager/AppFolder";

interface FileMenuProps {
  onSetBaseDir: (folderPath: string) => void;
}

export function FileMenu(props: FileMenuProps) {
  const { onSetBaseDir } = props;
  const [isFolderPathModalOpen, setIsFolderPathModalOpen] =
    React.useState<boolean>(false);
  const [folderPath, setFolderPath] = React.useState<string>("");

  const handleManualOk = async () => {
    if (!folderPath.trim()) {
      toast.error("The folder path cannot be empty.");
      return;
    }

    onSetBaseDir(folderPath.trim());
    setFolderPath("");
  };

  return (
    <>
      <NavigationMenuItem>
        <NavigationMenuTrigger className="submenu-trigger">
          File
        </NavigationMenuTrigger>
        <NavigationMenuContent
          className={cn("flex flex-col items-center px-1 py-1.5 gap-1.5")}
        >
          <NavigationMenuLink asChild>
            <Button
              className="w-full justify-start"
              variant="ghost"
              onClick={() => setIsFolderPathModalOpen(true)}
            >
              <FolderOpenIcon className="w-4 h-4 me-2" />
              Open app folder
            </Button>
          </NavigationMenuLink>
        </NavigationMenuContent>
      </NavigationMenuItem>
      {isFolderPathModalOpen && (
        <Popup
          title="Open App Folder"
          onClose={() => setIsFolderPathModalOpen(false)}
          resizable={false}
          initialWidth={600}
          initialHeight={400}
          onCollapseToggle={() => {}}
        >
          <div className="flex flex-col gap-2 w-full h-full">
            <ThreeColumnFileManager
              allowSelectTypes={["folder"]}
              className="w-full h-[calc(100%-3rem)]"
              onSelect={(path) => setFolderPath(path)}
            />
            <div className="flex justify-end h-fit gap-2">
              <Button
                variant="outline"
                onClick={() => setIsFolderPathModalOpen(false)}
              >
                Cancel
              </Button>
              <Button onClick={handleManualOk}>Save</Button>
            </div>
          </div>
        </Popup>
      )}
    </>
  );
}
