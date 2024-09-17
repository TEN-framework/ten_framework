#
# Copyright © 2024 Agora
# This file is part of TEN Framework, an open source project.
# Licensed under the Apache License, Version 2.0, with certain conditions.
# Refer to https://github.com/TEN-framework/ten_framework/LICENSE for more
# information.
#
from enum import IntEnum


class LogLevel(IntEnum):
    INVALID = 0
    VERBOSE = 1
    DEBUG = 2
    INFO = 3
    WARN = 4
    ERROR = 5
    FATAL = 6
