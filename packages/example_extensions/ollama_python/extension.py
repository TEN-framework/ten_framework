#
# This file is part of TEN Framework, an open source project.
# Licensed under the Apache License, Version 2.0.
# See the LICENSE file for more information.
#
from ten import (
    AsyncExtension,
    AsyncTenEnv,
    Cmd,
    StatusCode,
    CmdResult,
)
from ollama import chat
from ollama import ChatResponse


class OllamaExtension(AsyncExtension):
    async def on_cmd(self, ten_env: AsyncTenEnv, cmd: Cmd) -> None:
        cmd_name = cmd.get_name()
        ten_env.log_debug("on_cmd name {}".format(cmd_name))

        if cmd_name == "ask":
            question = cmd.get_property_string("question")

            model = "smollm:135m"
            response: ChatResponse = chat(
                model=model,
                messages=[
                    {
                        "role": "user",
                        "content": question,
                    }
                ],
                stream=True,
            )

            for chunk in response:
                cmd_result = CmdResult.create(StatusCode.OK)
                cmd_result.set_property_string(
                    "response", chunk["message"]["content"]
                )
                cmd_result.set_final(False)
                await ten_env.return_result(cmd_result, cmd)

            # Return the final response.
            cmd_result = CmdResult.create(StatusCode.OK)
            await ten_env.return_result(cmd_result, cmd)
