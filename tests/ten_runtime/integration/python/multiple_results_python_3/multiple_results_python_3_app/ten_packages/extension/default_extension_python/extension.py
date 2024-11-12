#
# Copyright © 2024 Agora
# This file is part of TEN Framework, an open source project.
# Licensed under the Apache License, Version 2.0, with certain conditions.
# Refer to the "LICENSE" file in the root directory for more information.
#
from ten import (
    Extension,
    TenEnv,
    Cmd,
    StatusCode,
    CmdResult,
)


class DefaultExtension(Extension):
    def __init__(self, name: str) -> None:
        super().__init__(name)
        self.name = name

        self.__counter = 0

    def on_init(self, ten_env: TenEnv) -> None:
        ten_env.log_debug("on_init")
        ten_env.on_init_done()

    def check_hello(self, ten_env: TenEnv, result: CmdResult, receivedCmd: Cmd):
        self.__counter += 1

        if self.__counter == 1:
            assert result.is_completed() is False
            ten_env.log_info("receive 1 cmd result")
        elif self.__counter == 2:
            assert result.is_completed() is True
            ten_env.log_info("receive 2 cmd result")

            respCmd = CmdResult.create(StatusCode.OK)
            respCmd.set_property_string("detail", "nbnb")
            ten_env.return_result(respCmd, receivedCmd)
        else:
            assert 0

    def on_cmd(self, ten_env: TenEnv, cmd: Cmd) -> None:
        cmd_json = cmd.to_json()
        ten_env.log_debug(f"on_cmd json: {cmd_json}")

        if self.name == "default_extension_python_1":
            new_cmd = Cmd.create("hello")
            ten_env.send_cmd(
                new_cmd,
                lambda ten_env, result: self.check_hello(ten_env, result, cmd),
            )
        elif self.name == "default_extension_python_2":
            ten_env.log_info("create respCmd")
            respCmd = CmdResult.create(StatusCode.OK)
            ten_env.return_result(respCmd, cmd)
        elif self.name == "default_extension_python_3":
            ten_env.log_info("create respCmd")
            respCmd = CmdResult.create(StatusCode.OK)
            ten_env.return_result(respCmd, cmd)