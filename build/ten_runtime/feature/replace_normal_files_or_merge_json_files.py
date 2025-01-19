#
# Copyright © 2025 Agora
# This file is part of TEN Framework, an open source project.
# Licensed under the Apache License, Version 2.0, with certain conditions.
# Refer to the "LICENSE" file in the root directory for more information.
#
import argparse
import json
import os
import sys
from build.scripts import fs_utils


class ArgumentInfo(argparse.Namespace):
    def __init__(self):
        self.src: str
        self.dest: str


def merge_json_file_at_root_level(src: str, dest: str) -> None:
    if not os.path.exists(src):
        raise Exception(f"{src} does not exist.")
    if not os.path.exists(dest):
        raise Exception(f"{dest} does not exist.")

    with open(src, "r") as src_file, open(dest, "r+") as dest_file:
        try:
            src_file_content = json.load(src_file)
        except Exception as e:
            print(f"{src_file} contains invalid JSON: {e}")
            raise e

        try:
            dest_file_content = json.load(dest_file)
        except Exception as e:
            print(f"{dest_file} contains invalid JSON: {e}")
            raise e

        for key, value in src_file_content.items():
            dest_file_content[key] = value

        dest_file.seek(0)
        json.dump(dest_file_content, dest_file, indent=2)
        dest_file.truncate()


def replace_normal_files_or_merge_json_files(src: str, dest: str) -> None:
    if not os.path.exists(src):
        raise Exception(f"{src} does not exist.")

    if os.path.isdir(src):
        raise Exception(f"{src} is a directory, not a file.")

    if os.path.exists(dest) and os.path.isdir(dest):
        raise Exception(f"{dest} is a directory, not a file.")

    if (
        src.endswith(".json")
        and dest.endswith(".json")
        and os.path.exists(dest)
    ):
        merge_json_file_at_root_level(src, dest)
    else:
        fs_utils.copy(src, dest)


if __name__ == "__main__":
    parser = argparse.ArgumentParser()

    parser.add_argument("--src", type=str, required=True)
    parser.add_argument("--dest", type=str, required=True)

    arg_info = ArgumentInfo()
    args = parser.parse_args(namespace=arg_info)

    origin_wd = os.getcwd()
    returncode = 0

    try:
        replace_normal_files_or_merge_json_files(args.src, args.dest)
    except Exception as exc:
        returncode = 1
        print(exc)
    finally:
        os.chdir(origin_wd)
        sys.exit(returncode)
