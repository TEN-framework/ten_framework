#
# Copyright © 2025 Agora
# This file is part of TEN Framework, an open source project.
# Licensed under the Apache License, Version 2.0, with certain conditions.
# Refer to the "LICENSE" file in the root directory for more information.
#
from typing import Callable, Optional, final, Optional

from libten_runtime_python import _ExtensionTester, _TenEnvTester
from .cmd_result import CmdResult
from .error import TenError
from .cmd import Cmd
from .data import Data
from .audio_frame import AudioFrame
from .video_frame import VideoFrame


class TenEnvTester: ...  # type: ignore


ResultHandler = Optional[
    Callable[[TenEnvTester, Optional[CmdResult], Optional[TenError]], None]
]

ErrorHandler = Optional[Callable[[TenEnvTester, Optional[TenError]], None]]


class TenEnvTester:

    def __init__(self, internal_obj: _TenEnvTester) -> None:
        self._internal = internal_obj

    def __del__(self) -> None:
        pass

    def on_start_done(self) -> None:
        return self._internal.on_start_done()

    def send_cmd(self, cmd: Cmd, result_handler: ResultHandler) -> None:
        return self._internal.send_cmd(cmd, result_handler)

    def send_data(self, data: Data, error_handler: ErrorHandler = None) -> None:
        return self._internal.send_data(data, error_handler)

    def send_audio_frame(
        self, audio_frame: AudioFrame, error_handler: ErrorHandler = None
    ) -> None:
        return self._internal.send_audio_frame(audio_frame, error_handler)

    def send_video_frame(
        self, video_frame: VideoFrame, error_handler: ErrorHandler = None
    ) -> None:
        return self._internal.send_video_frame(video_frame, error_handler)

    def return_result(
        self,
        cmd_result: CmdResult,
        target_cmd: Cmd,
        error_handler: ErrorHandler = None,
    ) -> None:
        return self._internal.return_result(
            cmd_result, target_cmd, error_handler
        )

    def stop_test(self) -> None:
        return self._internal.stop_test()


class ExtensionTester(_ExtensionTester):
    @final
    def _on_test_app_configure(self, ten_env_tester: TenEnvTester) -> None:
        self.on_start(ten_env_tester)

    @final
    def set_test_mode_single(
        self, addon_name: str, property_json_str: Optional[str] = None
    ) -> None:
        return _ExtensionTester.set_test_mode_single(
            self, addon_name, property_json_str
        )

    @final
    def run(self) -> None:
        return _ExtensionTester.run(self)

    def on_start(self, ten_env_tester: TenEnvTester) -> None:
        ten_env_tester.on_start_done()

    def on_cmd(self, ten_env_tester: TenEnvTester, cmd: Cmd) -> None:
        pass

    def on_data(self, ten_env_tester: TenEnvTester, data: Data) -> None:
        pass

    def on_audio_frame(
        self, ten_env_tester: TenEnvTester, audio_frame: AudioFrame
    ) -> None:
        pass

    def on_video_frame(
        self, ten_env_tester: TenEnvTester, video_frame: VideoFrame
    ) -> None:
        pass
