#
# Copyright © 2025 Agora
# This file is part of TEN Framework, an open source project.
# Licensed under the Apache License, Version 2.0, with certain conditions.
# Refer to the "LICENSE" file in the root directory for more information.
#
import asyncio
from ten import (
    AsyncExtension,
    AsyncTenEnv,
    Cmd,
    CmdResult,
    StatusCode,
)


class ServerExtension(AsyncExtension):
    def __init__(self, name: str) -> None:
        super().__init__(name)
        self.name = name
        self.register_count = 0

    async def on_cmd(self, ten_env: AsyncTenEnv, cmd: Cmd) -> None:
        if cmd.get_name() == "register":
            self.register_count += 1
            await ten_env.return_result(CmdResult.create(StatusCode.OK), cmd)
        elif cmd.get_name() == "unregister":
            self.register_count -= 1
            await ten_env.return_result(CmdResult.create(StatusCode.OK), cmd)
        elif cmd.get_name() == "test":
            cmd_result = CmdResult.create(StatusCode.OK)
            cmd_result.set_property_string("detail", "ok")
            await ten_env.return_result(cmd_result, cmd)
        else:
            assert False

    async def on_stop(self, ten_env: AsyncTenEnv) -> None:
        if self.register_count == 0:
            return

        # Wait for the client to unregister.
        while self.register_count > 0:
            await asyncio.sleep(0.5)

        ten_env.log_info("server extension is stopped")


class ClientExtension(AsyncExtension):
    def __init__(self, name: str) -> None:
        super().__init__(name)
        self.name = name
        self.task_after_deinited = None

    def __del__(self):
        assert self.task_after_deinited is not None
        result = self.task_after_deinited.result()
        assert result == "ok"

    async def on_start(self, ten_env: AsyncTenEnv) -> None:
        cmd = Cmd.create("register")
        cmd_result, _ = await ten_env.send_cmd(cmd)
        assert cmd_result is not None
        assert cmd_result.get_status_code() == StatusCode.OK

    async def on_stop(self, ten_env: AsyncTenEnv) -> None:
        pass

    async def call_api_after_deinit(self, ten_env: AsyncTenEnv) -> str:
        cancel_exception_caught = False

        try:
            await asyncio.sleep(1)
        except asyncio.CancelledError:
            cancel_exception_caught = True
        finally:
            assert cancel_exception_caught

            ten_env_api_exception_caught = False
            try:
                # call ten_env api will throw exception
                ten_env.log_info("call ten_env api will throw exception")
            except Exception as e:
                ten_env_api_exception_caught = True
            finally:
                assert ten_env_api_exception_caught

        return "ok"

    async def on_deinit(self, ten_env: AsyncTenEnv) -> None:
        cmd = Cmd.create("unregister")
        cmd_result, _ = await ten_env.send_cmd(cmd)
        assert cmd_result is not None
        assert cmd_result.get_status_code() == StatusCode.OK

        ten_env.log_info("client extension is de-initialized")

        self.task_after_deinited = asyncio.create_task(
            self.call_api_after_deinit(ten_env)
        )

    async def on_cmd(self, ten_env: AsyncTenEnv, cmd: Cmd) -> None:
        # bypass the cmd to the next extension
        result, _ = await ten_env.send_cmd(cmd)
        assert result is not None

        await ten_env.return_result_directly(result)
