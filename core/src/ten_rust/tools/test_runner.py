#
# This file is part of the TEN Framework project.
# See https://github.com/TEN-framework/ten_framework/LICENSE for license
# information.
#
import os
import shutil
import sys
from pathlib import Path
import subprocess
import platform


def main(executable_path, *args):
    print("ten_rust unit_test executable path:", executable_path)
    print("ten_rust unit_test arguments:", args)

    src_path = Path(executable_path).parent

    if platform.system() == "Windows":
        target_filename = "unit_test.exe"
    else:
        target_filename = "unit_test"

    target_path = os.path.join(src_path, target_filename)

    shutil.copy(executable_path, target_path)

    # Prepare the command with arguments passed to the script.
    command = [str(executable_path)] + list(args)
    print("ten_rust unit_test final test command:", command)

    result = subprocess.run(command, capture_output=True, text=True)

    print("STDOUT:", result.stdout)
    print("STDERR:", result.stderr)

    exit(result.returncode)


if __name__ == "__main__":
    main(*sys.argv[1:])
