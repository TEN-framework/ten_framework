//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import { Extension } from "../extension/extension";
import ten_addon from "../ten_addon";
import { TenEnv } from "../ten_env/ten_env";

export abstract class Addon {
  constructor() {
    ten_addon.ten_nodejs_addon_create(this);
  }

  private async onInitProxy(tenEnv: TenEnv): Promise<void> {
    try {
      await this.onInit(tenEnv);
    } catch (error) {
      // TODO: What should we do in this situation?
    } finally {
      ten_addon.ten_nodejs_ten_env_on_init_done(tenEnv);
    }
  }

  private async onDeinitProxy(tenEnv: TenEnv): Promise<void> {
    try {
      await this.onDeinit(tenEnv);
    } catch (error) {
      // TODO: What should we do in this situation?
    } finally {
      ten_addon.ten_nodejs_ten_env_on_deinit_done(tenEnv);

      // JS addon prepare to be destroyed, so notify the underlying C runtime this
      // fact.
      ten_addon.ten_nodejs_addon_on_end_of_life(this);
    }
  }

  private async onCreateInstanceProxy(
    tenEnv: TenEnv,
    instanceName: string,
    context: any
  ): Promise<void> {
    const extension = await this.onCreateInstance(tenEnv, instanceName);

    ten_addon.ten_nodejs_ten_env_on_create_instance_done(
      tenEnv,
      extension,
      context
    );
  }

  async onInit(tenEnv: TenEnv): Promise<void> {
    // Stub for override.
  }

  async onDeinit(tenEnv: TenEnv): Promise<void> {
    // Stub for override.
  }

  abstract onCreateInstance(
    tenEnv: TenEnv,
    instanceName: string
  ): Promise<Extension>;
}
