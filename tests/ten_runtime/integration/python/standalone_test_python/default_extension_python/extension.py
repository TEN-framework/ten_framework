#
# Copyright © 2025 Agora
# This file is part of TEN Framework, an open source project.
# Licensed under the Apache License, Version 2.0, with certain conditions.
# Refer to the "LICENSE" file in the root directory for more information.
#
from ten import (
    Extension,
    TenEnv,
    Cmd,
    StatusCode,
    CmdResult,
    Data,
    AudioFrame,
    VideoFrame,
)


class DefaultExtension(Extension):
    def __init__(self, name: str) -> None:
        super().__init__(name)
        self.name = name
        self.recv_data_count = 0
        self.cached_cmd = None

    def return_if_all_data_received(self, ten_env: TenEnv) -> None:
        if self.cached_cmd is not None and self.recv_data_count == 3:
            ten_env.log_info("All data received")
            cmd_result = CmdResult.create(StatusCode.OK)
            ten_env.return_result(cmd_result, self.cached_cmd)
            self.cached_cmd = None

    def on_data(self, ten_env: TenEnv, data: Data) -> None:
        if data.get_name() == "test":
            ten_env.log_info(
                "DefaultExtension on_data: " + data.get_property_to_json()
            )
            self.recv_data_count += 1
            self.return_if_all_data_received(ten_env)

    def on_audio_frame(self, ten_env: TenEnv, audio_frame: AudioFrame) -> None:
        if audio_frame.get_name() == "test":
            ten_env.log_info(
                "DefaultExtension on_audio_frame: "
                + audio_frame.get_property_to_json()
            )
            self.recv_data_count += 1
            self.return_if_all_data_received(ten_env)

    def on_video_frame(self, ten_env: TenEnv, video_frame: VideoFrame) -> None:
        if video_frame.get_name() == "test":
            ten_env.log_info(
                "DefaultExtension on_video_frame: "
                + video_frame.get_property_to_json()
            )
            self.recv_data_count += 1
            self.return_if_all_data_received(ten_env)

    def on_cmd(self, ten_env: TenEnv, cmd: Cmd) -> None:
        cmd_json = cmd.get_property_to_json()
        ten_env.log_info("DefaultExtension on_cmd json: " + cmd_json)

        if cmd.get_name() == "hello_world":
            self.cached_cmd = cmd
            self.return_if_all_data_received(ten_env)
        elif cmd.get_name() == "greeting":
            greeting = ten_env.get_property_string("greeting")
            cmd_result = CmdResult.create(StatusCode.OK)
            cmd_result.set_property_string("detail", greeting)
            ten_env.return_result(cmd_result, cmd)
