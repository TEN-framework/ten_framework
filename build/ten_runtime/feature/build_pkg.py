#
# Copyright © 2024 Agora
# This file is part of TEN Framework, an open source project.
# Licensed under the Apache License, Version 2.0, with certain conditions.
# Refer to the "LICENSE" file in the root directory for more information.
#
import argparse
import json
import os
import sys
import platform
from datetime import datetime
from build.scripts import fs_utils, cmd_exec, timestamp_proxy


class ArgumentInfo(argparse.Namespace):
    def __init__(self):
        self.pkg_src_root_dir: str
        self.pkg_run_root_dir: str
        self.tg_timestamp_proxy_file: str
        self.pkg_name: str
        self.pkg_language: str
        self.os: str
        self.cpu: str
        self.build: str
        self.is_clang: bool
        self.enable_sanitizer: bool
        self.vs_version: str
        self.log_level: int


def construct_extra_args_for_cpp_ag(args: ArgumentInfo) -> list[str]:
    cmd = ["--"]

    if args.is_clang is True:
        cmd += ["is_clang=true"]
    else:
        cmd += ["is_clang=false"]

    if args.enable_sanitizer is True:
        cmd += ["enable_sanitizer=true"]
    else:
        cmd += ["enable_sanitizer=false"]

    if args.vs_version:
        cmd += [f"vs_version={args.vs_version}"]

    return cmd


def build_cpp_app(args: ArgumentInfo) -> int:
    # tgn gen ...
    cmd = ["tgn", "gen", args.os, args.cpu, args.build]
    cmd += construct_extra_args_for_cpp_ag(args)

    returncode, output = cmd_exec.run_cmd(cmd, args.log_level)

    if returncode:
        print(f"Failed to build c++ app: {output}")
        return 1

    # tgn build ...
    cmd = ["tgn", "build", args.os, args.cpu, args.build]

    t1 = datetime.now()
    returncode, output = cmd_exec.run_cmd(cmd, args.log_level)
    t2 = datetime.now()

    duration = (t2 - t1).seconds

    if args.log_level > 0:
        print(f"Build c++ app({args.pkg_name}) costs {duration} seconds.")

    if returncode:
        print(f"Failed to build c++ app: {output}")
        return 1

    # Copy the build result to the specified run folder.
    fs_utils.copy(
        f"{args.pkg_src_root_dir}/out/{args.os}/{args.cpu}/app/{args.pkg_name}",
        args.pkg_run_root_dir,
        True,
    )

    return returncode


def build_cpp_extension(args: ArgumentInfo) -> int:
    # tgn gen ...
    cmd = ["tgn", "gen", args.os, args.cpu, args.build]
    cmd += construct_extra_args_for_cpp_ag(args)

    returncode, output = cmd_exec.run_cmd(cmd, args.log_level)

    if returncode:
        print(f"Failed to build c++ extension: {output}")
        return 1

    # tgn build ...
    cmd = ["tgn", "build", args.os, args.cpu, args.build]

    returncode, output = cmd_exec.run_cmd(cmd, args.log_level)

    if returncode:
        print(f"Failed to build c++ extension: {output}")
        return 1

    return returncode


def is_mac_arm64() -> bool:
    return (
        platform.system().lower() == "darwin"
        and platform.machine().lower() == "arm64"
    )


