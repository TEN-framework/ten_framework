#
# This file is part of the TEN Framework project.
# See https://github.com/TEN-framework/ten_framework/LICENSE for license
# information.
#
import argparse
import json
import sys
import os
from build.scripts import cmd_exec, fs_utils, timestamp_proxy


class ArgumentInfo(argparse.Namespace):
    def __init__(self):
        super().__init__()
        self.no_run: bool
        self.project_path: str
        self.build_type: str
        self.target_path: str
        self.env: list[str]
        self.log_level: int
        self.test_output_dir: str
        self.tg_timestamp_proxy_file: str | None = None
        self.integration_test_output_name: str | None = None


def get_crate_test_output_name(log_level: int) -> str:
    cmd = ["cargo", "metadata", "--no-deps", "--format-version", "1"]
    returncode, output = cmd_exec.run_cmd(cmd, log_level)
    if returncode:
        raise Exception(f"Failed to get crate name: {output}")
    
    # Get the last line of the output, as there might be some note or warning
    # messages from cargo.
    output = output.splitlines()[-1]
    metadata = json.loads(output)
    for target in metadata["packages"][0]["targets"]:
        if target["test"]:
            return target["name"]
    
    raise Exception("Failed to get crate name from targets.")


def copy_test_binaries(
    source_dir: str,
    base_name: str,
    output_dir: str,
    target_name: str,
    log_level: int = 0,
):
    if log_level > 0:
        print(
            f"Copying test binary {base_name} from {source_dir} to {output_dir}"
        )

    
    for f in os.listdir(source_dir):
        is_executable = os.access(os.path.join(source_dir, f), os.X_OK)
        if not is_executable:
            continue

        # The file name generated by `cargo build` always has a hash suffix, ex:
        # <library_name>-<hash>.<extension>. We need to remove the hash suffix.

        # We have to deal with the extension of the executable file, as if the
        # `crate-type` is `cdylib`, the extension is `.so` on linux, which is 
        # not what we want. The extension is not present on linux or macos, and
        # the extension is '.exe' on windows.
        extension = os.path.splitext(f)[1]
        if extension != "" and extension != ".exe":
            continue

        binary_name = f[: f.rindex("-") if "-" in f else -1]
        if binary_name != base_name:
            continue

        dest = target_name + extension

        fs_utils.copy_file(
            os.path.join(source_dir, f),
            os.path.join(output_dir, dest),
            True,
        )


if __name__ == "__main__":
    parser = argparse.ArgumentParser()

    parser.add_argument("--no-run", type=bool, default=True)
    parser.add_argument("--project-path", type=str, required=True)
    parser.add_argument("--manifest-path", type=str, required=False)
    parser.add_argument("--build-type", type=str, required=True)
    parser.add_argument("--target-path", type=str, required=True)
    parser.add_argument("--target", type=str, required=True)
    parser.add_argument("--env", type=str, action="append", default=[])
    parser.add_argument("--log-level", type=int, required=True)
    parser.add_argument(
        "--test-output-dir", type=str, default="", required=False
    )
    parser.add_argument(
        "--tg-timestamp-proxy-file", type=str, default="", required=False
    )
    parser.add_argument(
        "--integration-test-output-name", type=str, required=False
    )

    arg_info = ArgumentInfo()
    args = parser.parse_args(namespace=arg_info)

    # Setup environment variables.
    for env in args.env:
        split_key_index = str(env).find("=")
        if split_key_index == -1:
            sys.exit(1)
        else:
            os.environ[(str(env)[:split_key_index])] = str(env)[
                split_key_index + len("=") :
            ]

    origin_wd = os.getcwd()

    returncode = 0
    try:
        os.chdir(args.project_path)

        # cargo build --tests: only compile the test source files, without 
        # running the test cases.
        # cargo test: compile the test source files and run the test cases.
        # 
        # `cargo build --tests` or `cargo test --no-run` will not trigger the
        # `runner` script in .cargo/config.toml.
        if args.no_run:
            cmd = [
                "cargo",
                "build",
                "--target-dir",
                args.target_path,
                "--target",
                args.target,
                "--tests",
            ]
        else:
            cmd = [
                "cargo",
                "test",
                "--target-dir",
                args.target_path,
                "--target",
                args.target,
            ]

        if args.build_type == "release":
            cmd.append("--release")

        returncode, logs = cmd_exec.run_cmd(cmd, args.log_level)
        if returncode:
            raise Exception(f"Failed to build rust tests: {logs}")
        else:
            print(logs)

        # The prefix of the unit test binary is the crate name (i.e., the 
        # 'name' in [[bin]] or [[lib]] in a crate).
        unit_test_output_name = get_crate_test_output_name(args.log_level)

        # The output of the dependencies will be in <target_path>/<build_type>
        # /deps, while the output of the tests will be in <target_path>/<target>
        # /<build_type>/deps.
        if args.test_output_dir != "":
            # Copy the unit test binary.
            copy_test_binaries(
                os.path.join(
                    args.target_path,
                    args.target,
                    args.build_type,
                    "deps",
                ),
                unit_test_output_name,
                args.test_output_dir,
                "unit_test",
                args.log_level,
            )

            if args.integration_test_output_name:
                # Copy the integration test binary.
                copy_test_binaries(
                    os.path.join(
                        args.target_path,
                        args.target,
                        args.build_type,
                        "deps",
                    ),
                    args.integration_test_output_name,
                    args.test_output_dir,
                    "integration_test",
                    args.log_level,
                )

        # Success to build the app, update the stamp file to represent this
        # fact.
        timestamp_proxy.touch_timestamp_proxy_file(args.tg_timestamp_proxy_file)

    except Exception as exc:
        returncode = 1
        timestamp_proxy.remove_timestamp_proxy_file(
            args.tg_timestamp_proxy_file
        )
        print(exc)

    finally:
        os.chdir(origin_wd)
        sys.exit(-1 if returncode != 0 else 0)
