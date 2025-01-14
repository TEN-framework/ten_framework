#
# Copyright © 2025 Agora
# This file is part of TEN Framework, an open source project.
# Licensed under the Apache License, Version 2.0, with certain conditions.
# Refer to the "LICENSE" file in the root directory for more information.
#
import asyncio
import os
import threading
import traceback
from typing import Optional, final, Optional

from libten_runtime_python import _ExtensionTester
from .cmd_result import CmdResult
from .error import TenError
from .cmd import Cmd
from .data import Data
from .audio_frame import AudioFrame
from .video_frame import VideoFrame
from .test import TenEnvTester
from .test_base import TenEnvTesterBase

CmdResultTuple = tuple[Optional[CmdResult], Optional[TenError]]


class AsyncTenEnvTester(TenEnvTesterBase):
    def __init__(
        self,
        ten_env_tester: TenEnvTester,
        loop: asyncio.AbstractEventLoop,
        thread: threading.Thread,
    ) -> None:
        super().__init__(ten_env_tester._internal)

        self._ten_loop = loop
        self._ten_thread = thread
        ten_env_tester._set_release_handler(lambda: self._on_release())

    def __del__(self) -> None:
        pass

    def _deinit_routine(self) -> None:
        # Wait for the internal thread to finish.
        self._ten_thread.join()

        # Since the `_ten_thread` used by the asyncio task queue has already
        # ended, we can be confident that no Python code will be using `ten_env`
        # at this point. (Of course, if the user has created their own Python
        # threads that are holding onto `ten_env`, they will need to handle the
        # thread-safety issues themselves.) Therefore, it is safe to call
        # `on_stop_done`, during which `ten_env` (_proxy) will be released.

        self._internal.on_stop_done()

    def _on_release(self) -> None:
        if hasattr(self, "_deinit_thread"):
            self._deinit_thread.join()

    def _deinit(self) -> None:
        # Start the deinit thread to avoid blocking the extension tester thread.
        self._deinit_thread = threading.Thread(target=self._deinit_routine)
        self._deinit_thread.start()

    async def send_cmd(self, cmd: Cmd) -> CmdResultTuple:
        q = asyncio.Queue(maxsize=1)
        self._internal.send_cmd(
            cmd,
            lambda _, result, error: asyncio.run_coroutine_threadsafe(
                q.put([result, error]),
                self._ten_loop,
            ),  # type: ignore
        )
        [result, error] = await q.get()

        if result is not None:
            assert result.is_completed()

        return result, error

    async def send_data(self, data: Data) -> Optional[TenError]:
        q = asyncio.Queue(maxsize=1)
        self._internal.send_data(
            data,
            lambda _, error: asyncio.run_coroutine_threadsafe(
                q.put([error]),
                self._ten_loop,
            ),  # type: ignore
        )
        error = await q.get()
        return error

    async def send_audio_frame(
        self, audio_frame: AudioFrame
    ) -> Optional[TenError]:
        q = asyncio.Queue(maxsize=1)
        self._internal.send_audio_frame(
            audio_frame,
            lambda _, error: asyncio.run_coroutine_threadsafe(
                q.put(error),
                self._ten_loop,
            ),  # type: ignore
        )
        error = await q.get()
        return error

    async def send_video_frame(
        self, video_frame: VideoFrame
    ) -> Optional[TenError]:
        q = asyncio.Queue(maxsize=1)
        self._internal.send_video_frame(
            video_frame,
            lambda _, error: asyncio.run_coroutine_threadsafe(
                q.put(error),
                self._ten_loop,
            ),  # type: ignore
        )
        error = await q.get()
        return error

    async def return_result(
        self,
        cmd_result: CmdResult,
        target_cmd: Cmd,
    ) -> None:
        q = asyncio.Queue(maxsize=1)
        self._internal.return_result(
            cmd_result,
            target_cmd,
            lambda _, error: asyncio.run_coroutine_threadsafe(
                q.put(error),
                self._ten_loop,
            ),  # type: ignore
        )
        error = await q.get()
        if error is not None:
            raise RuntimeError(error.err_msg())

    def stop_test(self) -> None:
        return self._internal.stop_test()


