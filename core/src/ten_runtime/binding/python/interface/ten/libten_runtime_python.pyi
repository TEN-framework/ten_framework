#
# Copyright © 2025 Agora
# This file is part of TEN Framework, an open source project.
# Licensed under the Apache License, Version 2.0, with certain conditions.
# Refer to the "LICENSE" file in the root directory for more information.
#
from typing import Callable, Optional

from .ten_env_attach_to_enum import _TenEnvAttachTo
from .log_level import LogLevel
from .addon import Addon
from .error import TenError
from .ten_env import ResultHandler, ErrorHandler
from .test import (
    ResultHandler as TestResultHandler,
    ErrorHandler as TestErrorHandler,
)

class _TenError:
    def error_code(self) -> int: ...
    def error_message(self) -> str: ...

class _Msg:
    def get_name(self) -> str: ...
    def set_name(self, name: str) -> None: ...
    def set_dest(
        self,
        app_uri: Optional[str],
        graph_id: Optional[str],
        extension_group: Optional[str],
        extension: Optional[str],
    ) -> None: ...
    def set_property_from_json(
        self, path: Optional[str], json_str: str
    ) -> Optional[TenError]: ...
    def get_property_to_json(
        self, path: Optional[str] = None
    ) -> tuple[str, Optional[TenError]]: ...
    def get_property_int(self, path: str) -> tuple[int, Optional[TenError]]: ...
    def set_property_int(self, path: str, value: int) -> Optional[TenError]: ...
    def get_property_string(
        self, path: str
    ) -> tuple[str, Optional[TenError]]: ...
    def set_property_string(
        self, path: str, value: str
    ) -> Optional[TenError]: ...
    def get_property_bool(
        self, path: str
    ) -> tuple[bool, Optional[TenError]]: ...
    def set_property_bool(
        self, path: str, value: int
    ) -> Optional[TenError]: ...
    def get_property_float(
        self, path: str
    ) -> tuple[float, Optional[TenError]]: ...
    def set_property_float(
        self, path: str, value: float
    ) -> Optional[TenError]: ...
    def get_property_buf(
        self, path: str
    ) -> tuple[bytearray, Optional[TenError]]: ...
    def set_property_buf(
        self, path: str, value: bytes
    ) -> Optional[TenError]: ...

class _Cmd(_Msg):
    def __new__(cls, name: str): ...
    def clone(self) -> "_Cmd": ...

class _CmdResult(_Msg):
    def clone(self) -> "_CmdResult": ...
    def set_status_code(self, status_code: int) -> None: ...
    def get_status_code(self) -> int: ...
    def set_final(self, is_final_flag: int) -> None: ...
    def is_final(self) -> bool: ...
    def is_completed(self) -> bool: ...

class _Data(_Msg):
    def __new__(cls, name: str): ...
    def clone(self) -> "_Data": ...
    def alloc_buf(self, size: int) -> None: ...
    def lock_buf(self) -> memoryview: ...
    def unlock_buf(self, buf: memoryview) -> None: ...
    def get_buf(self) -> bytearray: ...

class _VideoFrame(_Msg):
    def __new__(cls, name: str): ...
    def clone(self) -> "_VideoFrame": ...
    def alloc_buf(self, size: int) -> None: ...
    def lock_buf(self) -> memoryview: ...
    def unlock_buf(self, buf: memoryview) -> None: ...
    def get_buf(self) -> bytearray: ...
    def get_width(self) -> int: ...
    def set_width(self, width: int) -> None: ...
    def get_height(self) -> int: ...
    def set_height(self, height: int) -> None: ...
    def get_timestamp(self) -> int: ...
    def set_timestamp(self, timestamp: int) -> None: ...
    def get_pixel_fmt(self) -> int: ...
    def set_pixel_fmt(self, pixel_fmt: int) -> None: ...
    def is_eof(self) -> bool: ...
    def set_eof(self, eof: bool) -> None: ...

