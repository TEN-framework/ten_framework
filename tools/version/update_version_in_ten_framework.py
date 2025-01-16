#
# Copyright © 2025 Agora
# This file is part of TEN Framework, an open source project.
# Licensed under the Apache License, Version 2.0, with certain conditions.
# Refer to the "LICENSE" file in the root directory for more information.
#
import datetime
import json
import os
from common import (
    get_latest_git_tag,
    update_c_preserved_metadata_version,
    update_version_in_manifest_json_file,
    update_dependency_version_in_manifest_json_file,
    update_version_source_file_of_tman,
    update_version_in_manifest_json_file_for_pkgs,
    PkgInfo,
)

MANIFEST_JSON_FILE = "manifest.json"
MANIFEST_JSON_TENT_FILE = "manifest.json.tent"


def __get_pkg_info_from_manifest_file(manifest_file: str) -> PkgInfo:
    with open(manifest_file, "r") as f:
        manifest = json.load(f)
        return PkgInfo(manifest["type"], manifest["name"])


def __collect_manifest_files(directory) -> list[str]:
    manifests = []

    for _, dirs, _ in os.walk(directory, followlinks=True):
        for dir in dirs:
            if os.path.exists(os.path.join(directory, dir, MANIFEST_JSON_FILE)):
                manifests.append(
                    os.path.join(
                        directory,
                        dir,
                        MANIFEST_JSON_FILE,
                    )
                )
        break

    return manifests


def __collect_manifest_tent_files(directory) -> list[str]:
    manifest_templates = []

    for _, dirs, _ in os.walk(directory, followlinks=True):
        for dir in dirs:
            if os.path.exists(
                os.path.join(directory, dir, MANIFEST_JSON_TENT_FILE)
            ):
                manifest_templates.append(
                    os.path.join(
                        directory,
                        dir,
                        MANIFEST_JSON_TENT_FILE,
                    )
                )
        break

    return manifest_templates


def update_c_preserved_metadata_version_of_ten_runtime_binary(
    log_level, year, year_month, git_version
):
    # Update the version in the C preserved metadata files.
    c_preserved_metadata_file_src_file = os.path.join(
        repo_base_dir,
        "core",
        "src",
        "ten_runtime",
        "build_template",
        "preserved_metadata.c",
    )

    c_preserved_metadata_file_template_file = os.path.join(
        repo_base_dir,
        "core",
        "src",
        "ten_runtime",
        "build_template",
        "preserved_metadata.c.jinja2",
    )

    update_c_preserved_metadata_version(
        log_level,
        year,
        year_month,
        git_version,
        c_preserved_metadata_file_src_file,
        c_preserved_metadata_file_template_file,
    )


def update_version_of_tman(
    log_level: int,
    year: str,
    year_month: str,
    repo_base_dir: str,
    git_version: str,
):
    tman_version_src_file_path = os.path.join(
        repo_base_dir,
        "core",
        "src",
        "ten_manager",
        "src",
        "version.rs",
    )

    tman_version_template_file_path = os.path.join(
        repo_base_dir,
        "core",
        "src",
        "ten_manager",
        "src",
        "version.rs.jinja2",
    )

    update_version_source_file_of_tman(
        log_level,
        year,
        year_month,
        git_version,
        tman_version_src_file_path,
        tman_version_template_file_path,
    )


def collect_and_update_version_of_system_packages(
    log_level, repo_base_dir, git_version
) -> list[PkgInfo]:
    # Collect manifest files for ten_runtime and all corresponding system
    # packages (python & go bindings).
    manifest_files = [
        # ten_runtime
        os.path.join(
            repo_base_dir,
            "core",
            "src",
            "ten_runtime",
            MANIFEST_JSON_FILE,
        ),
        # ten_runtime_go
        os.path.join(
            repo_base_dir,
            "core",
            "src",
            "ten_runtime",
            "binding",
            "go",
            MANIFEST_JSON_FILE,
        ),
        # ten_runtime_python
        os.path.join(
            repo_base_dir,
            "core",
            "src",
            "ten_runtime",
            "binding",
            "python",
            MANIFEST_JSON_FILE,
        ),
        # ten_runtime_nodejs
        os.path.join(
            repo_base_dir,
            "core",
            "src",
            "ten_runtime",
            "binding",
            "nodejs",
            MANIFEST_JSON_FILE,
        ),
    ]

    pkgInfos = []

    for manifest_file in manifest_files:
        update_version_in_manifest_json_file(
            log_level, git_version, manifest_file
        )
        pkgInfos.append(__get_pkg_info_from_manifest_file(manifest_file))

    return pkgInfos


