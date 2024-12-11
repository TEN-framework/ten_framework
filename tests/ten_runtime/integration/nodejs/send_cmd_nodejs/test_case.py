"""
Test send_cmd_nodejs.
"""

import subprocess
import os
import sys
from sys import stdout
from .common import http, build_config, build_pkg


def test_send_cmd_nodejs():
    """Test client and app server."""
    base_path = os.path.dirname(os.path.abspath(__file__))
    root_dir = os.path.join(base_path, "../../../../../")

    my_env = os.environ.copy()

    source_pkg_name = "send_cmd_nodejs_app"
    app_root_path = os.path.join(base_path, source_pkg_name)
    app_language = "nodejs"

    build_config_args = build_config.parse_build_config(
        os.path.join(root_dir, "tgn_args.txt"),
    )

    if build_config_args.ten_enable_integration_tests_prebuilt is False:
        print('Assembling and building package "{}".'.format(source_pkg_name))

        rc = build_pkg.prepare_and_build_app(
            build_config_args,
            root_dir,
            base_path,
            app_root_path,
            source_pkg_name,
            app_language,
        )
        if rc != 0:
            assert False, "Failed to build package."

    if sys.platform == "linux":
        if build_config_args.enable_sanitizer:
            libasan_path = os.path.join(
                base_path,
                "send_cmd_nodejs_app/ten_packages/system/ten_runtime/lib/libasan.so",
            )

            if os.path.exists(libasan_path):
                my_env["LD_PRELOAD"] = libasan_path

    server_cmd = os.path.join(base_path, "send_cmd_nodejs_app/bin/start")

    if not os.path.isfile(server_cmd):
        print(f"Server command '{server_cmd}' does not exist.")
        assert False

    server = subprocess.Popen(
        server_cmd,
        stdout=stdout,
        stderr=subprocess.STDOUT,
        env=my_env,
        cwd=app_root_path,
    )

    exit_code = server.wait()
    print("The exit code of send_cmd_nodejs: ", exit_code)
