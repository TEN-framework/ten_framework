"""
Test standalone_test_python.
"""

import subprocess
import os
import sys
from sys import stdout


def test_standalone_test_python():
    base_path = os.path.dirname(os.path.abspath(__file__))
    root_dir = os.path.join(base_path, "../../../../../")

    my_env = os.environ.copy()

    extension_root_path = os.path.join(base_path, "default_extension_python")

    # Step 1:
    #
    # Standalone testing involves the use of ten_runtime_python, so use tman
    # install to install the ten_runtime_python system package.
    tman_install_cmd = [
        os.path.join(root_dir, "ten_manager/bin/tman"),
        "--config-file",
        os.path.join(root_dir, "tests/local_registry/config.json"),
        "install",
    ]

    tman_install_process = subprocess.Popen(
        tman_install_cmd,
        stdout=stdout,
        stderr=subprocess.STDOUT,
        env=my_env,
        cwd=extension_root_path,
    )
    tman_install_rc = tman_install_process.wait()
    assert tman_install_rc == 0

    # Step 2:
    #
    # Run standalone test cases.

    # Set the required environment variables for the test.
    my_env["PYTHONMALLOC"] = "malloc"
    my_env["PYTHONDEVMODE"] = "1"

    if sys.platform == "linux":
        if os.path.exists(os.path.join(base_path, "use_asan_lib_marker")):
            libasan_path = os.path.join(
                base_path,
                (
                    "default_extension_python/.ten/app/ten_packages/"
                    "system/ten_runtime/lib/libasan.so"
                ),
            )

            if os.path.exists(libasan_path):
                my_env["LD_PRELOAD"] = libasan_path

    tester_process = subprocess.Popen(
        "tests/bin/start",
        stdout=stdout,
        stderr=subprocess.STDOUT,
        env=my_env,
        cwd=extension_root_path,
    )
    tester_rc = tester_process.wait()
    assert tester_rc == 0
