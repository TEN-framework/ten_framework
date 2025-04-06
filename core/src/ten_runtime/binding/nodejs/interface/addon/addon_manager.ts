//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import * as path from "path";
import * as fs from "fs";

import { Addon } from "./addon.js";
import ten_addon from "../ten_addon.js";
import { dirname } from "path";
import { fileURLToPath } from "url";

type Ctor<T> = {
  new (): T;
  prototype: T;
};

type addonRegisterHandler = (registerContext: unknown) => void;

export class AddonManager {
  private static _registry: Map<string, addonRegisterHandler> = new Map();

  static _set_register_handler(
    name: string,
    handler: addonRegisterHandler,
  ): void {
    AddonManager._registry.set(name, handler);
  }

  static _find_app_base_dir(): string {
    let currentDir = dirname(fileURLToPath(import.meta.url));

    while (currentDir !== path.dirname(currentDir)) {
      const manifestPath = path.join(currentDir, "manifest.json");
      if (fs.existsSync(manifestPath)) {
        try {
          const manifestJson = JSON.parse(
            fs.readFileSync(manifestPath, "utf-8"),
          );
          if (manifestJson.type === "app") {
            return currentDir;
          }
          // eslint-disable-next-line @typescript-eslint/no-unused-vars
        } catch (error) {
          // Ignore
        }
      }

      currentDir = path.dirname(currentDir);
    }

    throw new Error("Cannot find app base dir");
  }

  static async _load_all_addons(): Promise<void> {
    const app_base_dir = AddonManager._find_app_base_dir();

    const manifest_path = path.join(app_base_dir, "manifest.json");
    if (!fs.existsSync(manifest_path)) {
      throw new Error("Cannot find manifest.json");
    }

    const manifest = JSON.parse(fs.readFileSync(manifest_path, "utf-8"));

    const extension_names = [];

    const dependencies = manifest.dependencies;
    for (const dep of dependencies) {
      if (dep.type === "extension") {
        extension_names.push(dep.name);
      }
    }

    const extension_folder = path.join(app_base_dir, "ten_packages/extension");
    if (!fs.existsSync(extension_folder)) {
      return;
    }

    const dirs = fs.opendirSync(extension_folder);
    const loadPromises = [];

    for (;;) {
      const entry = dirs.readSync();
      if (!entry) {
        break;
      }

      if (entry.name.startsWith(".")) {
        continue;
      }

      const packageJsonFile = `${extension_folder}/${entry.name}/package.json`;

      if (entry.isDirectory() && fs.existsSync(packageJsonFile)) {
        // Log the extension name.
        console.log(`_load_all_addons Loading extension ${entry.name}`);
        loadPromises.push(
          import(`${extension_folder}/${entry.name}/build/index.js`),
        );
      }
    }

    // Wait for all modules to be loaded.
    await Promise.all(loadPromises);
    console.log(`_load_all_addons loaded ${loadPromises.length} extensions`);
  }

  static async _load_single_addon(name: string): Promise<boolean> {
    const app_base_dir = AddonManager._find_app_base_dir();

    const extension_folder = path.join(
      app_base_dir,
      "ten_packages/extension",
      name,
    );
    if (!fs.existsSync(extension_folder)) {
      console.log(`Addon ${name} directory not found in ${extension_folder}`);
      return false;
    }

    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    const dirs = fs.opendirSync(extension_folder);
    const packageJsonFile = `${extension_folder}/package.json`;
    if (!fs.existsSync(packageJsonFile)) {
      console.log(
        `Addon ${name} package.json not found in ${extension_folder}`,
      );
      return false;
    }

    try {
      await import(`${extension_folder}/build/index.js`);
      console.log(`Addon ${name} loaded`);
      return true;
    } catch (error) {
      console.error(`Failed to load addon ${name}: ${error}`);
      return false;
    }
  }

  static _register_single_addon(name: string, registerContext: unknown): void {
    const handler = AddonManager._registry.get(name);
    if (handler) {
      try {
        handler(registerContext);

        console.log(`Addon ${name} registered`);
      } catch (error) {
        console.error(`Failed to register addon ${name}: ${error}`);
      }
    } else {
      console.log(`Failed to find the register handler for addon ${name}`);
    }
  }

  static _register_all_addons(registerContext: unknown): void {
    for (const [name, handler] of AddonManager._registry) {
      try {
        handler(registerContext);

        console.log(`Addon ${name} registered`);
      } catch (error) {
        console.error(`Failed to register addon ${name}: ${error}`);
      }
    }

    AddonManager._registry.clear();
  }
}

export function RegisterAddonAsExtension(
  name: string,
): <T extends Ctor<Addon>>(klass: T) => T {
  return function <T extends Ctor<Addon>>(klass: T): T {
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    function registerHandler(registerContext: unknown) {
      const addon_instance = new klass();

      ten_addon.ten_nodejs_addon_manager_register_addon_as_extension(
        name,
        addon_instance,
      );
    }

    AddonManager._set_register_handler(name, registerHandler);

    console.log(`RegisterAddonAsExtension ${name} registered`);

    return klass;
  };
}