class _AudioFrame(_Msg):
    def __new__(cls, name: str): ...
    def clone(self) -> "_AudioFrame": ...
    def alloc_buf(self, size: int) -> None: ...
    def lock_buf(self) -> memoryview: ...
    def unlock_buf(self, buf: memoryview) -> None: ...
    def get_buf(self) -> bytearray: ...
    def get_timestamp(self) -> int: ...
    def set_timestamp(self, timestamp: int) -> None: ...
    def get_sample_rate(self) -> int: ...
    def set_sample_rate(self, sample_rate: int) -> None: ...
    def get_samples_per_channel(self) -> int: ...
    def set_samples_per_channel(self, samples_per_channel: int) -> None: ...
    def get_bytes_per_sample(self) -> int: ...
    def set_bytes_per_sample(self, bytes_per_sample: int) -> None: ...
    def get_number_of_channels(self) -> int: ...
    def set_number_of_channels(self, number_of_channels: int) -> None: ...
    def get_data_fmt(self) -> int: ...
    def set_data_fmt(self, data_fmt: int) -> None: ...
    def get_line_size(self) -> int: ...
    def set_line_size(self, line_size: int) -> None: ...
    def is_eof(self) -> bool: ...
    def set_eof(self, is_eof: bool) -> None: ...

class _TenEnv:
    @property
    def _attach_to(self) -> _TenEnvAttachTo: ...
    def on_configure_done(self) -> None: ...
    def on_init_done(self) -> None: ...
    def on_start_done(self) -> None: ...
    def on_stop_done(self) -> None: ...
    def on_deinit_done(self) -> None: ...
    def on_create_instance_done(
        self, instance: _Extension, context
    ) -> None: ...
    def get_property_to_json(
        self, path: Optional[str] = None
    ) -> tuple[str, Optional[TenError]]: ...
    def set_property_from_json(
        self, path: str, json_str: str
    ) -> Optional[TenError]: ...
    def get_property_int(self, path: str) -> tuple[int, Optional[TenError]]: ...
    def set_property_int(self, path: str, value: int) -> Optional[TenError]: ...
    def get_property_string(
        self, path: str
    ) -> tuple[str, Optional[TenError]]: ...
    def set_property_string(
        self, path: str, value: str
    ) -> Optional[TenError]: ...
    def get_property_bool(
        self, path: str
    ) -> tuple[bool, Optional[TenError]]: ...
    def set_property_bool(
        self, path: str, value: int
    ) -> Optional[TenError]: ...
    def get_property_float(
        self, path: str
    ) -> tuple[float, Optional[TenError]]: ...
    def set_property_float(
        self, path: str, value: float
    ) -> Optional[TenError]: ...
    def is_property_exist(
        self, path: str
    ) -> tuple[bool, Optional[TenError]]: ...
    def init_property_from_json(self, json_str: str) -> Optional[TenError]: ...
    def get_property_to_json_async(
        self,
        path: Optional[str],
        callback: Callable[[str, Optional[TenError]], None],
    ) -> Optional[TenError]: ...
    def set_property_from_json_async(
        self,
        path: str,
        json_str: str,
        callback: Callable[[Optional[TenError]], None],
    ) -> Optional[TenError]: ...
    def get_property_int_async(
        self, path: str, callback: Callable[[int, Optional[TenError]], None]
    ) -> Optional[TenError]: ...
    def set_property_int_async(
        self,
        path: str,
        value: int,
        callback: Callable[[Optional[TenError]], None],
    ) -> Optional[TenError]: ...
    def get_property_string_async(
        self, path: str, callback: Callable[[str, Optional[TenError]], None]
    ) -> Optional[TenError]: ...
    def set_property_string_async(
        self,
        path: str,
        value: str,
        callback: Callable[[Optional[TenError]], None],
    ) -> Optional[TenError]: ...
    def get_property_bool_async(
        self, path: str, callback: Callable[[bool, Optional[TenError]], None]
    ) -> Optional[TenError]: ...
    def set_property_bool_async(
        self,
        path: str,
        value: int,
        callback: Callable[[Optional[TenError]], None],
    ) -> Optional[TenError]: ...
    def get_property_float_async(
        self, path: str, callback: Callable[[float, Optional[TenError]], None]
    ) -> Optional[TenError]: ...
    def set_property_float_async(
        self,
        path: str,
        value: float,
        callback: Callable[[Optional[TenError]], None],
    ) -> Optional[TenError]: ...
    def is_property_exist_async(
        self, path: str, callback: Callable[[bool, Optional[TenError]], None]
    ) -> Optional[TenError]: ...
    def init_property_from_json_async(
        self, json_str: str, callback: Callable[[Optional[TenError]], None]
    ) -> Optional[TenError]: ...
    def send_cmd(
        self, cmd: _Cmd, result_handler: Optional[ResultHandler], is_ex: bool
    ) -> Optional[TenError]: ...
    def send_data(
        self, data: _Data, error_handler: Optional[ErrorHandler]
    ) -> Optional[TenError]: ...
    def send_video_frame(
        self, video_frame: _VideoFrame, error_handler: Optional[ErrorHandler]
    ) -> Optional[TenError]: ...
    def send_audio_frame(
        self, audio_frame: _AudioFrame, error_handler: Optional[ErrorHandler]
    ) -> Optional[TenError]: ...
    def return_result(
        self,
        result: _CmdResult,
        target_cmd: _Cmd,
        error_handler: Optional[ErrorHandler],
    ) -> Optional[TenError]: ...
    def return_result_directly(
        self, result: _CmdResult, error_handler: Optional[ErrorHandler]
    ) -> Optional[TenError]: ...
    def log(
        self,
        level: LogLevel,
        func_name: Optional[str],
        file_name: Optional[str],
        line_no: int,
        msg: str,
    ) -> None: ...

