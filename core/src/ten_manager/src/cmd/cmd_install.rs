//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::{
    collections::HashMap,
    fs::{self},
    path::{Path, PathBuf},
    sync::Arc,
    time::Instant,
};

use anyhow::{anyhow, Context, Result};
use clap::{Arg, ArgMatches, Command};
use console::Emoji;
use indicatif::HumanDuration;
use inquire::Confirm;

use semver::VersionReq;
use ten_rust::pkg_info::{
    constants::{BUILD_GN_FILENAME, MANIFEST_JSON_FILENAME},
    manifest::dependency::ManifestDependency,
    manifest::support::ManifestSupport,
    pkg_basic_info::PkgBasicInfo,
};
use ten_rust::pkg_info::{
    get_pkg_info_from_path,
    pkg_type::PkgType,
    pkg_type_and_name::PkgTypeAndName,
    supports::{Arch, Os},
    PkgInfo,
};

use crate::{
    config::TmanConfig,
    constants::{APP_DIR_IN_DOT_TEN_DIR, DOT_TEN_DIR},
    dep_and_candidate::get_all_candidates_from_deps,
    fs::{check_is_addon_folder, find_nearest_app_dir},
    install::{
        compare_solver_results_with_installed_pkgs,
        filter_compatible_pkgs_to_candidates,
        write_installing_pkg_into_manifest_file,
        write_pkgs_into_manifest_lock_file,
    },
    manifest_lock::parse_manifest_lock_in_folder,
    output::TmanOutput,
    package_info::tman_get_all_installed_pkgs_info_of_app,
    solver::{
        introducer::extract_introducer_relations_from_raw_solver_results,
        solve::{solve_all, DependencyRelationship},
        solver_error::{parse_error_statement, print_conflict_info},
        solver_result::{
            extract_solver_results_from_raw_solver_results,
            filter_solver_results_by_type_and_name,
            install_solver_results_in_app_folder,
        },
    },
    version_utils::parse_pkg_name_version_req,
};

#[derive(Debug, Clone, Copy)]
pub enum LocalInstallMode {
    Invalid,
    Copy,
    Link,
}

impl std::str::FromStr for LocalInstallMode {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "copy" => Ok(LocalInstallMode::Copy),
            "link" => Ok(LocalInstallMode::Link),
            _ => Err(anyhow::anyhow!("Invalid local install mode: {}", s)),
        }
    }
}

#[derive(Debug)]
pub struct InstallCommand {
    pub package_type: Option<String>,
    pub package_name: Option<String>,
    pub support: ManifestSupport,
    pub local_install_mode: LocalInstallMode,
    pub standalone: bool,
    pub cwd: String,

    /// When the user only inputs a single path parameter, if a `manifest.json`
    /// exists under that path, it indicates installation from a local path.
    pub local_path: Option<String>,
}

pub fn create_sub_cmd(args_cfg: &crate::cmd_line::ArgsCfg) -> Command {
    Command::new("install")
        .about("Install a package")
        .arg(
            Arg::new("PACKAGE_TYPE")
                .help("The type of the package")
                .required(false),
        )
        .arg(
            Arg::new("PACKAGE_NAME")
                .help("The name of the package with optional version requirement (e.g., foo@1.0.0)")
                .required(false),
        )
        .arg(
            Arg::new("OS")
                .long("os")
                .help("The operating system")
                .value_parser(args_cfg.os.possible_values.clone())
                .required(false),
        )
        .arg(
            Arg::new("ARCH")
                .long("arch")
                .help("The CPU architecture")
                .value_parser(args_cfg.arch.possible_values.clone())
                .required(false),
        )
        .arg(
            Arg::new("LOCAL_INSTALL_MODE")
                .long("local-install-mode")
                .help("Local install mode: copy or link")
                .value_parser(["copy", "link"])
                .default_value("link")
                .required(false),
        )
        .arg(
            Arg::new("STANDALONE")
                .long("standalone")
                .help("Install in standalone mode, only for extension package")
                .action(clap::ArgAction::SetTrue)
                .required(false),
        )
        .arg(
            Arg::new("CWD")
                .long("cwd")
                .short('C')
                .help("Change the working directory")
                .value_name("DIR")
                .required(false),
        )
}