class AsyncExtensionTester(_ExtensionTester):
    def __init__(self) -> None:
        self._ten_stop_event = asyncio.Event()

    def __del__(self) -> None:
        pass

    def _exit_on_exception(
        self, async_ten_env_tester: AsyncTenEnvTester, e: Exception
    ):
        traceback_info = traceback.format_exc()
        async_ten_env_tester.log_fatal(
            f"Uncaught exception: {e} \ntraceback: {traceback_info}"
        )
        os._exit(1)

    async def _thread_routine(self, ten_env_tester: TenEnvTester) -> None:
        self._ten_loop = asyncio.get_running_loop()
        self._async_ten_env_tester = AsyncTenEnvTester(
            ten_env_tester, self._ten_loop, self._ten_thread
        )

        await self._wrapper_on_start(self._async_ten_env_tester)
        ten_env_tester.on_start_done()

        # Suspend the thread until stopEvent is set.
        await self._ten_stop_event.wait()

        await self._wrapper_on_stop(self._async_ten_env_tester)

        # We cannot directly call `on_stop_done` here as above, because after
        # `on_stop_done`, `ten_env_proxy` will be released. Therefore, we need
        # to wait until certain essential tasks are completed before calling
        # `on_stop_done`. Otherwise, if anything needs to use `ten_env` (_proxy)
        # after `on_stop_done`, it will cause issues.

        self._async_ten_env_tester._deinit()

    async def _stop_thread(self):
        self._ten_stop_event.set()

    @final
    def _proxy_on_start(self, ten_env_tester: TenEnvTester) -> None:
        self._ten_thread = threading.Thread(
            target=asyncio.run, args=(self._thread_routine(ten_env_tester),)
        )
        self._ten_thread.start()

    async def _wrapper_on_start(
        self, ten_env_tester: AsyncTenEnvTester
    ) -> None:
        try:
            await self.on_start(ten_env_tester)
        except Exception as e:
            self._exit_on_exception(ten_env_tester, e)

    async def _wrapper_on_stop(self, ten_env_tester: AsyncTenEnvTester) -> None:
        try:
            await self.on_stop(ten_env_tester)
        except Exception as e:
            self._exit_on_exception(ten_env_tester, e)

    @final
    def _proxy_on_stop(self, ten_env_tester: TenEnvTester) -> None:
        asyncio.run_coroutine_threadsafe(self._stop_thread(), self._ten_loop)

    @final
    def _proxy_on_cmd(self, ten_env_tester: TenEnvTester, cmd: Cmd) -> None:
        asyncio.run_coroutine_threadsafe(
            self._wrapper_on_cmd(self._async_ten_env_tester, cmd),
            self._ten_loop,
        )

    async def _wrapper_on_cmd(
        self, ten_env_tester: AsyncTenEnvTester, cmd: Cmd
    ) -> None:
        try:
            await self.on_cmd(ten_env_tester, cmd)
        except Exception as e:
            self._exit_on_exception(ten_env_tester, e)

    @final
    def _proxy_on_data(self, ten_env_tester: TenEnvTester, data: Data) -> None:
        asyncio.run_coroutine_threadsafe(
            self._wrapper_on_data(self._async_ten_env_tester, data),
            self._ten_loop,
        )

    async def _wrapper_on_data(
        self, ten_env_tester: AsyncTenEnvTester, data: Data
    ) -> None:
        try:
            await self.on_data(ten_env_tester, data)
        except Exception as e:
            self._exit_on_exception(ten_env_tester, e)

    @final
    def _proxy_on_audio_frame(
        self, ten_env_tester: TenEnvTester, audio_frame: AudioFrame
    ) -> None:
        asyncio.run_coroutine_threadsafe(
            self._wrapper_on_audio_frame(
                self._async_ten_env_tester, audio_frame
            ),
            self._ten_loop,
        )

    async def _wrapper_on_audio_frame(
        self, ten_env_tester: AsyncTenEnvTester, audio_frame: AudioFrame
    ) -> None:
        try:
            await self.on_audio_frame(ten_env_tester, audio_frame)
        except Exception as e:
            self._exit_on_exception(ten_env_tester, e)

    @final
    def _proxy_on_video_frame(
        self, ten_env_tester: TenEnvTester, video_frame: VideoFrame
    ) -> None:
        asyncio.run_coroutine_threadsafe(
            self._wrapper_on_video_frame(
                self._async_ten_env_tester, video_frame
            ),
            self._ten_loop,
        )

    async def _wrapper_on_video_frame(
        self, ten_env_tester: AsyncTenEnvTester, video_frame: VideoFrame
    ) -> None:
        try:
            await self.on_video_frame(ten_env_tester, video_frame)
        except Exception as e:
            self._exit_on_exception(ten_env_tester, e)

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

    async def on_start(self, ten_env_tester: AsyncTenEnvTester) -> None:
        pass

    async def on_stop(self, ten_env_tester: AsyncTenEnvTester) -> None:
        pass

    async def on_cmd(self, ten_env_tester: AsyncTenEnvTester, cmd: Cmd) -> None:
        pass

    async def on_data(
        self, ten_env_tester: AsyncTenEnvTester, data: Data
    ) -> None:
        pass

    async def on_audio_frame(
        self, ten_env_tester: AsyncTenEnvTester, audio_frame: AudioFrame
    ) -> None:
        pass

    async def on_video_frame(
        self, ten_env_tester: AsyncTenEnvTester, video_frame: VideoFrame
    ) -> None:
        pass
