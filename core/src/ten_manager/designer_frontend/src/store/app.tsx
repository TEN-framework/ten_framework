//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import { create } from "zustand";
import { devtools } from "zustand/middleware";

import { TEN_DEFAULT_APP_RUN_SCRIPT } from "@/constants";

import { type IFMItem } from "@/components/FileManager/utils";
import { type IExtensionAddon } from "@/types/addons";

export interface IAppStore {
  runScript: string;
  setRunScript: (runScript: string) => void;
  folderPath: string;
  setFolderPath: (folderPath: string) => void;
  fmItems: IFMItem[][];
  setFmItems: (fmItems: IFMItem[][]) => void;
  addons: IExtensionAddon[];
  setAddons: (addons: IExtensionAddon[]) => void;
  defaultOsArch: {
    os?: string;
    arch?: string;
  };
  setDefaultOsArch: (osArch: { os?: string; arch?: string }) => void;
}

export const useAppStore = create<IAppStore>()(
  devtools((set) => ({
    runScript: TEN_DEFAULT_APP_RUN_SCRIPT,
    setRunScript: (runScript: string) => set({ runScript }),
    folderPath: "/",
    setFolderPath: (folderPath: string) => set({ folderPath }),
    fmItems: [[]],
    setFmItems: (fmItems: IFMItem[][]) => set({ fmItems }),
    addons: [],
    setAddons: (addons: IExtensionAddon[]) => set({ addons }),
    defaultOsArch: {
      os: undefined,
      arch: undefined,
    },
    setDefaultOsArch: (osArch: { os?: string; arch?: string }) =>
      set({ defaultOsArch: osArch }),
  }))
);
