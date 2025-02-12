#
# Copyright © 2025 Agora
# This file is part of TEN Framework, an open source project.
# Licensed under the Apache License, Version 2.0, with certain conditions.
# Refer to the "LICENSE" file in the root directory for more information.
#
import asyncio
import os
import sys
import threading
import traceback
from typing import Optional, final

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

    def __del__(self) -> None:
        pass

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
            raise RuntimeError(error.error_message())

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
        try:
            async_ten_env_tester.log_fatal(
                f"Uncaught exception: {e} \ntraceback: {traceback_info}"
            )
        except Exception as e:
            # If the log_fatal API fails, print the error message to the
            # console.
            print(f"Uncaught exception: {e} \ntraceback: {traceback_info}")

        # `os._exit` directly calls C's `_exit`, but as a result, it does not
        # flush `stdout/stderr`, which may cause some logs to not be output.
        # Therefore, flushing is proactively called to avoid this issue.
        sys.stdout.flush()
        sys.stderr.flush()

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

    @final
    async def _stop_thread(self) -> None:
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
    async def _proxy_async_on_stop(self, ten_env_tester: TenEnvTester):
        await self._wrapper_on_stop(self._async_ten_env_tester)
        ten_env_tester._internal.on_stop_done()

    @final
    def _proxy_on_stop(self, ten_env_tester: TenEnvTester) -> None:
        asyncio.run_coroutine_threadsafe(
            self._proxy_async_on_stop(ten_env_tester), self._ten_loop
        )

    async def _wrapper_on_deinit(
        self, ten_env_tester: AsyncTenEnvTester
    ) -> None:
        try:
            await self.on_deinit(ten_env_tester)
        except Exception as e:
            self._exit_on_exception(ten_env_tester, e)

    @final
    async def _proxy_async_on_deinit(
        self, ten_env_tester: TenEnvTester
    ) -> None:
        await self._wrapper_on_deinit(self._async_ten_env_tester)
        ten_env_tester._internal.on_deinit_done()

    @final
    def _proxy_on_deinit(self, ten_env_tester: TenEnvTester) -> None:
        asyncio.run_coroutine_threadsafe(
            self._proxy_async_on_deinit(ten_env_tester),
            self._ten_loop,
        )

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
        # This is a blocking operation.
        _ExtensionTester.run(self)

        # The `extension_tester` has two attributes.
        #
        # 1. All `on_xxx` APIs of the `extension_tester` act as proxies for
        # `test_extension`. From this perspective, the `extension_tester` has
        # the attributes of an `extension`.
        #
        # 2. The `extension_tester` also has a `run` API, which is equivalent to
        # the `run` API of an `app` in a testing scenario. From this
        # perspective, the `extension_tester` has the attributes of an `app`.
        #
        # When the `run` API of the `extension_tester` completes, it signifies
        # that all tasks in the extension world have finished and no new tasks
        # will be added (since the extension context has fully ended). At this
        # point, it is an appropriate time to enqueue a final close task into
        # the asyncio run loop queue to terminate the asyncio thread.
        #
        # In the context of an `async_extension`, since there is no specific
        # `run` API completion point, a similar behavior is implemented using a
        # callback triggered after `on_deinit_done`.

        # If execution reaches this point, it means that the TEN app thread used
        # for testing has already ended. Therefore, the process to terminate the
        # asyncio thread/runloop is initiated.

        if hasattr(self, "_ten_thread") and self._ten_thread.is_alive():
            asyncio.run_coroutine_threadsafe(
                self._stop_thread(), self._ten_loop
            )

            # Wait for the asyncio thread to finish.
            self._ten_thread.join()

    async def on_start(self, ten_env_tester: AsyncTenEnvTester) -> None:
        pass

    async def on_stop(self, ten_env_tester: AsyncTenEnvTester) -> None:
        pass

    async def on_deinit(self, ten_env_tester: AsyncTenEnvTester) -> None:
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
