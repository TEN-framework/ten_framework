//
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0.
// See the LICENSE file for more information.
//

import { Addon, AddonManager, RegisterAddonAsExtension, Extension, TenEnv } from 'ten-runtime-nodejs';

console.log('Im a default ts extension');

@RegisterAddonAsExtension("default_extension_ts")
class DefaultAddon extends Addon {
    async onCreateInstance(tenEnv: TenEnv): Promise<Extension> {

    }
}