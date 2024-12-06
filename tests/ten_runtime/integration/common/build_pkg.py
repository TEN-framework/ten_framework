#
# Copyright © 2024 Agora
# This file is part of TEN Framework, an open source project.
# Licensed under the Apache License, Version 2.0, with certain conditions.
# Refer to the "LICENSE" file in the root directory for more information.
#
import json
import os
from datetime import datetime
from . import (
    cmd_exec,
    fs_utils,
    build_config,
    install_pkg,
    install_all,
    replace,
)


class ArgumentInfo:
    def __init__(self):
        self.pkg_src_root_dir: str
        self.pkg_run_root_dir: str
        self.pkg_name: str
        self.pkg_language: str
        self.os: str
        self.cpu: str
        self.build: str
        self.tgn_path: str
        self.is_clang: bool
        self.enable_sanitizer: bool
        self.vs_version: str
        self.log_level: int


def _construct_extra_args_for_cpp_ag(args: ArgumentInfo) -> list[str]:
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


def _build_cpp_app(args: ArgumentInfo) -> int:
    # tgn gen ...
    cmd = [f"{args.tgn_path}", "gen", args.os, args.cpu, args.build]
    cmd += _construct_extra_args_for_cpp_ag(args)

    returncode, output = cmd_exec.run_cmd(cmd, args.log_level)

    if returncode:
        raise Exception("Failed to build c++ app")

    # tgn build ...
    cmd = [f"{args.tgn_path}", "build", args.os, args.cpu, args.build]

    t1 = datetime.now()
    returncode, output = cmd_exec.run_cmd(cmd, args.log_level)
    t2 = datetime.now()

    duration = (t2 - t1).seconds

    if args.log_level > 0:
        print(f"Build c++ app({args.pkg_name}) costs {duration} seconds.")

    if returncode:
        raise Exception("Failed to build c++ app")

    # Copy the build result to the specified run folder.
    fs_utils.copy(
        f"{args.pkg_src_root_dir}/out/{args.os}/{args.cpu}/app/{args.pkg_name}",
        args.pkg_run_root_dir,
        True,
    )

    return returncode


def _build_cpp_extension(args: ArgumentInfo) -> int:
    # tgn gen ...
    cmd = [f"{args.tgn_path}", "gen", args.os, args.cpu, args.build]
    cmd += _construct_extra_args_for_cpp_ag(args)

    returncode, _ = cmd_exec.run_cmd(cmd, args.log_level)

    if returncode:
        raise Exception("Failed to build c++ extension.")

    # tgn build ...
    cmd = [f"{args.tgn_path}", "build", args.os, args.cpu, args.build]

    returncode, _ = cmd_exec.run_cmd(cmd, args.log_level)

    if returncode:
        raise Exception("Failed to build c++ extension.")

    return returncode


def _build_go_app(args: ArgumentInfo) -> int:
    # Determine the path to the main.go script. Some cases require a customized
    # Go build script, but in most situations, the build script provided by the
    # TEN runtime Go binding can be used directly.
    main_go_path = "scripts/build/main.go"
    if not os.path.exists(main_go_path):
        main_go_path = "ten_packages/system/ten_runtime_go/tools/build/main.go"

    cmd = ["go", "run", main_go_path]
    if args.log_level > 0:
        cmd += ["--verbose"]

    if args.enable_sanitizer:
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
        print(output)
        raise Exception("Failed to build go app.")

    return returncode


def _get_pkg_type(pkg_root: str) -> str:
    manifest_path = os.path.join(pkg_root, "manifest.json")
    with open(manifest_path, "r") as f:
        manifest_json = json.load(f)
    return manifest_json["type"]


def _build_app(args: ArgumentInfo) -> int:
    returncode = 0

    if args.pkg_language == "cpp":
        returncode = _build_cpp_app(args)
    elif args.pkg_language == "go":
        returncode = _build_go_app(args)
    elif args.pkg_language == "python":
        returncode = 0
    else:
        raise Exception(f"Unknown app language: {args.pkg_language}")

    return returncode


