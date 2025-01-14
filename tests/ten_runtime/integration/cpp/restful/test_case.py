"""
Test restful_http_app.
"""

import subprocess
import os
import sys
from sys import stdout
from .common import http, build_config, build_pkg


def run_test_post():
    resp = http.post(
        "http://127.0.0.1:8001/",
        {"_ten": {"name": "hello_world"}},
    )

    assert resp == '"hello_world, too"'


def test_restful_http_app():
    """Test app server."""
    base_path = os.path.dirname(os.path.abspath(__file__))
    root_dir = os.path.join(base_path, "../../../../../")

    my_env = os.environ.copy()

    if sys.platform == "win32":
        my_env["PATH"] = (
            os.path.join(
                base_path,
                "restful_app/ten_packages/system/ten_runtime/lib",
            )
            + ";"
            + my_env["PATH"]
        )
        server_cmd = os.path.join(base_path, "restful_app/bin/restful_app.exe")
    elif sys.platform == "darwin":
        server_cmd = os.path.join(base_path, "restful_app/bin/restful_app")
    else:
        server_cmd = os.path.join(base_path, "restful_app/bin/restful_app")

    app_root_path = os.path.join(base_path, "restful_app")
    app_dir_name = "restful_app"
    app_language = "cpp"

    build_config_args = build_config.parse_build_config(
        os.path.join(root_dir, "tgn_args.txt"),
    )

    if build_config_args.ten_enable_integration_tests_prebuilt is False:
        print('Assembling and building package "{}".'.format(app_dir_name))

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

    is_started = http.is_app_started("127.0.0.1", 8001, 30)
    if not is_started:
        print("The restful_app is not started after 10 seconds.")

        server.kill()
        exit_code = server.wait()
        print("The exit code of restful_app: ", exit_code)

        assert exit_code == 0
        assert False

        return

    try:
        run_test_post()

    finally:
        is_stopped = http.stop_app("127.0.0.1", 8001, 30)

        if not is_stopped:
            print("The restful_app can not stop after 30 seconds.")
            server.kill()

        exit_code = server.wait()
        print("The exit code of restful_app: ", exit_code)

        assert exit_code == 0

        if build_config_args.ten_enable_integration_tests_prebuilt is False:
            source_root_path = os.path.join(base_path, app_dir_name)
            # Testing complete. If builds are only created during the testing
            # phase, we can clear the build results to save disk space.
            build_pkg.cleanup(source_root_path, app_root_path)
