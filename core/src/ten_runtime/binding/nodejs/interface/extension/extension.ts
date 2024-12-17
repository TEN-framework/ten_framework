//
// Copyright © 2024 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import { Cmd } from '../msg/cmd';
import { Data } from '../msg/data';
import { AudioFrame } from '../msg/audio_frame';
import { VideoFrame } from '../msg/video_frame';
import ten_addon from '../ten_addon'
import { TenEnv } from '../ten_env/ten_env';

export abstract class Extension {
    constructor(name: string) {
        ten_addon.ten_nodejs_extension_create(this, name);
    }

    private async onConfigureProxy(
        tenEnv: TenEnv
    ): Promise<void> {
        try {
            await this.onConfigure(tenEnv);
        } catch (e) {
            // log
        } finally {
            ten_addon.ten_nodejs_ten_env_on_configure_done(tenEnv);
        }
    }

    private async onInitProxy(
        tenEnv: TenEnv
    ): Promise<void> {
        try {
            await this.onInit(tenEnv);
        } catch (e) {
            // log
        } finally {
            ten_addon.ten_nodejs_ten_env_on_init_done(tenEnv);
        }
    }

    private async onStartProxy(
        tenEnv: TenEnv
    ): Promise<void> {
        try {
            await this.onStart(tenEnv);
        } catch (e) {
            // log
        } finally {
            ten_addon.ten_nodejs_ten_env_on_start_done(tenEnv);
        }
    }

    private async onStopProxy(
        tenEnv: TenEnv
    ): Promise<void> {
        try {
            await this.onStop(tenEnv);
        } catch (e) {
            // log
        } finally {
            ten_addon.ten_nodejs_ten_env_on_stop_done(tenEnv);
        }
    }

    private async onDeinitProxy(
        tenEnv: TenEnv
    ): Promise<void> {
        try {
            await this.onDeinit(tenEnv);
        } catch (e) {
            // log
        } finally {
            ten_addon.ten_nodejs_ten_env_on_deinit_done(tenEnv);

            /**
            * JS extension prepare to be destroyed, so notify the underlying C runtime this
            * fact.
            */
            ten_addon.ten_nodejs_extension_on_end_of_life(this);
        }
    }

    private async onCmdProxy(
        tenEnv: TenEnv,
        cmd: Cmd
    ): Promise<void> {
    }

    private async onDataProxy(
        tenEnv: TenEnv,
        data: Data
    ): Promise<void> {
    }

    private async onAudioFrameProxy(
        tenEnv: TenEnv,
        frame: AudioFrame
    ): Promise<void> {
    }

    private async onVideoFrameProxy(
        tenEnv: TenEnv,
        frame: VideoFrame
    ): Promise<void> {
    }

    async onConfigure(
        tenEnv: TenEnv,
    ): Promise<void> {
        // stub for override
    }

    async onInit(
        tenEnv: TenEnv,
    ): Promise<void> {
        // stub for override
    }

    async onStart(
        tenEnv: TenEnv,
    ): Promise<void> {
        // stub for override
    }

    async onStop(
        tenEnv: TenEnv,
    ): Promise<void> {
        // stub for override
    }

    async onDeinit(
        tenEnv: TenEnv,
    ): Promise<void> {
        // stub for override
    }
}