class _App:
    def run(self, run_in_background_flag: int) -> None: ...
    def wait(self) -> None: ...
    def close(self) -> None: ...
    def on_init(
        self,
        ten_env: _TenEnv,
    ) -> None: ...
    def on_deinit(
        self,
        ten_env: _TenEnv,
    ) -> None: ...

class _Extension:
    def __new__(cls, name: str): ...
    def __init__(self, name: str) -> None: ...
    def on_init(
        self,
        ten_env: _TenEnv,
    ) -> None: ...
    def on_start(self, ten_env: _TenEnv) -> None: ...
    def on_stop(self, ten_env: _TenEnv) -> None: ...
    def on_deinit(self, ten_env: _TenEnv) -> None: ...
    def on_cmd(self, ten_env: _TenEnv, cmd: _Cmd) -> None: ...
    def on_data(self, ten_env: _TenEnv, data: _Data) -> None: ...
    def on_video_frame(
        self, ten_env: _TenEnv, video_frame: _VideoFrame
    ) -> None: ...
    def on_audio_frame(
        self, ten_env: _TenEnv, audio_frame: _AudioFrame
    ) -> None: ...

class _Addon:
    def on_init(
        self,
        ten_env: _TenEnv,
    ) -> None: ...
    def on_deinit(self, ten_env: _TenEnv) -> None: ...
    def on_create_instance(
        self, ten_env: _TenEnv, name: str, context
    ) -> None: ...

class _TenEnvTester:
    def on_start_done(self) -> None: ...
    def on_stop_done(self) -> None: ...
    def on_deinit_done(self) -> None: ...
    def send_cmd(
        self,
        cmd: _Cmd,
        result_handler: Optional[TestResultHandler],
        is_ex: bool,
    ) -> Optional[TenError]: ...
    def send_data(
        self, data: _Data, error_handler: Optional[TestErrorHandler]
    ) -> Optional[TenError]: ...
    def send_audio_frame(
        self,
        audio_frame: _AudioFrame,
        error_handler: Optional[TestErrorHandler],
    ) -> Optional[TenError]: ...
    def send_video_frame(
        self,
        video_frame: _VideoFrame,
        error_handler: Optional[TestErrorHandler],
    ) -> Optional[TenError]: ...
    def return_result(
        self,
        result: _CmdResult,
        target_cmd: _Cmd,
        error_handler: Optional[TestErrorHandler],
    ) -> Optional[TenError]: ...
    def stop_test(self) -> None: ...
    def log(
        self,
        level: LogLevel,
        func_name: Optional[str],
        file_name: Optional[str],
        line_no: int,
        msg: str,
    ) -> None: ...

class _ExtensionTester:
    def set_test_mode_single(
        self, addon_name: str, property_json_str: Optional[str]
    ) -> None: ...
    def run(self) -> None: ...

def _register_addon_as_extension(
    name: str, base_dir: Optional[str], instance: Addon, register_ctx: object
): ...
def _unregister_all_addons_and_cleanup() -> None: ...
