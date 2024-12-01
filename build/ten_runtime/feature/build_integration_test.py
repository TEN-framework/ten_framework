#
# Copyright © 2024 Agora
# This file is part of TEN Framework, an open source project.
# Licensed under the Apache License, Version 2.0, with certain conditions.
# Refer to the "LICENSE" file in the root directory for more information.
#
import os
import subprocess
import sys
import argparse

# Import existing scripts if needed
# from install_pkg import main as install_pkg_main
# from install_all import main as install_all_main
# ... etc.


class ArgumentInfo(argparse.Namespace):
    def __init__(self):
        self.tman_path: str
        self.pkg_src_root_dir: str
        self.tg_timestamp_proxy_file: str
        self.src_pkg: str
        self.build_type: str
        self.config_file: str
        self.possible_published_results: list[str]
        self.depfile: str
        self.depfile_target: str
        self.log_level: int
        self.pkg_type: str
        self.local_registry_path: str


def prepare_app(args: ArgumentInfo):
    # Calculate paths.
    test_case_root_dir = os.path.join(args.root_out_dir, args.test_case_dir)
    app_src_root_dir = os.path.join(
        test_case_root_dir, args.generated_app_src_root_dir_name
    )
    app_run_root_dir = os.path.join(test_case_root_dir, args.target_name)

    # Install app.
    if args.log_level > 0:
        print(f"> Install app to {args.pkg_src_root_dir}")

    install_pkg_cmd = [
        sys.executable,
        "install_pkg.py",
        "--tman-path",
        args.tman_path,
        "--pkg-type",
        "app",
        "--pkg-src-root-dir",
        app_src_root_dir,
        "--tg-timestamp-proxy-file",
        args.install_app_dummy_output_file,
        "--src-pkg",
        args.src_app,
        "--build-type",
        args.build_type,
        "--config-file",
        args.config_file,
        "--log-level",
        str(args.log_level),
        "--local-registry-path",
        args.local_registry_path,
    ]
    subprocess.check_call(install_pkg_cmd)

    # Replace files after install app
    replace_files(args.replace_files_after_install_app, test_case_root_dir)

    # Install all dependencies
    install_all_cmd = [
        sys.executable,
        "install_all.py",
        "--tman-path",
        args.tman_path,
        "--pkg-src-root-dir",
        app_src_root_dir,
        "--tg-timestamp-proxy-file",
        args.install_all_dummy_output_file,
        "--build-type",
        args.build_type,
        "--config-file",
        args.config_file,
        "--log-level",
        str(args.log_level),
        "--assume-yes",
        "True",
    ]
    subprocess.check_call(install_all_cmd)

    # Replace files after install all
    replace_files(args.replace_files_after_install_all, test_case_root_dir)

    # Build the app
    build_pkg_cmd = [
        sys.executable,
        "build_pkg.py",
        "--pkg-src-root-dir",
        app_src_root_dir,
        "--pkg-run-root-dir",
        app_run_root_dir,
        "--tg-timestamp-proxy-file",
        args.build_app_dummy_output_file,
        "--pkg-name",
        args.generated_app_src_root_dir_name,
        "--pkg-language",
        args.src_app_language,
        "--os",
        args.target_os,
        "--cpu",
        args.target_cpu,
        "--build",
        args.build_type,
        "--tgn-path",
        args.tgn_path,
        "--is-clang",
        str(args.is_clang),
        "--enable-sanitizer",
        str(args.enable_sanitizer),
        "--vs-version",
        args.vs_version,
        "--log-level",
        str(args.log_level),
    ]
    subprocess.check_call(build_pkg_cmd)


def replace_files(replace_files_list, test_case_root_dir):
    for replace_file in replace_files_list:
        src_dest = replace_file.split("=>")
        if len(src_dest) == 1:
            src = src_dest[0]
            dest = src_dest[0]
        else:
            src, dest = src_dest
        src = os.path.abspath(src)
        dest = os.path.join(test_case_root_dir, dest)
        replace_cmd = [
            sys.executable,
            "replace_normal_files_or_merge_json_files.py",
            "--replaced-files",
            f"{src}=>{dest}",
        ]
        subprocess.check_call(replace_cmd)


def main():
    parser = argparse.ArgumentParser()

    parser.add_argument(
        "--tman-path",
        type=str,
        required=True,
        help="Path to tman command itself",
    )
    parser.add_argument(
        "--pkg-type",
        type=str,
        required=True,
        help="The type of package",
        choices=["app", "extension"],
    )
    parser.add_argument(
        "--pkg-src-root-dir",
        type=str,
        required=True,
        help="Path to app source root directory",
    )
    parser.add_argument(
        "--tg-timestamp-proxy-file",
        type=str,
        required=True,
        help="Path to the stamp file",
    )
    parser.add_argument(
        "--src-pkg",
        type=str,
        required=True,
        help="The name of TEN package",
    )
    parser.add_argument(
        "--build-type",
        type=str,
        required=False,
        help="'debug', 'release'",
    )
    parser.add_argument(
        "--config-file",
        type=str,
        required=False,
        help="Path of tman config file",
    )
    parser.add_argument(
        "--possible-published-results",
        type=str,
        action="append",
        default=[],
    )
    parser.add_argument(
        "--depfile",
        type=str,
    )
    parser.add_argument(
        "--depfile-target",
        type=str,
    )
    parser.add_argument(
        "--log-level",
        type=int,
        required=True,
        help="specify log level",
    )
    parser.add_argument(
        "--local-registry-path",
        type=str,
        required=False,
        help="Path to local registry",
    )

    arg_info = ArgumentInfo()
    args = parser.parse_args(namespace=arg_info)

    prepare_app(args)


if __name__ == "__main__":
    main()
