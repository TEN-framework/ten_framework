//
// Copyright © 2024 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//

import { Extension } from "../extension/extension";
import ten_addon from "../ten_addon";
import { TenEnv } from "../ten_env/ten_env";

export abstract class Addon {

    private async onInitProxy(tenEnv: TenEnv): Promise<void> {
        try {
            await this.onInit(tenEnv);
        } catch (error) {
            // tenEnv.log
        } finally {
            ten_addon.ten_nodejs_ten_env_on_init_done(tenEnv);
        }
    }

    private async onDeinitProxy(tenEnv: TenEnv): Promise<void> {
        try {
            await this.onDeinit(tenEnv);
        } catch (error) {
            // tenEnv.log
        } finally {
            ten_addon.ten_nodejs_ten_env_on_deinit_done(tenEnv);
        }
    }

    private async onCreateInstanceProxy(tenEnv: TenEnv): Promise<void> {
        try {
            const extension = await this.onCreateInstance(tenEnv);
            ten_addon.ten_nodejs_ten_env_on_create_instance_done(tenEnv, extension);
        } catch (error) {
            // exit process
        }
    }


    async onInit(tenEnv: TenEnv): Promise<void> {
        // Stub for override.
    }

    async onDeinit(tenEnv: TenEnv): Promise<void> {
        // Stub for override.
    }

    abstract onCreateInstance(tenEnv: TenEnv): Promise<Extension>;
}