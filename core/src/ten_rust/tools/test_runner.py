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


def copy_executable_to_predefined_name(executable_path: str):
    executable_dirname = Path(executable_path).parent
    executable_filename = Path(executable_path).name

    # Determine if it's a unit test or an integration test based on the filename
    # prefix
    if executable_filename.startswith("integration_test"):
        test_type = "integration_test"
    elif executable_filename.startswith("ten_rust"):
        test_type = "unit_test"
    else:
        return

    if platform.system() == "Windows":
        predefined_filename = f"{test_type}.exe"
    else:
        predefined_filename = test_type

    target_path = os.path.join(executable_dirname, predefined_filename)
    shutil.copy(executable_path, target_path)


def main(executable_path: str, *args):
    copy_executable_to_predefined_name(executable_path)

    # Prepare the command with arguments passed to the script.
    command = [str(executable_path)] + list(args)
    print("ten_rust test command:", command)

    result = subprocess.run(command, capture_output=True, text=True)

    print("STDOUT:", result.stdout)
    print("STDERR:", result.stderr)

    exit(result.returncode)


if __name__ == "__main__":
    main(*sys.argv[1:])