def _build_extension(args: ArgumentInfo) -> int:
    returncode = 0

    if args.pkg_language == "cpp":
        returncode = _build_cpp_extension(args)
    else:
        returncode = 0

    return returncode


def prepare_app(
    build_config: build_config.BuildConfig,
    root_dir: str,
    test_case_base_dir: str,
    source_pkg_name: str,
    log_level: int,
) -> int:
    if log_level and log_level > 0:
        print(f"> Install app to {source_pkg_name}")

    tman_path = os.path.join(root_dir, "ten_manager/bin/tman")
    tman_config_file = os.path.join(
        root_dir, "tests/local_registry/config.json"
    )
    local_registry_path = os.path.join(
        root_dir, "tests/local_registry/registry"
    )

    # read the assembly info from <test_case_base_dir>/.assemble_info/<source_pkg_name>/info.json
    assemble_info_dir = os.path.join(
        test_case_base_dir, ".assemble_info", source_pkg_name
    )
    info_file = os.path.join(assemble_info_dir, "info.json")
    with open(info_file, "r") as f:
        info = json.load(f)
        pkg_name = info["src_app"]
        generated_app_src_root_dir_name = info[
            "generated_app_src_root_dir_name"
        ]

    app_src_root_dir = os.path.join(
        test_case_base_dir, generated_app_src_root_dir_name
    )

    arg = install_pkg.ArgumentInfo()
    arg.tman_path = tman_path
    arg.pkg_type = "app"
    arg.pkg_src_root_dir = app_src_root_dir
    arg.src_pkg = pkg_name
    arg.build_type = build_config.target_build
    arg.config_file = tman_config_file
    arg.log_level = log_level
    arg.local_registry_path = local_registry_path

    install_res = install_pkg.main(arg)
    if install_res != 0:
        raise Exception("Failed to install app")

    if log_level and log_level > 0:
        print(f"> Replace files after install app")

    rc = _replace_after_install_app(test_case_base_dir, source_pkg_name)
    if rc != 0:
        raise Exception("Failed to replace files after install app")

    if log_level and log_level > 0:
        print(f"> Install all")

    install_all_args = install_all.ArgumentInfo()
    install_all_args.tman_path = tman_path
    install_all_args.pkg_src_root_dir = app_src_root_dir
    install_all_args.build_type = build_config.target_build
    install_all_args.config_file = tman_config_file
    install_all_args.log_level = log_level
    install_all_args.assume_yes = True

    rc = install_all.main(install_all_args)

    if log_level and log_level > 0:
        print(f"> Replace files after install all")

    rc = _replace_after_install_all(test_case_base_dir, source_pkg_name)
    if rc != 0:
        raise Exception("Failed to replace files after install all")

    return rc


def _replace_after_install_app(
    test_case_base_dir: str, source_pkg_name: str
) -> int:
    assemble_info_dir = os.path.join(
        test_case_base_dir, ".assemble_info", source_pkg_name
    )
    assemble_info_file = os.path.join(assemble_info_dir, "info.json")

    replace_files_after_install_app: list[str] = []

    with open(assemble_info_file, "r") as f:
        info = json.load(f)
        replace_files_after_install_app = info[
            "replace_files_after_install_app"
        ]

    if (
        not replace_files_after_install_app
        or len(replace_files_after_install_app) == 0
    ):
        return 0

    replaced_files = []
    for replace_file in replace_files_after_install_app:
        src_file = os.path.join(
            assemble_info_dir,
            "files_to_be_replaced_after_install_app",
            replace_file,
        )

        if not os.path.exists(src_file):
            raise Exception(f"{src_file} does not exist.")

        dst_file = os.path.join(test_case_base_dir, replace_file)
        replaced_files.append((src_file, dst_file))

    replace.replace_normal_files_or_merge_json_files(replaced_files)

    return 0


