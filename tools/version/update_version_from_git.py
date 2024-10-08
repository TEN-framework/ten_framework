#
# Copyright © 2024 Agora
# This file is part of TEN Framework, an open source project.
# Licensed under the Apache License, Version 2.0, with certain conditions.
# Refer to the "LICENSE" file in the root directory for more information.
#
import subprocess
import json
import argparse
import os
import datetime
from jinja2 import Template


class PkgInfo:
    def __init__(self, pkg_type, pkg_name):
        self.pkg_type = pkg_type
        self.pkg_name = pkg_name

    def __eq__(self, value):
        return self.pkg_type == value.pkg_type and self.pkg_name == value.pkg_name

    def __hash__(self):
        return hash((self.pkg_type, self.pkg_name))


def touch(path):
    with open(path, "a"):
        try:
            os.utime(path, follow_symlinks=False)
        except Exception:
            try:
                # If follow_symlinks parameter is not supported, fall back to
                # default behavior
                os.utime(path)
            except Exception:
                exit(1)


class ArgumentInfo(argparse.Namespace):
    def __init__(self):
        super().__init__()
        self.repo_base_dir: str
        self.log_level: int
        self.pkg_type: str | None = None
        self.pkg_name: str | None = None
        self.c_preserved_metadata_path: list[tuple[str, str]] = []
        self.version_update_manifest_path: list[str] = []
        self.dependency_version_update_manifest_path: list[str] = []


def get_latest_git_tag() -> str:
    result = subprocess.run(
        ["git", "describe", "--tags", "--abbrev=0"],
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        raise Exception("Failed to execute git command")
    return result.stdout.strip()


def update_c_preserved_metadata_file(
    log_level: int,
    year: str,
    year_month: str,
    version: str,
    src_path: str,
    template_path: str,
) -> None:
    update_needed = False

    if os.path.exists(src_path):
        with open(src_path, "r") as file:
            content = file.read()

        if f"version={version}" not in content:
            update_needed = True
            if log_level > 0:
                print(f"Version mismatch found. Updating version in {src_path}.")
    else:
        update_needed = True

    if update_needed:
        with open(template_path, "r") as file:
            template_content = file.read()

        template = Template(template_content)

        rendered_content = template.render(
            VERSION=version, YEAR=year, YEAR_MONTH=year_month
        )

        with open(src_path, "w") as file:
            file.write(rendered_content)
    else:
        if log_level > 0:
            print(f"No update needed for {src_path}; versions match.")


# {
#   "type": "<type-in-main-manifest>",
#   "name": "<name-in-main-manifest>",
#   "version": "...", <- update here
#   ...
# }
def update_version_in_manifest_json_file(
    log_level: int,
    version: str,
    src_path: str,
) -> None:
    if log_level > 0:
        print(f"Checking {src_path} for updates...")

    if not os.path.exists(src_path):
        touch(src_path)

    with open(src_path, "r") as file:
        data = json.load(file)

    if data.get("version") != version:
        if log_level > 0:
            print(f"Updating version in {src_path}.")

        # Update the version in the JSON data.
        data["version"] = version

        with open(src_path, "w") as file:
            json.dump(data, file, indent=2)

            # Notify that the content of the src_path has changed.
            touch(src_path)
    else:
        if log_level > 0:
            print(f"No update needed for {src_path}; versions match.")


# {
#   ...
#   "dependencies": [
#     {
#       "type": "<type-in-main-manifest>",
#       "name": "<name-in-main-manifest>",
#       "version": "...", <- update here
#     }
#   }
#   ...
# }
def update_dependency_manifest_json_file(
    log_level: int,
    version: str,
    src_path: str,
    pkgs: list[PkgInfo],
) -> None:
    if log_level > 0:
        print(f"Checking {src_path} for updates...")

    if not os.path.exists(src_path):
        touch(src_path)

    with open(src_path, "r") as file:
        data = json.load(file)

    updated = False
    for dependency in data.get("dependencies", []):
        dependent_pkg = PkgInfo(dependency.get("type"), dependency.get("name"))

        if dependent_pkg in pkgs and dependency.get("version") != version:
            dependency["version"] = version
            updated = True

    if updated:
        if log_level > 0:
            print(f"Updating version in {src_path}.")

        with open(src_path, "w") as file:
            json.dump(data, file, indent=2)

            # Notify that the content of the src_path has changed.
            touch(src_path)
    else:
        if log_level > 0:
            print(f"No update needed for {src_path}; versions match.")


def update_tman_version_source_file(
    log_level: int,
    year: str,
    year_month: str,
    version: str,
    src_path: str,
    template_path: str,
) -> None:
    update_needed = False

    if os.path.exists(src_path):
        with open(src_path, "r") as file:
            content = file.read()

        if f'VERSION: &str = "{version}"' not in content:
            update_needed = True
            if log_level > 0:
                print(f"Version mismatch found. Updating version in {src_path}.")
    else:
        update_needed = True

    if update_needed:
        with open(template_path, "r") as file:
            template_content = file.read()

        template = Template(template_content)

        rendered_content = template.render(
            VERSION=version, YEAR=year, YEAR_MONTH=year_month
        )

        with open(src_path, "w") as file:
            file.write(rendered_content)
    else:
        if log_level > 0:
            print(f"No update needed for {src_path}; versions match.")


def main(args: ArgumentInfo):
    now = datetime.datetime.now()
    year = now.strftime("%Y")
    year_month = now.strftime("%Y-%m")

    # Change to the correct directory to get the correct git tag.
    os.chdir(args.repo_base_dir)

    # Get the latest Git tag, and optionally strip leading 'v' if present.
    git_version = get_latest_git_tag()
    git_version = git_version.lstrip("v")

    if args.c_preserved_metadata_path:
        for src, template in args.c_preserved_metadata_path:
            update_c_preserved_metadata_file(
                args.log_level, year, year_month, git_version, src, template
            )

    if args.version_update_manifest_path:
        for src in args.version_update_manifest_path:
            update_version_in_manifest_json_file(args.log_level, git_version, src)

    if args.dependency_version_update_manifest_path:
        for src in args.dependency_version_update_manifest_path:
            update_dependency_manifest_json_file(
                args.log_level,
                git_version,
                src,
                [PkgInfo(args.pkg_type, args.pkg_name)],
            )


if __name__ == "__main__":
    parser = argparse.ArgumentParser()

    parser.add_argument("--repo-base-dir", type=str, required=True)
    parser.add_argument("--log-level", type=int, required=True)

    parser.add_argument("--pkg-type", type=str)
    parser.add_argument("--pkg-name", type=str)

    parser.add_argument(
        "--c-preserved-metadata-path",
        action="append",
        nargs=2,
        metavar=("src", "template"),
    )
    parser.add_argument(
        "--version-update-manifest-path",
        action="append",
        default=[],
    )
    parser.add_argument(
        "--dependency-version-update-manifest-path",
        action="append",
        default=[],
    )

    arg_info = ArgumentInfo()
    args = parser.parse_args(namespace=arg_info)

    if args.dependency_version_update_manifest_path and (
        not args.pkg_type or not args.pkg_name
    ):
        error_message = (
            "--pkg-type and --pkg-name are required when "
            "--dependency-version-update-manifest-path is specified"
        )
        parser.error(error_message)

    main(args)