def build_go_app(args: ArgumentInfo) -> int:
    # Determine the path to the main.go script. Some cases require a customized
    # Go build script, but in most situations, the build script provided by the
    # TEN runtime Go binding can be used directly.
    main_go_path = "scripts/build/main.go"
    if not os.path.exists(main_go_path):
        main_go_path = "ten_packages/system/ten_runtime_go/tools/build/main.go"

    cmd = ["go", "run", main_go_path]
    if args.log_level > 0:
        cmd += ["--verbose"]

    # `-asan` is not supported by go compiler on darwin/arm64.
    if args.enable_sanitizer and not is_mac_arm64():
        cmd += ["-asan"]

    envs = os.environ.copy()
    if args.is_clang:
        envs["CC"] = "clang"
        envs["CXX"] = "clang++"
    else:
        envs["CC"] = "gcc"
        envs["CXX"] = "g++"

    t1 = datetime.now()
    returncode, output = cmd_exec.run_cmd_realtime(
        cmd, None, envs, args.log_level
    )
    t2 = datetime.now()

    duration = (t2 - t1).seconds

    if args.log_level > 0:
        print(f"Build go app({args.pkg_name}) costs {duration} seconds.")

    if returncode:
        print(f"Failed to build go app: {output}")
        return 1

    return returncode


def get_pkg_type(pkg_root: str) -> str:
    manifest_path = os.path.join(pkg_root, "manifest.json")
    with open(manifest_path, "r") as f:
        manifest_json = json.load(f)
    return manifest_json["type"]


def build_app(args: ArgumentInfo) -> int:
    returncode = 0

    if args.pkg_language == "cpp":
        returncode = build_cpp_app(args)
    elif args.pkg_language == "go":
        returncode = build_go_app(args)
    elif args.pkg_language == "python":
        returncode = 0
    else:
        print(f"Unknown app language: {args.pkg_language}")
        returncode = 1

    return returncode


def build_extension(args: ArgumentInfo) -> int:
    returncode = 0

    if args.pkg_language == "cpp":
        returncode = build_cpp_extension(args)
    else:
        returncode = 0

    return returncode


def build(args: ArgumentInfo) -> int:
    if args.log_level > 0:
        msg = (
            f"> Start to build package({args.pkg_name})"
            f" in {args.pkg_src_root_dir}"
        )
        print(msg)

    origin_wd = os.getcwd()
    returncode = 0

    try:
        os.chdir(args.pkg_src_root_dir)
        pkg_type = get_pkg_type(args.pkg_src_root_dir)
        if pkg_type == "app":
            returncode = build_app(args)
        elif pkg_type == "extension":
            returncode = build_extension(args)
        else:
            returncode = 0

        if returncode > 0:
            print(f"Failed to build {pkg_type}({args.pkg_name})")
            return 1

        # Success to build the app, update the stamp file to represent this
        # fact.
        if args.tg_timestamp_proxy_file:
            timestamp_proxy.touch_timestamp_proxy_file(
                args.tg_timestamp_proxy_file
            )

        if args.log_level > 0:
            print(f"Build {pkg_type}({args.pkg_name}) success")

    except Exception as e:
        returncode = 1
        if args.tg_timestamp_proxy_file:
            timestamp_proxy.remove_timestamp_proxy_file(
                args.tg_timestamp_proxy_file
            )
        print(f"Build package({args.pkg_name}) failed: {repr(e)}")

    finally:
        os.chdir(origin_wd)

    return returncode


if __name__ == "__main__":
    parser = argparse.ArgumentParser()

    parser.add_argument("--pkg-src-root-dir", type=str, required=True)
    parser.add_argument("--pkg-run-root-dir", type=str, required=False)
    parser.add_argument("--tg-timestamp-proxy-file", type=str, required=False)
    parser.add_argument("--pkg-name", type=str, required=True)
    parser.add_argument("--pkg-language", type=str, required=True)
    parser.add_argument("--os", type=str, required=True)
    parser.add_argument("--cpu", type=str, required=True)
    parser.add_argument("--build", type=str, required=True)
    parser.add_argument("--is-clang", action=argparse.BooleanOptionalAction)
    parser.add_argument(
        "--enable-sanitizer", action=argparse.BooleanOptionalAction
    )
    parser.add_argument("--vs-version", type=str, required=False)
    parser.add_argument(
        "--log-level", type=int, required=True, help="specify log level"
    )

    arg_info = ArgumentInfo()
    args = parser.parse_args(namespace=arg_info)

    return_code = build(arg_info)
    sys.exit(return_code)
