"""
Test cpp_app_nodejs.
"""

import subprocess
import os
import sys
from sys import stdout
from .utils import http, build_config, build_pkg, fs_utils


def http_request():
    return http.post(
        "http://127.0.0.1:8002/",
        {
            "_ten": {
                "name": "test",
            },
        },
    )


def test_cpp_app_nodejs():
    """Test client and app server."""
    base_path = os.path.dirname(os.path.abspath(__file__))
    root_dir = os.path.join(base_path, "../../../../../")

    my_env = os.environ.copy()

    app_dir_name = "cpp_app_nodejs_app"
    app_root_path = os.path.join(base_path, app_dir_name)
    app_language = "cpp"

    fs_utils.remove_tree(app_root_path)

    build_config_args = build_config.parse_build_config(
        os.path.join(root_dir, "tgn_args.txt"),
    )

    if build_config_args.ten_enable_integration_tests_prebuilt is False:
        print(f'Assembling and building package "{app_dir_name}".')

        rc = build_pkg.prepare_and_build_app(
            build_config_args,
            root_dir,
            base_path,
            app_dir_name,
            app_language,
        )
        if rc != 0:
            assert False, "Failed to build package."

        tman_install_cmd = [
            os.path.join(root_dir, "ten_manager/bin/tman"),
            "--config-file",
            os.path.join(root_dir, "tests/local_registry/config.json"),
            "--yes",
            "install",
        ]

        tman_install_process = subprocess.Popen(
            tman_install_cmd,
            stdout=stdout,
            stderr=subprocess.STDOUT,
            env=my_env,
            cwd=app_root_path,
        )
        tman_install_process.wait()
        return_code = tman_install_process.returncode
        if return_code != 0:
            assert False, "Failed to install package."

        # Compile typescript extensions.
        rc = build_pkg.build_nodejs_extensions(app_root_path)
        if rc != 0:
            assert False, "Failed to build TypeScript extensions."

    if sys.platform == "linux":
        if (
            build_config_args.enable_sanitizer
            and not build_config_args.is_clang
        ):
            libasan_path = os.path.join(
                base_path,
                (
                    "cpp_app_nodejs_app/ten_packages/system/"
                    "ten_runtime/lib/libasan.so"
                ),
            )

            if os.path.exists(libasan_path):
                print("Using AddressSanitizer library.")
                my_env["LD_PRELOAD"] = libasan_path

    server_cmd = os.path.join(base_path, "cpp_app_nodejs_app/bin/start")

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

    is_started = http.is_app_started("127.0.0.1", 8002, 30)
    if not is_started:
        print("The cpp_app_nodejs is not started after 10 seconds.")

        server.kill()
        exit_code = server.wait()
        print("The exit code of cpp_app_nodejs: ", exit_code)

        assert exit_code == 0
        assert False

        return

    try:
        resp = http_request()
        assert resp != 500
        print(resp)

    finally:
        is_stopped = http.stop_app("127.0.0.1", 8002, 30)
        if not is_stopped:
            print("The cpp_app_nodejs can not stop after 30 seconds.")
            server.kill()

        exit_code = server.wait()
        print("The exit code of cpp_app_nodejs: ", exit_code)

        assert exit_code == 0

        if build_config_args.ten_enable_integration_tests_prebuilt is False:
            # Testing complete. If builds are only created during the testing
            # phase, we  can clear the build results to save disk space.
            fs_utils.remove_tree(app_root_path)
