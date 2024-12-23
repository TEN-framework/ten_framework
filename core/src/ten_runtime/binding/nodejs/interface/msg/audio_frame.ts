//
// Copyright © 2024 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import ten_addon from '../ten_addon'
import { Msg } from './msg';

export class AudioFrame extends Msg {
    private constructor(name: string, createShellOnly: boolean) {
        super();

        if (createShellOnly) {
            return;
        }

        ten_addon.ten_nodejs_audio_frame_create(this, name);
    }

    static Create(name: string): AudioFrame {
        return new AudioFrame(name, false);
    }
}

ten_addon.ten_nodejs_audio_frame_register_class(AudioFrame);