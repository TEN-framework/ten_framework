#
# Copyright © 2025 Agora
# This file is part of TEN Framework, an open source project.
# Licensed under the Apache License, Version 2.0, with certain conditions.
# Refer to the "LICENSE" file in the root directory for more information.
#
import argparse
import os
import sys
import glob


class ArgumentInfo(argparse.Namespace):
    def __init__(self):
        self.dir: list[str]
        self.recursive: bool
        self.only_output_file: bool


def glob_file(dirs: list[str], recursive: bool, only_file: bool) -> None:
    for dir in dirs:
        for v in glob.glob(dir, recursive=recursive):
            if only_file:
                if not os.path.isfile(v):
                    continue

            sys.stdout.write(v.replace("\\", "/") + "\n")


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--dir", type=str, action="append", default=[])
    parser.add_argument("--recursive", action="store_true")
    # Skip directory in the output.
    parser.add_argument("--only-output-file", action="store_true")

    arg_info = ArgumentInfo()
    args = parser.parse_args(namespace=arg_info)

    glob_file(args.dir, args.recursive, args.only_output_file)

    sys.exit(0)
