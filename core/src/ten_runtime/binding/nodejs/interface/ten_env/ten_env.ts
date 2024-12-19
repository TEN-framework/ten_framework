//
// Copyright © 2024 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import { AudioFrame } from '../msg/audio_frame';
import { Cmd } from '../msg/cmd';
import { CmdResult } from '../msg/cmd_result';
import { Data } from '../msg/data';
import ten_addon from '../ten_addon'
import { LogLevel } from './log_level';

export class TenEnv {
    async sendCmd(cmd: Cmd): Promise<CmdResult> {
        return new Promise<CmdResult>((resolve, reject) => {
            ten_addon.ten_nodejs_ten_env_send_cmd(
                this,
                cmd,
                async (tenEnv: TenEnv, cmdResult: CmdResult, error: Error) => {
                    if (error) {
                        reject(error);
                    } else {
                        resolve(cmdResult);
                    }
                }
            )
        });
    }

    async sendData(data: Data): Promise<void> {
        return new Promise<void>((resolve, reject) => {
            ten_addon.ten_nodejs_ten_env_send_data(
                this,
                data,
                async (tenEnv: TenEnv, error: Error) => {
                    if (error) {
                        reject(error);
                    } else {
                        resolve();
                    }
                }
            )
        });
    }

    async sendVideoFrame(videoFrame: VideoFrame) {
        return new Promise<void>((resolve, reject) => {
            ten_addon.ten_nodejs_ten_env_send_video_frame(
                this,
                videoFrame,
                async (tenEnv: TenEnv, error: Error) => {
                    if (error) {
                        reject(error);
                    } else {
                        resolve();
                    }
                }
            )
        });
    }

    async sendAudioFrame(audioFrame: AudioFrame) {
        return new Promise<void>((resolve, reject) => {
            ten_addon.ten_nodejs_ten_env_send_audio_frame(
                this,
                audioFrame,
                async (tenEnv: TenEnv, error: Error) => {
                    if (error) {
                        reject(error);
                    } else {
                        resolve();
                    }
                }
            )
        });
    }

    async returnResult(cmdResult: CmdResult, targetCmd: Cmd) {
        return new Promise<void>((resolve, reject) => {
            ten_addon.ten_nodejs_ten_env_return_result(
                this,
                cmdResult,
                targetCmd,
                async (tenEnv: TenEnv, error: Error) => {
                    if (error) {
                        reject(error);
                    } else {
                        resolve();
                    }
                }
            )
        });
    }

    async returnResultDirectly(cmdResult: CmdResult) {
        return new Promise<void>((resolve, reject) => {
            ten_addon.ten_nodejs_ten_env_return_result_directly(
                this,
                cmdResult,
                async (tenEnv: TenEnv, error: Error) => {
                    if (error) {
                        reject(error);
                    } else {
                        resolve();
                    }
                }
            )
        });
    }

    async isPropertyExist(path: string): Promise<boolean> {
        return new Promise<boolean>((resolve) => {
            ten_addon.ten_nodejs_ten_env_is_property_exist(
                this,
                path,
                async (result: boolean) => {
                    resolve(result);
                }
            )
        });
    }

    async getPropertyToJson(path: string): Promise<string> {
        return new Promise<string>((resolve, reject) => {
            ten_addon.ten_nodejs_ten_env_get_property_to_json(
                path,
                async (result: string, error: Error) => {
                    if (error) {
                        reject(error);
                    } else {
                        resolve(result);
                    }
                }
            )
        });
    }

    async setPropertyFromJson(path: string, jsonStr: string): Promise<void> {
        return new Promise<void>((resolve, reject) => {
            ten_addon.ten_nodejs_ten_env_set_property_from_json(
                path,
                jsonStr,
                async (error: Error) => {
                    if (error) {
                        reject(error);
                    } else {
                        resolve();
                    }
                }
            )
        });
    }

    async getPropertyNumber(path: string): Promise<number> {
        return new Promise<number>((resolve, reject) => {
            ten_addon.ten_nodejs_ten_env_get_property_number(
                path,
                async (result: number, error: Error) => {
                    if (error) {
                        reject(error);
                    } else {
                        resolve(result);
                    }
                }
            )
        });
    }

    async setPropertyNumber(path: string, value: number): Promise<void> {
        return new Promise<void>((resolve, reject) => {
            ten_addon.ten_nodejs_ten_env_set_property_number(
                path,
                value,
                async (error: Error) => {
                    if (error) {
                        reject(error);
                    } else {
                        resolve();
                    }
                }
            )
        });
    }

    async getPropertyString(path: string): Promise<string> {
        return new Promise<string>((resolve, reject) => {
            ten_addon.ten_nodejs_ten_env_get_property_string(
                path,
                async (result: string, error: Error) => {
                    if (error) {
                        reject(error);
                    } else {
                        resolve(result);
                    }
                }
            )
        });
    }

    async setPropertyString(path: string, value: string): Promise<void> {
        return new Promise<void>((resolve, reject) => {
            ten_addon.ten_nodejs_ten_env_set_property_string(
                path,
                value,
                async (error: Error) => {
                    if (error) {
                        reject(error);
                    } else {
                        resolve();
                    }
                }
            )
        });
    }

    log_verbose(message: string): void {
        this._log_internal(LogLevel.VERBOSE, message)
    }

    log_debug(message: string): void {
        this._log_internal(LogLevel.DEBUG, message)
    }

    log_info(message: string): void {
        this._log_internal(LogLevel.INFO, message)
    }

    log_warn(message: string): void {
        this._log_internal(LogLevel.WARN, message)
    }

    log_error(message: string): void {
        this._log_internal(LogLevel.ERROR, message)
    }

    log_fatal(message: string): void {
        this._log_internal(LogLevel.FATAL, message)
    }

    _log_internal(level: number, message: string): void {
        ten_addon.ten_nodejs_ten_env_log_internal(this, level, message);
    }
}

ten_addon.ten_nodejs_ten_env_register_class(TenEnv);