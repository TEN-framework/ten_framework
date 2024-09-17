#
# Copyright © 2024 Agora
# This file is part of TEN Framework, an open source project.
# Licensed under the Apache License, Version 2.0, with certain conditions.
# Refer to https://github.com/TEN-framework/ten_framework/LICENSE for more
# information.
#
import sys
from build.scripts import cmd_exec


if __name__ == "__main__":
    cmd_exec.run_cmd(sys.argv[1:])