pub fn parse_sub_cmd(sub_cmd_args: &ArgMatches) -> Result<InstallCommand> {
    let mut cmd = InstallCommand {
        package_type: None,
        package_name: None,
        support: ManifestSupport {
            os: sub_cmd_args
                .get_one::<String>("OS")
                .and_then(|s| s.parse::<Os>().ok()),
            arch: sub_cmd_args
                .get_one::<String>("ARCH")
                .and_then(|s| s.parse::<Arch>().ok()),
        },
        local_install_mode: LocalInstallMode::Invalid,
        standalone: false,
        cwd: String::new(),
        local_path: None,
    };

    let _ = cmd.support.set_defaults();

    // Set the working directory based on --cwd/-C or use the current directory.
    if let Some(cwd) = sub_cmd_args.get_one::<String>("CWD") {
        cmd.cwd = cwd.clone();
    } else {
        cmd.cwd = crate::fs::get_cwd()?.to_string_lossy().to_string();
    }

    // Retrieve the first positional parameter (in the `PACKAGE_TYPE`
    // parameter).
    if let Some(first_arg) = sub_cmd_args.get_one::<String>("PACKAGE_TYPE") {
        let first_arg_str = first_arg.as_str();

        // Determine whether it is an absolute path or a relative path starting
        // with `./` or `../`.
        let is_path = {
            let path = std::path::Path::new(first_arg_str);
            path.is_absolute()
                || first_arg_str.starts_with("./")
                || first_arg_str.starts_with("../")
        };

        if is_path {
            // If it is a path, treat it as `local_path` mode, and
            // `package_type` and `package_name` will no longer be needed
            // afterward.
            cmd.local_path = Some(first_arg.clone());

            if sub_cmd_args.get_one::<String>("PACKAGE_NAME").is_some() {
                return Err(anyhow::anyhow!(
                    "PACKAGE_NAME is not required when a local path is specified."
                ));
            }
        } else {
            // Otherwise, consider it as `package_type`, and `package_name` must
            // be provided as well.

            // Check if PACKAGE_TYPE is an allowed value.
            let allowed_package_types: &[&str] =
                &["system", "protocol", "addon_loader", "extension"];
            if !allowed_package_types
                .contains(&first_arg_str.to_lowercase().as_str())
            {
                return Err(anyhow::anyhow!(
                    "Invalid PACKAGE_TYPE: {}. Allowed values are: {}",
                    first_arg_str,
                    allowed_package_types.join(", ")
                ));
            }

            cmd.package_type = Some(first_arg.clone());

            if let Some(second_arg) =
                sub_cmd_args.get_one::<String>("PACKAGE_NAME")
            {
                cmd.package_name = Some(second_arg.clone());
            } else {
                return Err(anyhow::anyhow!(
                    "PACKAGE_NAME is required when PACKAGE_TYPE is specified."
                ));
            }
        }
    }

    if let Some(mode_str) = sub_cmd_args.get_one::<String>("LOCAL_INSTALL_MODE")
    {
        cmd.local_install_mode = mode_str.parse()?;
    }

    cmd.standalone = sub_cmd_args.get_flag("STANDALONE");

    Ok(cmd)
}

fn get_locked_pkgs(app_dir: &Path) -> Option<HashMap<PkgTypeAndName, PkgInfo>> {
    match parse_manifest_lock_in_folder(app_dir) {
        Ok(manifest_lock) => Some(manifest_lock.get_pkgs()),
        Err(_) => None,
    }
}

fn add_pkg_to_initial_pkg_to_find_candidates_and_all_candidates(
    pkg: &PkgInfo,
    initial_pkgs_to_find_candidates: &mut Vec<PkgInfo>,
    all_candidates: &mut HashMap<
        PkgTypeAndName,
        HashMap<PkgBasicInfo, PkgInfo>,
    >,
) -> Result<()> {
    initial_pkgs_to_find_candidates.push(pkg.clone());

    all_candidates
        .entry(pkg.into())
        .or_default()
        .insert(pkg.into(), pkg.clone());

    Ok(())
}

