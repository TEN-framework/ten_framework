#
# Copyright © 2024 Agora
# This file is part of TEN Framework, an open source project.
# Licensed under the Apache License, Version 2.0, with certain conditions.
# Refer to the "LICENSE" file in the root directory for more information.
#
import asyncio
import threading
from typing import final
from libten_runtime_python import _Extension
from .video_frame import VideoFrame
from .audio_frame import AudioFrame
from .ten_env import TenEnv
from .cmd import Cmd
from .cmd_result import CmdResult
from .data import Data


class AsyncExtension(_Extension):
    def __init__(self, name: str) -> None:
        self.__ten_stop_event = asyncio.Event()

    async def __thread_routine(self, ten_env: TenEnv):
        self.__ten_loop = asyncio.get_running_loop()
        await self.on_configure(ten_env)

        # Suspend the thread until stopEvent is set
        await self.__ten_stop_event.wait()

        await self.on_deinit(ten_env)

    async def __stop_thread(self):
        self.__ten_stop_event.set()

    @final
    def _proxy_on_configure(self, ten_env: TenEnv) -> None:
        self.__ten_thread = threading.Thread(
            target=asyncio.run, args=(self.__thread_routine(ten_env),)
        )
        self.__ten_thread.start()

    @final
    def _proxy_on_init(self, ten_env: TenEnv) -> None:
        asyncio.run_coroutine_threadsafe(
            self.on_init(ten_env), self.__ten_loop)

    @final
    def _proxy_on_start(self, ten_env: TenEnv) -> None:
        asyncio.run_coroutine_threadsafe(
            self.on_start(ten_env), self.__ten_loop)

    @final
    def _proxy_on_stop(self, ten_env: TenEnv) -> None:
        asyncio.run_coroutine_threadsafe(
            self.on_stop(ten_env), self.__ten_loop)

    @final
    def _proxy_on_deinit(self, ten_env: TenEnv) -> None:
        asyncio.run_coroutine_threadsafe(self.__stop_thread(), self.__ten_loop)

    @final
    def _proxy_on_cmd(self, ten_env: TenEnv, cmd: Cmd) -> None:
        asyncio.run_coroutine_threadsafe(
            self.on_cmd(ten_env, cmd), self.__ten_loop)

    @final
    def _proxy_on_data(self, ten_env: TenEnv, data: Data) -> None:
        asyncio.run_coroutine_threadsafe(
            self.on_data(ten_env, data), self.__ten_loop)

    @final
    def _proxy_on_video_frame(self, ten_env: TenEnv, video_frame: VideoFrame) -> None:
        asyncio.run_coroutine_threadsafe(
            self.on_video_frame(ten_env, video_frame), self.__ten_loop)

    @final
    def _proxy_on_audio_frame(self, ten_env: TenEnv, audio_frame: AudioFrame) -> None:
        asyncio.run_coroutine_threadsafe(
            self.on_audio_frame(ten_env, audio_frame), self.__ten_loop)

    @final
    async def send_cmd_async(self, ten_env: TenEnv, cmd: Cmd) -> CmdResult:
        q = asyncio.Queue(1)
        ten_env.send_cmd(
            cmd,
            lambda ten_env, result: asyncio.run_coroutine_threadsafe(
                q.put(result), self.__ten_loop
            ),  # type: ignore
        )
        return await q.get()

    # Override these methods in your extension

    async def on_configure(self, ten_env: TenEnv) -> None:
        ten_env.on_configure_done()

    async def on_init(self, ten_env: TenEnv) -> None:
        ten_env.on_init_done()

    async def on_start(self, ten_env: TenEnv) -> None:
        ten_env.on_start_done()

    async def on_stop(self, ten_env: TenEnv) -> None:
        ten_env.on_stop_done()

    async def on_deinit(self, ten_env: TenEnv) -> None:
        ten_env.on_deinit_done()

    async def on_cmd(self, ten_env: TenEnv, cmd: Cmd) -> None:
        pass

    async def on_data(self, ten_env: TenEnv, data: Data) -> None:
        pass

    async def on_video_frame(self, ten_env: TenEnv, video_frame: VideoFrame) -> None:
        pass

    async def on_audio_frame(self, ten_env: TenEnv, audio_frame: AudioFrame) -> None:
        pass
