"""
Test large_json_python.
"""

import subprocess
import os
import sys
import pytest
from sys import stdout
from .common import http, build_config, build_pkg


@pytest.fixture(scope="function")
def manage_env_var_for_memory_issue_detection():
    original_ten_enable_memory_sanitizer = os.environ.get(
        "TEN_ENABLE_MEMORY_TRACKING"
    )
    original_asan_options = os.environ.get("ASAN_OPTIONS")

    # Set the environment variable before the test.

    # TODO(xilin): The TEN Python binding has a known memory leak. After
    # resolving it, enable TEN_ENABLE_MEMORY_TRACKING=true.
    os.environ["TEN_ENABLE_MEMORY_TRACKING"] = "false"

    os.environ["ASAN_OPTIONS"] = "detect_leaks=0"

    yield

    # Remove the environment variable after the test.
    if original_ten_enable_memory_sanitizer is not None:
        os.environ["TEN_ENABLE_MEMORY_TRACKING"] = (
            original_ten_enable_memory_sanitizer
        )
    else:
        del os.environ["TEN_ENABLE_MEMORY_TRACKING"]

    if original_asan_options is not None:
        os.environ["ASAN_OPTIONS"] = original_asan_options
    else:
        del os.environ["ASAN_OPTIONS"]


def http_request():
    return http.post(
        "http://127.0.0.1:8002/",
        {
            "_ten": {
                "name": "test",
            },
        },
    )


@pytest.mark.usefixtures("manage_env_var_for_memory_issue_detection")
def test_large_json_python():
    """Test client and app server."""
    base_path = os.path.dirname(os.path.abspath(__file__))
    root_dir = os.path.join(base_path, "../../../../../")

    # Create virtual environment.
    venv_dir = os.path.join(base_path, "venv")
    subprocess.run([sys.executable, "-m", "venv", venv_dir])

    my_env = os.environ.copy()

    # Set the required environment variables for the test.
    my_env["PYTHONMALLOC"] = "malloc"
    my_env["PYTHONDEVMODE"] = "1"

    # Launch virtual environment.
    my_env["VIRTUAL_ENV"] = venv_dir
    my_env["PATH"] = os.path.join(venv_dir, "bin") + os.pathsep + my_env["PATH"]

    if sys.platform == "win32":
        print("test_large_json_python doesn't support win32")
        assert False
    elif sys.platform == "darwin":
        # client depends on some libraries in the TEN app.
        my_env["DYLD_LIBRARY_PATH"] = os.path.join(
            base_path, "large_json_python_app/lib"
        )
    else:
        # client depends on some libraries in the TEN app.
        my_env["LD_LIBRARY_PATH"] = os.path.join(
            base_path, "large_json_python_app/lib"
        )

    source_pkg_name = "large_json_python_app"
    app_root_path = os.path.join(base_path, source_pkg_name)
    app_language = "python"

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
    else:
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
            cwd=app_root_path,
        )
        tman_install_process.wait()
        return_code = tman_install_process.returncode
        if return_code != 0:
            assert False, "Failed to install package."

    bootstrap_cmd = os.path.join(
        base_path, "large_json_python_app/bin/bootstrap"
    )

    bootstrap_process = subprocess.Popen(
        bootstrap_cmd, stdout=stdout, stderr=subprocess.STDOUT, env=my_env
    )
    bootstrap_process.wait()

    if sys.platform == "linux":
        if build_config_args.enable_sanitizer:
            libasan_path = os.path.join(
                base_path,
                "large_json_python_app/ten_packages/system/ten_runtime/lib/libasan.so",
            )

            if os.path.exists(libasan_path):
                my_env["LD_PRELOAD"] = libasan_path

    server_cmd = os.path.join(base_path, "large_json_python_app/bin/start")

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
        print("The large_json_python is not started after 10 seconds.")

        server.kill()
        exit_code = server.wait()
        print("The exit code of large_json_python: ", exit_code)

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
            print("The large_json_python can not stop after 30 seconds.")
            server.kill()

        exit_code = server.wait()
        print("The exit code of large_json_python: ", exit_code)

        assert exit_code == 0

        if build_config_args.ten_enable_integration_tests_prebuilt is False:
            source_root_path = os.path.join(base_path, source_pkg_name)

            # Testing complete. If builds are only created during the testing
            # phase, we  can clear the build results to save disk space.
            build_pkg.cleanup(source_root_path, app_root_path)
