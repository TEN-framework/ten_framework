//
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0.
// See the LICENSE file for more information.
//

import { assert } from 'console';
import { Addon, AddonManager, RegisterAddonAsExtension, Extension, TenEnv, Cmd, Data, CmdResult, StatusCode } from 'ten-runtime-nodejs';

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

        const testData = Data.Create('testData');
        testData.allocBuf(10);
        let buf = testData.lockBuf();

        let view = new Uint8Array(buf);
        view[0] = 1;
        view[1] = 2;
        view[2] = 3;
        testData.unlockBuf(buf);

        let copiedBuf = testData.getBuf();
        let copiedView = new Uint8Array(copiedBuf);
        assert(copiedView[0] === 1, 'copiedView[0] incorrect');
        assert(copiedView[1] === 2, 'copiedView[1] incorrect');
        assert(copiedView[2] === 3, 'copiedView[2] incorrect');
    }

    async onStop(tenEnv: TenEnv) {
        console.log('DefaultExtension onStop');
    }

    async onDeinit(tenEnv: TenEnv) {
        console.log('DefaultExtension onDeinit');
    }

    async onCmd(tenEnv: TenEnv, cmd: Cmd) {
        tenEnv.logDebug('DefaultExtension onCmd');

        const cmdName = cmd.getName();
        tenEnv.logVerbose('cmdName:' + cmdName);

        const testCmd = Cmd.Create('test');
        const result = await tenEnv.sendCmd(testCmd);
        tenEnv.logInfo('received result detail:' + result.getPropertyToJson('detail'));

        const cmdResult = CmdResult.Create(StatusCode.OK);
        cmdResult.setPropertyFromJson('detail', JSON.stringify({ key1: 'value1', key2: 2 }))

        const detailJson = cmdResult.getPropertyToJson('detail');
        tenEnv.logInfo('detailJson:' + detailJson);

        tenEnv.returnResult(cmdResult, cmd);
    }
}