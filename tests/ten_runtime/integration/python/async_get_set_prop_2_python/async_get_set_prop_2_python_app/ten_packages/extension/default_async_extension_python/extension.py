#
# Copyright © 2025 Agora
# This file is part of TEN Framework, an open source project.
# Licensed under the Apache License, Version 2.0, with certain conditions.
# Refer to the "LICENSE" file in the root directory for more information.
#
import json
from ten import (
    AsyncExtension,
    AsyncTenEnv,
    Cmd,
)


class DefaultAsyncExtension(AsyncExtension):

    async def on_init(self, ten_env: AsyncTenEnv) -> None:
        test_prop_json_str = await ten_env.get_property_to_json("test_prop")
        test_prop_json = json.loads(test_prop_json_str)
        assert test_prop_json == 32

        test_prop_json_str = await ten_env.get_property_to_json()
        test_prop_json = json.loads(test_prop_json_str)
        assert test_prop_json["test_prop"] == 32

        test_prop_json_str = await ten_env.get_property_to_json("")
        test_prop_json = json.loads(test_prop_json_str)
        assert test_prop_json["test_prop"] == 32

    async def on_cmd(self, ten_env: AsyncTenEnv, cmd: Cmd) -> None:
        # Send a new command to other extensions and wait for the result. The
        # result will be returned to the original sender.
        new_cmd = Cmd.create("hello")
        cmd_result, _ = await ten_env.send_cmd(new_cmd)
        assert cmd_result is not None

        await ten_env.return_result(cmd_result, cmd)
