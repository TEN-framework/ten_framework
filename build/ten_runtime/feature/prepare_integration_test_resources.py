#
# Copyright © 2025 Agora
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
        self.replace_paths_after_install_app: list[str]
        self.replace_paths_after_install_all: list[str]


def dump_integration_test_preparation_info_json(args: ArgumentInfo):
    info = {
        "test_case_src_dir": args.test_case_src_dir,
        "src_app": args.src_app,
        "src_app_language": args.src_app_language,
        "generated_app_src_root_dir_name": args.generated_app_src_root_dir_name,
        "replace_paths_after_install_app": args.replace_paths_after_install_app,
        "replace_paths_after_install_all": args.replace_paths_after_install_all,
    }

    resource_dir = os.path.join(
        args.test_case_out_dir,
        assemble_info_dir,
        args.generated_app_src_root_dir_name,
    )
    info_file = os.path.join(resource_dir, "info.json")
    if not os.path.exists(resource_dir):
        os.makedirs(resource_dir, exist_ok=True)

    with open(info_file, "w", encoding="utf-8") as f:
        json.dump(info, f, ensure_ascii=False, indent=2)


def copy_replacement_files(
    args: ArgumentInfo, replaced_files: list[str], dest_dir: str
):
    if replaced_files is None:
        return

    out_dir = os.path.join(
        args.test_case_out_dir,
        assemble_info_dir,
        args.generated_app_src_root_dir_name,
        dest_dir,
    )

    for file in replaced_files:
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
        "--replace-paths-after-install-app",
        type=str,
        action="append",
        help="List of files to replace after installing app",
    )

    parser.add_argument(
        "--replace-paths-after-install-all",
        type=str,
        action="append",
        help="List of files to replace after installing all",
    )

    arg_info = ArgumentInfo()
    args = parser.parse_args(namespace=arg_info)

    dump_integration_test_preparation_info_json(args)

    copy_replacement_files(
        args,
        args.replace_paths_after_install_app,
        "files_to_be_replaced_after_install_app",
    )

    copy_replacement_files(
        args,
        args.replace_paths_after_install_all,
        "files_to_be_replaced_after_install_all",
    )

    sys.exit(0)