/// For a C++ extension, a complete and compilable C++ app is required, so a
/// basic `BUILD.gn` is needed.
fn prepare_cpp_standalone_app_dir(dot_ten_app_dir: &Path) -> Result<()> {
    let build_gn_file = dot_ten_app_dir.join(BUILD_GN_FILENAME);
    if !build_gn_file.exists() {
        // Create a basic `BUILD.gn` for the C++ app.
        let content = r#"import("//build/feature/ten_package.gni")

ten_package("app_for_standalone") {
  package_kind = "app"
}
"#;
        fs::write(&build_gn_file, content)?;
    }

    Ok(())
}

fn prepare_basic_standalone_app_dir(
    _tman_config: Arc<TmanConfig>,
    extension_dir: &Path,
) -> Result<PathBuf> {
    let dot_ten_app_dir =
        extension_dir.join(DOT_TEN_DIR).join(APP_DIR_IN_DOT_TEN_DIR);
    if !dot_ten_app_dir.exists() {
        fs::create_dir_all(&dot_ten_app_dir)?;
    }

    let manifest_json = dot_ten_app_dir.join(MANIFEST_JSON_FILENAME);
    if !manifest_json.exists() {
        // Create a basic `manifest.json`, and in that manifest.json, there will
        // be a local dependency pointing to the current extension folder.
        let content = r#"{
  "type": "app",
  "name": "app_for_standalone",
  "version": "0.1.0",
  "dependencies": [
    {
      "path": "../../"
    }
  ]
}
"#;
        fs::write(&manifest_json, content)?;
    }

    Ok(dot_ten_app_dir)
}

/// Prepare the `.ten/app/` folder in the extension standalone mode.
async fn prepare_standalone_app_dir(
    tman_config: Arc<TmanConfig>,
    extension_dir: &Path,
) -> Result<PathBuf> {
    let dot_ten_app_dir =
        prepare_basic_standalone_app_dir(tman_config.clone(), extension_dir)?;

    let build_gn_path = extension_dir.join("BUILD.gn");
    if build_gn_path.exists() {
        prepare_cpp_standalone_app_dir(&dot_ten_app_dir)?;
    }

    Ok(dot_ten_app_dir)
}

/// Path logic for standalone mode and non-standalone mode.
async fn determine_app_dir_to_work_with(
    tman_config: Arc<TmanConfig>,
    standalone: bool,
    specified_cwd: &Path,
) -> Result<PathBuf> {
    if standalone {
        // If it is standalone mode, it can only be executed in the extension
        // directory.
        check_is_addon_folder(specified_cwd).with_context(|| {
            "Standalone mode can only be executed in an addon folder."
        })?;

        let dot_ten_app_dir_path =
            prepare_standalone_app_dir(tman_config, specified_cwd).await?;

        Ok(dot_ten_app_dir_path)
    } else {
        // Non-standalone mode can only be executed in the extension directory.
        // If it is an extension, it should search upwards for the nearest app;
        // if it is an app, it can be used directly.
        let app_dir = find_nearest_app_dir(specified_cwd.to_path_buf())?;

        Ok(app_dir)
    }
}

