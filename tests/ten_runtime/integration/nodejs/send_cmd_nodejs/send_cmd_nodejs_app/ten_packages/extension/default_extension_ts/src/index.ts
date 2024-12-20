//
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0.
// See the LICENSE file for more information.
//

import { Addon, AddonManager, RegisterAddonAsExtension, Extension, TenEnv, Cmd, CmdResult, StatusCode } from 'ten-runtime-nodejs';

console.log('Im a default ts extension');

@RegisterAddonAsExtension("default_extension_ts")
class DefaultAddon extends Addon {
    async onInit() {
        console.log('DefaultAddon onInit');
    }

    async onCreateInstance(tenEnv: TenEnv, instanceName: string): Promise<Extension> {
        return new DefaultExtension(instanceName);
    }

    async onDeinit() {
        console.log('DefaultAddon onDeinit');
    }
}

class DefaultExtension extends Extension {
    constructor(name: string) {
        super(name);
    }

    async onConfigure(tenEnv: TenEnv) {
        console.log('DefaultExtension onConfigure');
    }

    async onInit(tenEnv: TenEnv) {
        console.log('DefaultExtension onInit');
    }

    async onStart(tenEnv: TenEnv) {
        console.log('DefaultExtension onStart');
    }

    async onStop(tenEnv: TenEnv) {
        console.log('DefaultExtension onStop');
    }

    async onDeinit(tenEnv: TenEnv) {
        console.log('DefaultExtension onDeinit');
    }

    async onCmd(tenEnv: TenEnv, cmd: Cmd) {
        console.log('DefaultExtension onCmd');

        const cmdName = cmd.getName();
        console.log('cmdName:', cmdName);

        const cmdResult = CmdResult.Create(StatusCode.OK);
        cmdResult.setPropertyFromJson('detail', JSON.stringify({ key1: 'value1', key2: 2 }))

        const detailJson = cmdResult.getPropertyToJson('detail');
        console.log('detailJson:', detailJson);

        tenEnv.returnResult(cmdResult, cmd);
    }
}