def _replace_after_install_all(
    test_case_base_dir: str, source_pkg_name: str
) -> int:
    assemble_info_dir = os.path.join(
        test_case_base_dir, ".assemble_info", source_pkg_name
    )
    assemble_info_file = os.path.join(assemble_info_dir, "info.json")

    replace_files_after_install_all: list[str] = []

    with open(assemble_info_file, "r") as f:
        info = json.load(f)
        replace_files_after_install_all = info[
            "replace_files_after_install_all"
        ]

    if (
        not replace_files_after_install_all
        or len(replace_files_after_install_all) == 0
    ):
        return 0

    replaced_files = []
    for replace_file in replace_files_after_install_all:
        src_file = os.path.join(
            assemble_info_dir,
            "files_to_be_replaced_after_install_all",
            replace_file,
        )

        if not os.path.exists(src_file):
            raise Exception(f"{src_file} does not exist.")

        dst_file = os.path.join(test_case_base_dir, replace_file)
        replaced_files.append((src_file, dst_file))

    replace.replace_normal_files_or_merge_json_files(replaced_files)

    return 0


def build(
    build_config: build_config.BuildConfig,
    pkg_src_root_dir: str,
    pkg_run_root_dir: str,
    pkg_name: str,
    pkg_language: str,
    log_level: int = 0,
) -> int:
    args = ArgumentInfo()
    args.pkg_src_root_dir = pkg_src_root_dir
    args.pkg_run_root_dir = pkg_run_root_dir
    args.pkg_name = pkg_name
    args.pkg_language = pkg_language
    args.os = build_config.target_os
    args.cpu = build_config.target_cpu
    args.build = build_config.target_build
    args.is_clang = build_config.is_clang
    args.enable_sanitizer = build_config.enable_sanitizer
    args.vs_version = build_config.vs_version
    args.log_level = log_level

    if args.os == "win":
        tgn_path_in_env = os.getenv("tgn")
        if tgn_path_in_env:
            args.tgn_path = tgn_path_in_env
        else:
            return 1
    else:
        args.tgn_path = "tgn"

    if args.log_level > 0:
        msg = (
            f"> Start to build package({args.pkg_name})"
            " in {args.pkg_src_root_dir}"
        )
        print(msg)

    origin_wd = os.getcwd()
    returncode = 0

    try:
        os.chdir(args.pkg_src_root_dir)
        pkg_type = _get_pkg_type(args.pkg_src_root_dir)
        if pkg_type == "app":
            returncode = _build_app(args)
        elif pkg_type == "extension":
            returncode = _build_extension(args)
        else:
            returncode = 0

        if returncode > 0:
            raise Exception(f"Failed to build {pkg_type}({args.pkg_name})")

        if args.log_level > 0:
            print(f"Build {pkg_type}({args.pkg_name}) success")

    except Exception as e:
        returncode = 1
        print(f"Build package({args.pkg_name}) failed: {repr(e)}")

    finally:
        os.chdir(origin_wd)

    return returncode


def prepare_and_build(
    build_config: build_config.BuildConfig,
    root_dir: str,
    test_case_base_dir: str,
    pkg_run_root_dir: str,
    source_pkg_name: str,
    pkg_language: str,
    log_level: int = 2,
):
    rc = prepare_app(
        build_config,
        root_dir,
        test_case_base_dir,
        source_pkg_name,
        log_level,
    )
    if rc != 0:
        return rc

    pkg_src_root_dir = os.path.join(test_case_base_dir, source_pkg_name)

    rc = build(
        build_config,
        pkg_src_root_dir,
        pkg_run_root_dir,
        source_pkg_name,
        pkg_language,
        log_level,
    )

    return rc


def cleanup(
    pkg_src_dir: str,
    pkg_run_dir: str,
):
    fs_utils.remove_tree(pkg_src_dir)
    fs_utils.remove_tree(pkg_run_dir)
    return 0