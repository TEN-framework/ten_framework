#
# Copyright © 2025 Agora
# This file is part of TEN Framework, an open source project.
# Licensed under the Apache License, Version 2.0, with certain conditions.
# Refer to the "LICENSE" file in the root directory for more information.
#
from ten import (
    AsyncExtensionTester,
    AsyncTenEnvTester,
    Cmd,
)


class OllamaExtensionTester(AsyncExtensionTester):
    async def on_start(self, ten_env: AsyncTenEnvTester) -> None:
        ten_env.log_debug("send ask cmd")

        ask_cmd = Cmd.create("ask")
        ask_cmd.set_property_string("question", "hi")
        async for result, error in ten_env.send_cmd_ex(ask_cmd):
            if error is not None:
                ten_env.log_error(f"error: {error}")
            elif result is not None:
                if result.is_completed():
                    ten_env.stop_test()
                    break
                else:
                    ten_env.log_info(
                        f"result: {result.get_property_string("response")}"
                    )


def test_basic():
    tester = OllamaExtensionTester()
    tester.set_test_mode_single("ollama_python")
    tester.run()
