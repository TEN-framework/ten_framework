#
# Copyright © 2025 Agora
# This file is part of TEN Framework, an open source project.
# Licensed under the Apache License, Version 2.0, with certain conditions.
# Refer to the "LICENSE" file in the root directory for more information.
#
import asyncio
from ten import AsyncExtension, AsyncTenEnv, Cmd, StatusCode, CmdResult


class DefaultAsyncExtension(AsyncExtension):
    async def on_cmd(self, ten_env: AsyncTenEnv, cmd: Cmd) -> None:
        cmd_json = cmd.get_property_to_json()
        ten_env.log_debug(f"on_cmd: {cmd_json}")

        if cmd.get_name() == "test":
            cloned_cmd = cmd.clone()
            result, _ = await ten_env.send_cmd(cloned_cmd)
            assert result is not None
            assert result.get_status_code() == StatusCode.OK

            cmd_result = CmdResult.create(StatusCode.OK)
            cmd_result.set_property_string("detail", "im ok!")
            await ten_env.return_result(cmd_result, cmd)
