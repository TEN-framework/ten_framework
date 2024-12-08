#
# Copyright © 2024 Agora
# This file is part of TEN Framework, an open source project.
# Licensed under the Apache License, Version 2.0, with certain conditions.
# Refer to the "LICENSE" file in the root directory for more information.
#
import argparse
import os
import sys
import sysconfig
from build.scripts import cmd_exec


class ArgumentInfo(argparse.Namespace):
    def __init__(self):
        super().__init__()
        self.python_version: str
        self.target_os: str
        self.config_type: str
        self.log_level: int


def get_embed_flags():
    config = sysconfig.get_config_vars()

    # Include directories.
    include_dirs = config.get("INCLUDEPY", "")

    # Library directories.
    lib_dirs = config.get("LIBDIR", "")
    # On some builds, you might need to add 'LIBPL' or similar.
    lib_dirs_extra = config.get("LIBPL", "")
    if lib_dirs_extra:
        lib_dirs += f" {lib_dirs_extra}"

    # Libraries.
    libs = config.get("LIBS", "")
    # Include the Python library.
    python_lib = config.get("LIBRARY", "").replace("lib", "").replace(".a", "")
    if python_lib:
        libs += f" {python_lib}"

    # C Flags.
    cflags = config.get("CFLAGS", "")

    return {
        "include_dirs": include_dirs,
        "lib_dirs": lib_dirs,
        "libs": libs.strip(),
        "cflags": cflags.strip(),
    }


def transform_flags_for_windows(embed_flags):
    transformed = {"include_dirs": [], "lib_dirs": [], "libs": [], "cflags": []}

    # Transform include directories.
    for include_dir in embed_flags["include_dirs"].split():
        transformed["include_dirs"].append(f'/I"{include_dir}"')

    # Transform library directories.
    for lib_dir in embed_flags["lib_dirs"].split():
        transformed["lib_dirs"].append(f'/LIBPATH:"{lib_dir}"')

    # Transform libraries.
    for lib in embed_flags["libs"].split():
        # Remove '-l' if present.
        if lib.startswith("-l"):
            lib = lib[2:]
        # Append .lib extension if not present.
        if not lib.endswith(".lib"):
            lib = f'"{lib}.lib"'
        else:
            lib = f'"{lib}"'
        transformed["libs"].append(lib)

    # Transform CFLAGS.
    transformed["cflags"] = embed_flags["cflags"].split()

    return transformed


def get_config_flags(config_type: str):
    config = sysconfig.get_config_vars()

    if config_type == "cflags":
        return config.get("CFLAGS", "")
    elif config_type == "ldflags":
        return config.get("LDFLAGS", "")
    elif config_type == "libs":
        libs = config.get("LIBS", "")
        # Remove '-l' prefixes if necessary.
        libs = libs.replace("-l", "")
        return libs
    else:
        raise ValueError(f"Unknown config_type: {config_type}")


def build_cmd(config_type):
    if config_type in ["cflags", "ldflags", "libs"]:
        flag = get_config_flags(config_type)
        cmd = flag.split()
        return cmd
    else:
        raise ValueError(f"Unknown config_type: {config_type}")


if __name__ == "__main__":
    parser = argparse.ArgumentParser()

    parser.add_argument("--python-version", type=str, required=True)
    parser.add_argument("--target-os", type=str, required=True)
    parser.add_argument("--config-type", type=str, required=True)
    parser.add_argument("--log-level", type=int, required=True)

    arg_info = ArgumentInfo()
    args = parser.parse_args(namespace=arg_info)

    returncode = 0

    try:
        if args.target_os == "win":
            # Retrieve embed flags using sysconfig.
            embed_flags = get_embed_flags()
            transformed_flags = transform_flags_for_windows(embed_flags)

            if args.config_type == "cflags":
                # Print include directories and any additional CFLAGS.
                for flag in transformed_flags["include_dirs"]:
                    print(flag)
                for flag in transformed_flags["cflags"]:
                    print(flag)
            elif args.config_type == "ldflags":
                # Print library directories.
                for lib_dir in transformed_flags["lib_dirs"]:
                    print(lib_dir)
            elif args.config_type == "libs":
                # Print libraries.
                for lib in transformed_flags["libs"]:
                    print(lib)
            else:
                raise Exception(f"Unknown config_type: {args.config_type}")
        else:
            if args.config_type == "cflags":
                cmd = [
                    f"python{args.python_version}-config",
                    "--embed",
                    "--cflags",
                ]
            elif args.config_type == "ldflags":
                cmd = [
                    f"python{args.python_version}-config",
                    "--embed",
                    "--ldflags",
                ]
            elif args.config_type == "libs":
                cmd = [
                    f"python{args.python_version}-config",
                    "--embed",
                    "--libs",
                    "| sed 's/-l//g'",  # remove -l prefix
                ]
            else:
                raise Exception(f"Unknown option: {args.config_type}")

            returncode, output_text = cmd_exec.run_cmd(cmd, args.log_level)
            if returncode:
                raise Exception("Failed to run python-config.")

            outputs = output_text.split()
            for output in outputs:
                print(output)

    except Exception as exc:
        print(exc)

    finally:
        sys.exit(-1 if returncode != 0 else 0)
