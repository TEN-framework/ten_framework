//
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0.
// See the LICENSE file for more information.
//

// import { Addon, RegisterAddonAsExtension, Extension, TenEnv, Cmd, Data, CmdResult, StatusCode, AudioFrame, AudioFrameDataFmt } from '../../../../../../../../../../core/src/ten_runtime/binding/nodejs/interface'
import { Addon, RegisterAddonAsExtension, Extension, TenEnv, Cmd, Data, CmdResult, StatusCode, AudioFrame, AudioFrameDataFmt } from 'ten-runtime-nodejs'

@RegisterAddonAsExtension("default_extension_nodejs")
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

        const audioFrame = AudioFrame.Create('audio_frame');
        audioFrame.setDataFmt(AudioFrameDataFmt.INTERLEAVE);
        audioFrame.setBytesPerSample(2);
        audioFrame.setSampleRate(16000);
        audioFrame.setNumberOfChannels(1);
        audioFrame.setSamplesPerChannel(160);

        const timestamp = Date.now();
        audioFrame.setTimestamp(timestamp);

        audioFrame.setEof(false);
        audioFrame.setLineSize(320);

        audioFrame.allocBuf(320);
        let buf = audioFrame.lockBuf();
        const bufView = new Uint8Array(buf);
        for (let i = 0; i < 320; i++) {
            bufView[i] = i % 256;
        }
        audioFrame.unlockBuf(buf);

        await tenEnv.sendAudioFrame(audioFrame);
    }

    async onAudioFrame(tenEnv: TenEnv, frame: AudioFrame): Promise<void> {
        tenEnv.logDebug('DefaultExtension onAudioFrame');

        assert(frame.getDataFmt() === AudioFrameDataFmt.INTERLEAVE, 'DataFmt is not INTERLEAVE');
        assert(frame.getBytesPerSample() === 2, 'BytesPerSample is not 2');
        assert(frame.getSampleRate() === 16000, 'SampleRate is not 16000');
        assert(frame.getNumberOfChannels() === 1, 'NumberOfChannels is not 1');
        assert(frame.getSamplesPerChannel() === 160, 'SamplesPerChannel is not 160');

        const timestamp = frame.getTimestamp();
        assert(timestamp > 0, 'Timestamp is not greater than 0');
        const now = Date.now();
        assert(timestamp <= now, 'Timestamp is not less than or equal to now');

        assert(frame.isEof() === false, 'isEof is not false');
        assert(frame.getLineSize() === 320, 'LineSize is not 320');

        const buf = frame.getBuf();
        const bufView = new Uint8Array(buf);
        for (let i = 0; i < 320; i++) {
            assert(bufView[i] === i % 256, 'Data is not correct');
        }

        const cmd = this.cachedCmd;
        assert(cmd !== null, 'Cached cmd is null');

        const result = CmdResult.Create(StatusCode.OK);
        result.setPropertyString('detail', "success")
        await tenEnv.returnResult(result, cmd!);
    }
}