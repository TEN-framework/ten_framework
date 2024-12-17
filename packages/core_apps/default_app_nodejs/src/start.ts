//
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0.
// See the LICENSE file for more information.
//

import { App, TenEnv } from 'ten-runtime-nodejs';

console.log('Hello, World!');

class DefaultApp extends App {
    async onConfigure(tenEnv: TenEnv): Promise<void> {
        console.log('Default App onConfigure');
    }

    async onInit(tenEnv: TenEnv): Promise<void> {
        console.log('Default App onInit');
    }

    async onDeinit(tenEnv: TenEnv): Promise<void> {
        console.log('Default App onDeinit');
    }
}

const app = new DefaultApp();
app.run();