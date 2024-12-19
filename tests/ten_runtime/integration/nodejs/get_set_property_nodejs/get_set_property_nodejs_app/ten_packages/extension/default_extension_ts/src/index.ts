//
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0.
// See the LICENSE file for more information.
//

import { Addon, AddonManager, RegisterAddonAsExtension, Extension, TenEnv } from 'ten-runtime-nodejs';

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

        const aaaExist = await tenEnv.isPropertyExist('aaa');
        const bbbExist = await tenEnv.isPropertyExist('bbb');

        console.log('aaaExist', aaaExist);
        console.log('bbbExist', bbbExist);

        if (!aaaExist || bbbExist) {
            // Exit with error
            console.error('Property aaa does not exist or property bbb exists');
            process.exit(1);
        }
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
}