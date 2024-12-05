#
# Copyright © 2024 Agora
# This file is part of TEN Framework, an open source project.
# Licensed under the Apache License, Version 2.0, with certain conditions.
# Refer to the "LICENSE" file in the root directory for more information.
#
import argparse
import json
import sys
import os
from build.scripts import fs_utils

assemble_info_dir = ".assemble_info"


class ArgumentInfo(argparse.Namespace):
    def __init__(self):
        self.test_case_out_dir: str
        self.test_case_src_dir: str
        self.src_app: str
        self.src_app_language: str
        self.generated_app_src_root_dir_name: str
        self.replace_files_after_install_app: list[str]
        self.replace_files_after_install_all: list[str]


def dump_info_json(args: ArgumentInfo):
    info = {
        "test_case_src_dir": args.test_case_src_dir,
        "src_app": args.src_app,
        "src_app_language": args.src_app_language,
        "generated_app_src_root_dir_name": args.generated_app_src_root_dir_name,
        "replace_files_after_install_app": args.replace_files_after_install_app,
        "replace_files_after_install_all": args.replace_files_after_install_all,
    }

    resource_dir = os.path.join(
        args.test_case_out_dir,
        assemble_info_dir,
        args.generated_app_src_root_dir_name,
    )
    info_file = os.path.join(resource_dir, "info.json")
    if not os.path.exists(resource_dir):
        os.makedirs(resource_dir, exist_ok=True)

    with open(info_file, "w") as f:
        f.write(json.dumps(info, ensure_ascii=False))


def copy_files_to_be_replaced_after_install_app(args: ArgumentInfo):
    if args.replace_files_after_install_app is None:
        return

    out_dir = os.path.join(
        args.test_case_out_dir,
        assemble_info_dir,
        args.generated_app_src_root_dir_name,
        "files_to_be_replaced_after_install_app",
    )

    for file in args.replace_files_after_install_app:
        src_file = os.path.join(args.test_case_src_dir, file)
        dst_file = os.path.join(out_dir, file)
        if not os.path.exists(src_file):
            print(f"File {src_file} does not exist")
            continue

        if not os.path.exists(os.path.dirname(dst_file)):
            os.makedirs(os.path.dirname(dst_file), exist_ok=True)

        fs_utils.copy(src_file, dst_file)


def copy_files_to_be_replaced_after_install_all(args: ArgumentInfo):
    if args.replace_files_after_install_all is None:
        return

    out_dir = os.path.join(
        args.test_case_out_dir,
        assemble_info_dir,
        args.generated_app_src_root_dir_name,
        "files_to_be_replaced_after_install_all",
    )

    for file in args.replace_files_after_install_all:
        src_file = os.path.join(args.test_case_src_dir, file)
        dst_file = os.path.join(out_dir, file)
        if not os.path.exists(src_file):
            print(f"File {src_file} does not exist")
            continue

        if not os.path.exists(os.path.dirname(dst_file)):
            os.makedirs(os.path.dirname(dst_file), exist_ok=True)

        fs_utils.copy(src_file, dst_file)


if __name__ == "__main__":
    parser = argparse.ArgumentParser()

    parser.add_argument(
        "--test-case-out-dir",
        type=str,
        required=True,
        help="Path to the test case output directory",
    )

    parser.add_argument(
        "--test-case-src-dir",
        type=str,
        required=True,
        help="Path to the test case source directory",
    )

    parser.add_argument(
        "--src-app",
        type=str,
        required=True,
        help="Path to the app source code",
    )

    parser.add_argument(
        "--src-app-language",
        type=str,
        required=True,
        help="Language of the app source code",
    )

    parser.add_argument(
        "--generated-app-src-root-dir-name",
        type=str,
        required=True,
        help="Name of the generated app source root directory",
    )

    parser.add_argument(
        "--replace-files-after-install-app",
        type=str,
        action="append",
        help="List of files to replace after installing app",
    )

    parser.add_argument(
        "--replace-files-after-install-all",
        type=str,
        action="append",
        help="List of files to replace after installing all",
    )

    arg_info = ArgumentInfo()
    args = parser.parse_args(namespace=arg_info)

    # dump the full json to root_dir/.assembly_resources/info.json
    dump_info_json(args)

    copy_files_to_be_replaced_after_install_app(args)

    copy_files_to_be_replaced_after_install_all(args)

    sys.exit(0)
