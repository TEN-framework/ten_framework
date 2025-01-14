"""
Test frequently_cgo_call_go.
"""

import subprocess
import os
import sys
from sys import stdout
from .common import msgpack, build_config, build_pkg


def test_frequently_cgo_call_go():
    """Test client and app server."""
    base_path = os.path.dirname(os.path.abspath(__file__))
    root_dir = os.path.join(base_path, "../../../../../")

    my_env = os.environ.copy()

    if sys.platform == "win32":
        print("test_frequently_cgo_call_go doesn't support win32")
        assert False
    elif sys.platform == "darwin":
        # client depends on some libraries in the TEN app.
        my_env["DYLD_LIBRARY_PATH"] = os.path.join(
            base_path,
            "frequently_cgo_call_go_app/ten_packages/system/ten_runtime/lib",
        )
    else:
        # client depends on some libraries in the TEN app.
        my_env["LD_LIBRARY_PATH"] = os.path.join(
            base_path,
            "frequently_cgo_call_go_app/ten_packages/system/ten_runtime/lib",
        )

    app_dir_name = "frequently_cgo_call_go_app"
    app_root_path = os.path.join(base_path, app_dir_name)
    app_language = "go"

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

    server_cmd = os.path.join(base_path, "frequently_cgo_call_go_app/bin/start")
    client_cmd = os.path.join(base_path, "frequently_cgo_call_go_app_client")

    if not os.path.isfile(server_cmd):
        print(f"Server command '{server_cmd}' does not exist.")
        assert False

    if not os.path.isfile(client_cmd):
        print(f"Client command '{client_cmd}' does not exist.")
        assert False

    server = subprocess.Popen(
        server_cmd,
        stdout=stdout,
        stderr=subprocess.STDOUT,
        env=my_env,
        cwd=app_root_path,
    )

    is_started, sock = msgpack.is_app_started("127.0.0.1", 8007, 10)
    if not is_started:
        print("The frequently_cgo_call_go is not started after 10 seconds.")

        server.kill()
        exit_code = server.wait()
        print("The exit code of frequently_cgo_call_go: ", exit_code)

        assert exit_code == 0
        assert False

        return

    client = subprocess.Popen(
        client_cmd, stdout=stdout, stderr=subprocess.STDOUT, env=my_env
    )

    client_rc = client.wait()
    if client_rc != 0:
        server.kill()

    sock.close()

    server_rc = server.wait()
    print("server: ", server_rc)
    print("client: ", client_rc)
    assert server_rc == 0
    assert client_rc == 0

    if build_config_args.ten_enable_integration_tests_prebuilt is False:
        source_root_path = os.path.join(base_path, app_dir_name)

        # Testing complete. If builds are only created during the testing phase,
        # we can clear the build results to save disk space.
        build_pkg.cleanup(source_root_path, app_root_path)