def collect_and_update_version_of_core_packages(
    log_level, repo_base_dir, git_version
) -> list[PkgInfo]:
    # Collect manifest files for all core packages.
    core_apps_dir_path = os.path.join(
        repo_base_dir,
        "packages",
        "core_apps",
    )
    core_extensions_dir_path = os.path.join(
        repo_base_dir,
        "packages",
        "core_extensions",
    )
    core_protocols_dir_path = os.path.join(
        repo_base_dir,
        "packages",
        "core_protocols",
    )
    core_addon_loaders_dir_path = os.path.join(
        repo_base_dir,
        "packages",
        "core_addon_loaders",
    )

    manifest_files = (
        __collect_manifest_files(core_apps_dir_path)
        + __collect_manifest_files(core_extensions_dir_path)
        + __collect_manifest_files(core_protocols_dir_path)
        + __collect_manifest_files(core_addon_loaders_dir_path)
    )

    manifest_template_files = (
        __collect_manifest_tent_files(core_apps_dir_path)
        + __collect_manifest_tent_files(core_extensions_dir_path)
        + __collect_manifest_tent_files(core_protocols_dir_path)
        + __collect_manifest_tent_files(core_addon_loaders_dir_path)
    )

    pkg_infos = []

    for manifest_file in manifest_files:
        update_version_in_manifest_json_file(
            log_level, git_version, manifest_file
        )
        pkg_infos.append(__get_pkg_info_from_manifest_file(manifest_file))

    for manifest_template_file in manifest_template_files:
        update_version_in_manifest_json_file(
            log_level, git_version, manifest_template_file
        )

    # Some core packages are used and overwritten in tests. Update them as well.
    manifests_in_tests = []

    test_dir_path = os.path.join(repo_base_dir, "tests")
    for root, _, files in os.walk(test_dir_path, followlinks=True):
        for file in files:
            if file == MANIFEST_JSON_FILE:
                manifests_in_tests.append(os.path.join(root, file))

    for manifest in manifests_in_tests:
        update_version_in_manifest_json_file_for_pkgs(
            log_level, git_version, manifest, pkg_infos
        )

    return pkg_infos


def update_dependencies_version(
    log_level, repo_base_dir, git_version, dependencies
):
    # Collect manifest files for system packages.
    system_package_dir_path = os.path.join(
        repo_base_dir, "core", "src", "ten_runtime", "binding"
    )

    # Collect manifest files for all packages.
    packages_dir_path = os.path.join(
        repo_base_dir,
        "packages",
    )

    # Collect manifest files for testing packages.
    test_dir_path = os.path.join(repo_base_dir, "tests")

    manifests = __collect_manifest_files(system_package_dir_path)

    # Collect manifest.json from all package directories.
    for root, dirs, files in os.walk(packages_dir_path, followlinks=True):
        for dir in dirs:
            manifests += __collect_manifest_files(
                os.path.join(packages_dir_path, dir)
            )
            manifests += __collect_manifest_tent_files(
                os.path.join(packages_dir_path, dir)
            )

        break

    # Define the blacklist of manifest.json files to skip.
    blacklist = [
        os.path.join(
            test_dir_path,
            "ten_manager",
            "error_context",
            "package_version_not_found",
            "test_app",
            MANIFEST_JSON_FILE,
        ),
        # Add more paths as needed.
    ]

    # Iterate through test files and apply blacklist.
    for root, dirs, files in os.walk(test_dir_path, followlinks=True):
        for file in files:
            if file == MANIFEST_JSON_FILE:
                manifest_path = os.path.join(root, file)
                # Blacklist Check.
                if manifest_path not in blacklist:
                    manifests.append(manifest_path)
                else:
                    print(f"Skipping blacklisted manifest: {manifest_path}")

    # Update dependency versions in all collected manifest.json.
    for manifest in manifests:
        update_dependency_version_in_manifest_json_file(
            log_level,
            git_version,
            manifest,
            dependencies,
        )


if __name__ == "__main__":
    now = datetime.datetime.now()
    year = now.strftime("%Y")
    year_month = now.strftime("%Y-%m")

    # Get the repo base directory.
    repo_base_dir = os.path.abspath(
        os.path.join(os.path.dirname(__file__), "..", "..")
    )

    # Change to the correct directory to get the correct git tag.
    os.chdir(repo_base_dir)

    # Get the latest Git tag, and optionally strip leading 'v' if present.
    git_version = get_latest_git_tag()
    git_version = git_version.lstrip("v")

    log_level = 1

    update_c_preserved_metadata_version_of_ten_runtime_binary(
        log_level, year, year_month, git_version
    )

    update_version_of_tman(
        log_level, year, year_month, repo_base_dir, git_version
    )

    system_pkgs = collect_and_update_version_of_system_packages(
        log_level, repo_base_dir, git_version
    )

    core_pkgs = collect_and_update_version_of_core_packages(
        log_level, repo_base_dir, git_version
    )

    update_dependencies_version(
        log_level, repo_base_dir, git_version, system_pkgs + core_pkgs
    )