pub async fn execute_cmd(
    tman_config: Arc<TmanConfig>,
    command_data: InstallCommand,
    out: Arc<Box<dyn TmanOutput>>,
) -> Result<()> {
    if tman_config.verbose {
        out.normal_line("Executing install command");
        out.normal_line(&format!("command_data: {:?}", command_data));
        out.normal_line(&format!("tman_config: {:?}", tman_config));
    }

    let started = Instant::now();

    // Save the actual current working directory before any changes.
    let original_cwd = crate::fs::get_cwd()?;

    // Properly handle relative paths in command_data.cwd.
    let specified_cwd = if Path::new(&command_data.cwd).is_absolute() {
        PathBuf::from(&command_data.cwd)
    } else {
        original_cwd.join(&command_data.cwd)
    };

    let app_dir_to_work_with = determine_app_dir_to_work_with(
        tman_config.clone(),
        command_data.standalone,
        &specified_cwd,
    )
    .await?;

    // If `tman install` is run within the scope of an app, then the app and
    // those addons (extensions, ...) installed in the app directory are all
    // considered initial_pkgs_to_find_candidates.
    let mut initial_pkgs_to_find_candidates = vec![];
    let mut all_candidates: HashMap<
        PkgTypeAndName,
        HashMap<PkgBasicInfo, PkgInfo>,
    > = HashMap::new();

    // 'all_installed_pkgs' contains all the packages which are already located
    // in the `app` directory, including the `app` itself, and all the
    // addons located in `ten_packages/<foo>/<bar>`
    //
    // After the completed dependency tree is resolved,
    // 'all_installed_pkgs' will be compared with the solver results:
    //
    // *) If some of these packages are not included in the dependency tree,
    // then users will be prompted that these packages can be added to the
    // dependencies.
    //
    // *) If some of these packages are not compatible with packages in
    // the dependency tree, then users will be questioned whether to overwrite
    // them with the new packages or quit the installation.
    let all_installed_pkgs: Vec<PkgInfo>;
    let mut all_compatible_installed_pkgs: HashMap<
        PkgTypeAndName,
        HashMap<PkgBasicInfo, PkgInfo>,
    > = HashMap::new();

    let mut dep_relationship_from_cmd_line: Option<DependencyRelationship> =
        None;

    let mut installing_pkg_type: Option<PkgType> = None;
    let mut installing_pkg_name: Option<String> = None;

    let app_pkg_to_work_with =
        get_pkg_info_from_path(&app_dir_to_work_with, true)?;

    // We need to start looking for dependencies outward from the cwd package,
    // and the cwd package itself is considered a candidate.
    add_pkg_to_initial_pkg_to_find_candidates_and_all_candidates(
        &app_pkg_to_work_with,
        &mut initial_pkgs_to_find_candidates,
        &mut all_candidates,
    )?;

    out.normal_line(&format!(
        "{}  Get all installed packages...",
        Emoji("📦", "")
    ));

    all_installed_pkgs = tman_get_all_installed_pkgs_info_of_app(
        tman_config.clone(),
        &app_dir_to_work_with,
        out.clone(),
    )?;

    out.normal_line(&format!(
        "{}  Filter compatible packages...",
        Emoji("🔍", "")
    ));

    filter_compatible_pkgs_to_candidates(
        tman_config.clone(),
        &all_installed_pkgs,
        &mut all_compatible_installed_pkgs,
        &command_data.support,
        out.clone(),
    );

    if command_data.local_path.is_some() {
        // tman install <local_path>

        // Parse the `manifest.json` inside the local_path.
        let local_path_str = command_data.local_path.clone().unwrap();
        let local_path = Path::new(&local_path_str);
        let local_path = local_path.canonicalize().with_context(|| {
            format!(
                "Failed to find the specified local path {}",
                local_path_str
            )
        })?;

        let local_manifest_dir = if local_path.is_dir() {
            local_path.clone()
        } else {
            return Err(anyhow!(
                "The specified local path is not a folder.: {}",
                local_path_str
            ));
        };

        let manifest_json_path =
            local_manifest_dir.join(MANIFEST_JSON_FILENAME);
        if !manifest_json_path.exists() {
            return Err(anyhow!(
                "No manifest.json found in the specified local path: {}",
                local_path_str
            ));
        }

        // Read the `manifest.json` of the local package to obtain its `type`,
        // `name`, and `version`.
        let local_pkg_info =
            get_pkg_info_from_path(&local_manifest_dir, false)?;

        installing_pkg_type =
            Some(local_pkg_info.basic_info.type_and_name.pkg_type);
        installing_pkg_name =
            Some(local_pkg_info.basic_info.type_and_name.name.clone());

        // Currently, tman uses the Rust semver crate, while the cloud store
        // uses the npm semver package. The semver requirement specifications of
        // these two packages are not completely identical. For example:
        //
        // - The Rust semver crate uses "," to separate different ranges,
        //   whereas the npm semver package uses a space (" ") to separate
        //   different requirement ranges.
        // - The npm semver package uses "||" to unify different ranges, but the
        //   Rust semver crate does not support this feature.
        //
        // Since TEN is a cross-language system, it needs to define its own
        // semver requirement specification. This specification could follow
        // either the Rust or npm format or other spec, but in either case, tman
        // or the cloud store would need to make adaptations.
        //
        // Therefore, the current approach is to simplify the specification to
        // only support a single-range semver requirement, which is the common
        // subset of both the npm semver package and the Rust semver crate.
        let local_version_str = local_pkg_info.basic_info.version.to_string();
        if local_version_str.contains("||")
            || local_version_str.chars().any(|c| c.is_whitespace())
            || local_version_str.contains(",")
        {
            return Err(anyhow!(
                    "Invalid version requirement '{}' in local package manifest: contains forbidden characters (||, whitespace, or ,)",
                    local_version_str
                ));
        }
        let installing_pkg_version_req = VersionReq::parse(&local_version_str)?;

        dep_relationship_from_cmd_line = Some(DependencyRelationship {
            type_and_name: PkgTypeAndName {
                pkg_type: app_pkg_to_work_with
                    .basic_info
                    .type_and_name
                    .pkg_type,
                name: app_pkg_to_work_with
                    .basic_info
                    .type_and_name
                    .name
                    .clone(),
            },
            version: app_pkg_to_work_with.basic_info.version.clone(),
            dependency: ManifestDependency::RegistryDependency {
                pkg_type: installing_pkg_type.unwrap(),
                name: installing_pkg_name.clone().unwrap(),
                version_req: installing_pkg_version_req,
            },
        });

        // The `local_pkg_info` needs to be used as a candidate.
        let mut local_pkg_clone = local_pkg_info.clone();

        local_pkg_clone.is_local_dependency = true;
        local_pkg_clone.local_dependency_path = Some(local_path_str.clone());
        local_pkg_clone.local_dependency_base_dir = Some("".to_string());

        all_candidates
            .entry((&local_pkg_clone).into())
            .or_default()
            .insert((&local_pkg_clone).into(), local_pkg_clone.clone());

        initial_pkgs_to_find_candidates.push(local_pkg_clone);
    } else if let Some(package_type_str) = command_data.package_type.as_ref() {
        // tman install <package_type> <package_name>
        let installing_pkg_type_: PkgType = package_type_str.parse()?;

        let (installing_pkg_name_, installing_pkg_version_req_) =
            parse_pkg_name_version_req(
                command_data.package_name.as_ref().unwrap(),
            )?;

        installing_pkg_type = Some(installing_pkg_type_);
        installing_pkg_name = Some(installing_pkg_name_.clone());

        dep_relationship_from_cmd_line = Some(DependencyRelationship {
            type_and_name: PkgTypeAndName {
                pkg_type: app_pkg_to_work_with
                    .basic_info
                    .type_and_name
                    .pkg_type,
                name: app_pkg_to_work_with
                    .basic_info
                    .type_and_name
                    .name
                    .clone(),
            },
            version: app_pkg_to_work_with.basic_info.version.clone(),
            dependency: ManifestDependency::RegistryDependency {
                pkg_type: installing_pkg_type_,
                name: installing_pkg_name_.clone(),
                version_req: installing_pkg_version_req_.clone(),
            },
        });
    }

    out.normal_line(&format!(
        "{}  Attempting to retrieve information about locked packages \
from manifest-lock.json...",
        Emoji("📜", "")
    ));

    // Get the locked pkgs from the lock file in the app folder.
    let locked_pkgs = get_locked_pkgs(&app_dir_to_work_with);

    out.normal_line(&format!(
        "{}  Collect all candidate packages...",
        Emoji("📦", "")
    ));

    // Get all possible candidates according to the input packages and extra
    // dependencies.
    let all_candidates = get_all_candidates_from_deps(
        tman_config.clone(),
        &command_data.support,
        initial_pkgs_to_find_candidates,
        dep_relationship_from_cmd_line
            .as_ref()
            .map(|rel| &rel.dependency),
        &all_compatible_installed_pkgs,
        all_candidates,
        locked_pkgs.as_ref(),
        out.clone(),
    )
    .await?;

    out.normal_line(&format!("{}  Resolving packages...", Emoji("🔍", "")));

    // Find an answer (a dependency tree) that satisfies all dependencies.
    let (usable_model, non_usable_models) = solve_all(
        tman_config.clone(),
        &app_pkg_to_work_with.basic_info.type_and_name.pkg_type,
        &app_pkg_to_work_with.basic_info.type_and_name.name,
        dep_relationship_from_cmd_line.as_ref(),
        &all_candidates,
        locked_pkgs.as_ref(),
        out.clone(),
    )?;

    // If there are answers are found, print out all the answers.
    if tman_config.verbose {
        out.normal_line("\n");
        out.normal_line("Result:");
    }

    if let Some(ref usable_model) = usable_model {
        for result in usable_model {
            if tman_config.verbose {
                out.normal_line(&format!(" {:?}", result));
            }
        }
    }
    if tman_config.verbose {
        out.normal_line("");
    }

    if let Some(ref usable_model) = usable_model {
        // Get the information of the resultant packages.
        let solver_results = extract_solver_results_from_raw_solver_results(
            usable_model,
            &all_candidates,
        )?;

        // Install all the dependencies which the app depends on.
        //
        // The first step is to filter out the package represented by `cwd`, as
        // this package already exists and does not need to be installed.
        let remaining_solver_results = filter_solver_results_by_type_and_name(
            &solver_results,
            Some(&app_pkg_to_work_with.basic_info.type_and_name.pkg_type),
            Some(&app_pkg_to_work_with.basic_info.type_and_name.name),
            false,
        )?;

        // Compare the solver results with the already installed packages to
        // check if there are any installed packages that need to be deleted
        // or replaced.
        let has_conflict = compare_solver_results_with_installed_pkgs(
            &remaining_solver_results,
            &all_installed_pkgs,
            out.clone(),
        );

        if has_conflict && !tman_config.assume_yes {
            if out.is_interactive() {
                // "y" for continuing to install, "n" for stopping.
                let ans = Confirm::new(
                    "Warning!!! Some local packages will be overwritten, \
do you want to continue?",
                )
                .with_default(false)
                .prompt();

                match ans {
                    Ok(true) => {
                        // Continue to install.
                    }
                    Ok(false) => {
                        // Stop to install. Change back to the original current
                        // directory before returning.
                        return Ok(());
                    }
                    Err(_) => {
                        // Stop to install. Change back to the original current
                        // directory before returning.
                        return Ok(());
                    }
                }
            } else {
                out.normal_line("Non-interactive mode, auto-continue...");
            }
        }

        write_pkgs_into_manifest_lock_file(
            &remaining_solver_results,
            &app_dir_to_work_with,
            out.clone(),
        )?;

        install_solver_results_in_app_folder(
            tman_config.clone(),
            &command_data,
            &remaining_solver_results,
            &app_dir_to_work_with,
            out.clone(),
        )
        .await?;

        // Write the installing package info to manifest.json.
        //
        // If it's in `local_path` mode, update `manifest.json` by writing `{
        // "path": "<local_path>" }`. Otherwise, write `type`, `name`, and
        // `version`.
        if command_data.local_path.is_some() {
            // tman install <local_path>

            if installing_pkg_type.is_some() && installing_pkg_name.is_some() {
                let mut origin_cwd_pkg =
                    get_pkg_info_from_path(&specified_cwd, true)?;

                write_installing_pkg_into_manifest_file(
                    &mut origin_cwd_pkg,
                    &solver_results,
                    &installing_pkg_type.unwrap(),
                    &installing_pkg_name.unwrap(),
                    Some(command_data.local_path.clone().unwrap()),
                )?;
            }
        } else {
            // tman install <package_type> <package_name>

            if installing_pkg_type.is_some() && installing_pkg_name.is_some() {
                let mut origin_cwd_pkg =
                    get_pkg_info_from_path(&specified_cwd, true)?;

                write_installing_pkg_into_manifest_file(
                    &mut origin_cwd_pkg,
                    &solver_results,
                    &installing_pkg_type.unwrap(),
                    &installing_pkg_name.unwrap(),
                    None,
                )?;
            }
        }

        out.normal_line(&format!(
            "{}  Install successfully in {}",
            Emoji("🏆", ":-)"),
            HumanDuration(started.elapsed())
        ));

        Ok(())
    } else {
        // The last model always represents the optimal version.
        //
        // The first error messages (i.e., non_usable_models.first()) might be
        // changed when we run `tman install` in an app folder, if the app
        // contains a conflicted dependency which has many candidates. The
        // different error messages are as follows.
        //
        // ```
        // ❌  Error: Select more than 1 version of '[system]ten_runtime':
        //     '@0.2.0' introduced by '[extension]agora_rtm@0.1.0', and '@0.3.1'
        //     introduced by '[system]ten_runtime_go@0.3.0'
        // ```
        //
        // ```
        // ❌  Error: Select more than 1 version of '[system]ten_runtime':
        //     '@0.2.0' introduced by '[extension]agora_rtm@0.1.2', and '@0.3.0'
        //     introduced by '[system]ten_runtime_go@0.3.0'
        // ```
        //
        // The introducer pkg are different in the two error messages,
        // `agora_rtm@0.1.0` vs `agora_rtm@0.1.2`.
        //
        // The reason is that the candidates are stored in a HashSet, and when
        // they are traversed and written to the `depends_on_declared` field in
        // `input.lp``, the order of writing is not guaranteed. Ex, the
        // `agora_rtm` has three candidates, 0.1.0, 0.1.1, 0.1.2. The
        // content in the input.ip might be:
        //
        // case 1:
        // ```
        // depends_on_declared("app", "ten_agent", "0.4.0", "extension", "agora_rtm", "0.1.0").
        // depends_on_declared("app", "ten_agent", "0.4.0", "extension", "agora_rtm", "0.1.1").
        // depends_on_declared("app", "ten_agent", "0.4.0", "extension", "agora_rtm", "0.1.2").
        // ```
        //
        // or
        //
        // case 2:
        // ```
        // depends_on_declared("app", "ten_agent", "0.4.0", "extension", "agora_rtm", "0.1.2").
        // depends_on_declared("app", "ten_agent", "0.4.0", "extension", "agora_rtm", "0.1.0").
        // depends_on_declared("app", "ten_agent", "0.4.0", "extension", "agora_rtm", "0.1.1").
        // ```
        // Due to the different order of input data, clingo may start searching
        // from different points. However, clingo will only output increasingly
        // better models. Therefore, if we select the first `non_usable_models`,
        // it is often inconsistent. But if we select the last model, it is
        // usually consistent, representing the optimal error model that clingo
        // can find. This phenomenon is similar to the gradient descent process
        // in neural networks that seeks local optima. Thus, we should select
        // the last model to obtain the optimal error model and achieve stable
        // results.
        if let Some(non_usable_model) = non_usable_models.last() {
            // Extract introducer relations, and parse the error message.
            if let Ok(conflict_info) = parse_error_statement(non_usable_model) {
                // Print the error message and dependency chains.
                print_conflict_info(
                    &conflict_info,
                    &extract_introducer_relations_from_raw_solver_results(
                        non_usable_model,
                        &all_candidates,
                    )?,
                    &all_candidates,
                    out,
                )?;

                // Since there is an error, we need to exit.
                return Err(anyhow!("Dependency resolution failed."));
            }
        }

        // If there are no error models or unable to parse, return a generic
        // error.
        Err(anyhow!(
            "Dependency resolution failed without specific error details."
        ))
    }
}
