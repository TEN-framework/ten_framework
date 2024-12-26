//
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0.
// See the LICENSE file for more information.
//

import { assert } from 'console';

// import { Addon, RegisterAddonAsExtension, Extension, TenEnv, Cmd, Data, CmdResult, StatusCode } from '../../../../../../../../../../core/src/ten_runtime/binding/nodejs/interface'
import { Addon, RegisterAddonAsExtension, Extension, TenEnv, Cmd, Data, CmdResult, StatusCode } from 'ten-runtime-nodejs'

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
        await tenEnv.initPropertyFromJson(JSON.stringify({ key1: 'value1', key2: 2 }));
    }

    async onInit(tenEnv: TenEnv) {
        console.log('DefaultExtension onInit');

        const value1 = await tenEnv.getPropertyString('key1');
        const value2 = await tenEnv.getPropertyNumber('key2');

        console.log('value1:', value1);
        console.log('value2:', value2);

        assert(value1 === 'value1', 'value1 incorrect');
        assert(value2 === 2, 'value2 incorrect');
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

        const newData = Data.Create('data');

        const str = "Hello, World!";

        const encoder = new TextEncoder();
        const uint8Array = encoder.encode(str);

        newData.allocBuf(uint8Array.byteLength);
        let dataBuf = newData.lockBuf()
        let dataBufView = new Uint8Array(dataBuf);
        dataBufView.set(uint8Array);

        newData.unlockBuf(dataBuf);

        await tenEnv.sendData(newData);

        const data2 = Data.Create('data2');
        data2.setPropertyString('key1', 'value1');
        data2.setPropertyNumber('key2', 2);
        data2.setPropertyBool('key3', true);

        const valueBuf = new Uint8Array([1, 2, 3]);
        data2.setPropertyBuf('key4', valueBuf);

        await tenEnv.sendData(data2);

        const cmdResult = CmdResult.Create(StatusCode.OK);
        cmdResult.setPropertyString('detail', 'send data done')

        tenEnv.returnResult(cmdResult, cmd);
    }

    async onData(tenEnv: TenEnv, data: Data) {
        tenEnv.logDebug('DefaultExtension onCmd');

        if (data.getName() === 'data2') {
            const value1 = data.getPropertyString('key1');
            const value2 = data.getPropertyNumber('key2');
            const value3 = data.getPropertyBool('key3');
            const value4 = data.getPropertyBuf('key4');

            assert(value1 === 'value1', 'value1 incorrect');
            assert(value2 === 2, 'value2 incorrect');
            assert(value3 === true, 'value3 incorrect');

            const value4Uint8Array = new Uint8Array(value4);
            assert(value4Uint8Array[0] === 1, 'value4[0] incorrect');
            assert(value4Uint8Array[1] === 2, 'value4[1] incorrect');
            assert(value4Uint8Array[2] === 3, 'value4[2] incorrect');

            return;
        }

        const buf = data.getBuf();
        const bufView = new Uint8Array(buf);

        for (let i = 0; i < buf.byteLength; i++) {
            console.log('buf[' + i + ']:', bufView[i]);
            bufView[i] += 1;
        }

        let lockedBuf = data.lockBuf();
        let view = new String(lockedBuf);

        assert(view === 'Hello, World!', 'view incorrect');

        data.unlockBuf(lockedBuf);
    }
}