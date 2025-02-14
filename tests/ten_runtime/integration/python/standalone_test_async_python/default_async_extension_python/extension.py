#
# This file is part of TEN Framework, an open source project.
# Licensed under the Apache License, Version 2.0.
# See the LICENSE file for more information.
#
import asyncio
import time
from ten import (
    AudioFrame,
    VideoFrame,
    AsyncExtension,
    AsyncTenEnv,
    Cmd,
    StatusCode,
    CmdResult,
    Data,
)


class DefaultAsyncExtension(AsyncExtension):
    async def on_init(self, ten_env: AsyncTenEnv) -> None:
        ten_env.log_debug("on_init")
        try:
            self.send_goodbye_cmd = await ten_env.get_property_bool(
                "send_goodbye_cmd"
            )
        except Exception:
            self.send_goodbye_cmd = False

        try:
            self.sleep_ms_before_goodbye = await ten_env.get_property_int(
                "sleep_ms_before_goodbye"
            )
        except Exception:
            self.sleep_ms_before_goodbye = 0

        try:
            self.assert_goodbye_result_success = (
                await ten_env.get_property_bool("assert_goodbye_result_success")
            )
        except Exception:
            self.assert_goodbye_result_success = False

    async def on_start(self, ten_env: AsyncTenEnv) -> None:
        ten_env.log_debug("on_start")

    async def on_deinit(self, ten_env: AsyncTenEnv) -> None:
        ten_env.log_debug("on_deinit")

    async def on_cmd(self, ten_env: AsyncTenEnv, cmd: Cmd) -> None:
        cmd_name = cmd.get_name()
        ten_env.log_debug("on_cmd name {}".format(cmd_name))

        if cmd_name == "query_weather":
            # Send a command to query weather.
            query_cmd = Cmd.create("query_weather")
            cmd_result, error = await ten_env.send_cmd(query_cmd)
            assert error is None
            assert cmd_result is not None

            # Get the weather detail.
            weather = cmd_result.get_property_string("detail")

            # Return the weather detail.
            cmd_result = CmdResult.create(StatusCode.OK)
            cmd_result.set_property_string("detail", weather)
            await ten_env.return_result(cmd_result, cmd)
        elif cmd_name == "flush":
            # Bypass the flush command. The purpose of bypassing this `flush`
            # command here is to allow the extension tester to call `stop_test`.
            await ten_env.send_cmd(Cmd.create("flush"))

            cmd_result = CmdResult.create(StatusCode.OK)
            await ten_env.return_result(cmd_result, cmd)

    async def on_data(self, ten_env: AsyncTenEnv, data: Data) -> None:
        data_name = data.get_name()
        ten_env.log_debug("on_data name {}".format(data_name))

        # TODO: process data
        pass

    async def on_audio_frame(
        self, ten_env: AsyncTenEnv, audio_frame: AudioFrame
    ) -> None:
        audio_frame_name = audio_frame.get_name()
        ten_env.log_debug("on_audio_frame name {}".format(audio_frame_name))

        # TODO: process audio frame
        pass

    async def on_video_frame(
        self, ten_env: AsyncTenEnv, video_frame: VideoFrame
    ) -> None:
        video_frame_name = video_frame.get_name()
        ten_env.log_debug("on_video_frame name {}".format(video_frame_name))

        # TODO: process video frame
        pass

    async def on_stop(self, ten_env: AsyncTenEnv) -> None:
        if self.send_goodbye_cmd:
            if self.sleep_ms_before_goodbye > 0:
                await asyncio.sleep(self.sleep_ms_before_goodbye / 1000)

            current_time = time.time()

            cmd = Cmd.create("goodbye")
            result, error = await ten_env.send_cmd(cmd)

            if self.assert_goodbye_result_success:
                assert error is None
                assert result is not None
                assert result.get_status_code() == StatusCode.OK

            cost_time = time.time() - current_time
            ten_env.log_info("goodbye cost time {} ms".format(cost_time * 1000))

            # To rule out that the result reply was triggered by path_timeout,
            # we need to ensure cost_time is less than 1 second.
            assert cost_time < 1
