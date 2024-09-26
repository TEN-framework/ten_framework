#
# Copyright © 2024 Agora
# This file is part of TEN Framework, an open source project.
# Licensed under the Apache License, Version 2.0, with certain conditions.
# Refer to the "LICENSE" file in the root directory for more information.
#
import argparse
import json
import shutil
import sys
import os
from build.scripts import cmd_exec, fs_utils, timestamp_proxy


GCC_ASAN_FLAGS = [
    "-C",
    "linker=gcc",
    "-Z",
    "external-clangrt",
    "-Z",
    "sanitizer=address",
    "-l",
    "asan",
]

CLANG_ASAN_FLAGS = [
    "-C",
    "linker=clang",
    "-C",
    "link-arg=-fuse-ld=lld",
    "-Z",
    "external-clangrt",
    "-Z",
    "sanitizer=address",
    "-C",
    "link-args=-fsanitize=address",
]

ASAN_CONFIG = """[target.{build_target}]
rustflags = {asan_flags}

[build]
target = "{build_target}"
"""


class ArgumentInfo(argparse.Namespace):
    def __init__(self):
        super().__init__()
        self.project_root: str
        self.build_type: str
        self.compiler: str
        self.env: list[str]
        self.target: str
        self.tg_timestamp_proxy_file: str | None = None
        self.enable_asan: bool
        self.gen: bool


def gen_cargo_config(root: str, compiler: str, target: str):
    if not os.path.exists(root):
        raise Exception(f"Project root {root} does not exist.")

    cargo_dir = os.path.join(root, ".cargo")
    if not os.path.exists(cargo_dir):
        os.mkdir(cargo_dir)

    cargo_config = os.path.join(cargo_dir, "config.toml")
    if os.path.exists(cargo_config):
        os.remove(cargo_config)

    flags = ""
    if compiler == "gcc":
        flags = json.dumps(GCC_ASAN_FLAGS)
    else:
        flags = json.dumps(CLANG_ASAN_FLAGS)

    config_content = ASAN_CONFIG.format(build_target=target, asan_flags=flags)

    with open(cargo_config, "w") as f:
        f.write(config_content)


def delete_cargo_config(root: str):
    cargo_dir = os.path.join(root, ".cargo")
    if os.path.exists(cargo_dir):
        shutil.rmtree(cargo_dir)


if __name__ == "__main__":
    parser = argparse.ArgumentParser()

    parser.add_argument("--gen", action="store_true")
    parser.add_argument("--project-root", type=str, required=False)
    parser.add_argument("--compiler", type=str, required=True)
    parser.add_argument("--target", type=str, required=False)
    parser.add_argument(
        "--tg-timestamp-proxy-file", type=str, default="", required=False
    )
    parser.add_argument(
        "--enable-asan", action=argparse.BooleanOptionalAction, default=True
    )

    arg_info = ArgumentInfo()
    args = parser.parse_args(namespace=arg_info)

    if args.gen:
        returncode = 0
        try:
            if args.enable_asan:
                gen_cargo_config(args.project_root, args.compiler, args.target)
            else:
                delete_cargo_config(args.project_root)

            # Success to build the app, update the stamp file to represent this
            # fact.
            timestamp_proxy.touch_timestamp_proxy_file(args.tg_timestamp_proxy_file)

        except Exception as exc:
            returncode = 1
            timestamp_proxy.remove_timestamp_proxy_file(args.tg_timestamp_proxy_file)
            print(exc)

        finally:
            sys.exit(-1 if returncode != 0 else 0)
    else:
        if args.compiler == "gcc":
            flags = [
                GCC_ASAN_FLAGS[i] + GCC_ASAN_FLAGS[i + 1]
                for i in range(0, len(GCC_ASAN_FLAGS) - 1, 2)
            ]
            print(" ".join(flags))
        else:
            flags = [
                CLANG_ASAN_FLAGS[i] + CLANG_ASAN_FLAGS[i + 1]
                for i in range(0, len(CLANG_ASAN_FLAGS) - 1, 2)
            ]
            print(" ".join(flags))
        sys.exit(0)
