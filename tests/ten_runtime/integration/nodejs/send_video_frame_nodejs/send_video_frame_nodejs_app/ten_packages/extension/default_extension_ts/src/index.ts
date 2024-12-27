//
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0.
// See the LICENSE file for more information.
//

// import { Addon, RegisterAddonAsExtension, Extension, TenEnv, Cmd, CmdResult, StatusCode, VideoFrame, PixelFmt } from '../../../../../../../../../../core/src/ten_runtime/binding/nodejs/interface'
import { Addon, RegisterAddonAsExtension, Extension, TenEnv, Cmd, CmdResult, StatusCode, VideoFrame, PixelFmt } from 'ten-runtime-nodejs'

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

function assert(condition: boolean, message: string) {
    if (!condition) {
        throw new Error(message);
    }
}

class DefaultExtension extends Extension {
    // Cache the received cmd
    cachedCmd: Cmd | null = null;

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
        const cmdName = cmd.getName();
        tenEnv.logDebug('DefaultExtension onCmd ' + cmdName);

        this.cachedCmd = cmd;

        let videoFrame = VideoFrame.Create("video_frame");
        videoFrame.setWidth(320);
        videoFrame.setHeight(240);
        videoFrame.setPixelFmt(PixelFmt.RGBA);
        videoFrame.setEof(false);

        const now = Date.now();
        videoFrame.setTimestamp(now);

        videoFrame.allocBuf(320 * 240 * 4);
        let buf = videoFrame.lockBuf();
        const bufView = new Uint8Array(buf);
        for (let i = 0; i < 320 * 240 * 4; i++) {
            bufView[i] = i % 256;
        }

        videoFrame.unlockBuf(buf);

        await tenEnv.sendVideoFrame(videoFrame);
    }

    async onVideoFrame(tenEnv: TenEnv, frame: VideoFrame): Promise<void> {
        tenEnv.logDebug('DefaultExtension onVideoFrame');

        assert(frame.getPixelFmt() === PixelFmt.RGBA, 'Pixel format is not RGBA');
        assert(frame.getWidth() === 320, 'Width is not 320');
        assert(frame.getHeight() === 240, 'Height is not 240');
        assert(frame.isEof() === false, 'Eof is not false');

        const timestamp = frame.getTimestamp();
        assert(timestamp !== 0, 'Timestamp is 0');
        const now = Date.now();
        assert(timestamp <= now, 'Timestamp is not less than or equal to now');

        const buf = frame.getBuf();
        const bufView = new Uint8Array(buf);
        for (let i = 0; i < 320 * 240 * 4; i++) {
            assert(bufView[i] === i % 256, 'Incorrect pixel value');
        }

        const cmd = this.cachedCmd;
        assert(cmd !== null, 'Cached cmd is null');

        const result = CmdResult.Create(StatusCode.OK);
        result.setPropertyString('detail', "success")
        await tenEnv.returnResult(result, cmd!);
    }
}