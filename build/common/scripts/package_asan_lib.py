#
# Copyright © 2024 Agora
# This file is part of TEN Framework, an open source project.
# Licensed under the Apache License, Version 2.0, with certain conditions.
# Refer to the "LICENSE" file in the root directory for more information.
#
import argparse
import os
from build.scripts import fs_utils, write_depfile, detect_asan_lib


class ArgumentInfo(argparse.Namespace):
    def __init__(self):
        self.target_os: str
        self.target_arch: str
        self.asan_lib_dst_loc: str
        self.depfile: str


if __name__ == "__main__":
    parser = argparse.ArgumentParser()

    parser.add_argument("--target-os", type=str)
    parser.add_argument("--target-arch", type=str)
    parser.add_argument("--asan-lib-dst-loc", type=str)
    parser.add_argument("--depfile", type=str)

    arg_info = ArgumentInfo()
    args = parser.parse_args(namespace=arg_info)

    if args.target_os == "mac":
        asan_lib_src_loc = detect_asan_lib.detect_mac_asan_lib(args.target_arch)
    elif args.target_os == "linux":
        asan_lib_src_loc = detect_asan_lib.detect_linux_asan_lib(
            args.target_arch
        )
    elif args.target_os == "linux_clang":
        asan_lib_src_loc = detect_asan_lib.detect_linux_clang_asan_lib(
            args.target_arch
        )
    else:
        raise Exception("Unsupported OS.")

    fs_utils.copy(asan_lib_src_loc, args.asan_lib_dst_loc)

    write_depfile.write_depfile(
        os.path.basename(args.asan_lib_dst_loc),
        [asan_lib_src_loc],
        args.depfile,
    )
