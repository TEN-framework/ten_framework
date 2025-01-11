//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::{
    collections::HashMap,
    env,
    fs::{self},
    path::{Path, PathBuf},
    time::Instant,
};

use anyhow::{anyhow, Result};
use clap::{Arg, ArgMatches, Command};
use console::Emoji;
use indicatif::HumanDuration;
use inquire::Confirm;

use ten_rust::pkg_info::{
    constants::MANIFEST_JSON_FILENAME, pkg_basic_info::PkgBasicInfo,
};
use ten_rust::pkg_info::{
    dependencies::PkgDependency,
    get_pkg_info_from_path,
    pkg_type::PkgType,
    pkg_type_and_name::PkgTypeAndName,
    supports::{Arch, Os, PkgSupport},
    PkgInfo,
};

use crate::{
    config::TmanConfig,
    constants::{APP_DIR_IN_DOT_TEN_DIR, DOT_TEN_DIR},
    dep_and_candidate::get_all_candidates_from_deps,
    fs::{check_is_extension_folder, find_nearest_app_dir},
    install::{
        compare_solver_results_with_existed_pkgs,
        filter_compatible_pkgs_to_candidates,
        write_installing_pkg_into_manifest_file,
        write_pkgs_into_manifest_lock_file,
    },
    log::tman_verbose_println,
    manifest_lock::parse_manifest_lock_in_folder,
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
    pub support: PkgSupport,
    pub local_install_mode: LocalInstallMode,
    pub standalone: bool,
}

pub fn create_sub_cmd(args_cfg: &crate::cmd_line::ArgsCfg) -> Command {
    Command::new("install")
        .about("Install a package")
        .arg(
            Arg::new("PACKAGE_TYPE")
                .help("The type of the package")
                .value_parser(args_cfg.pkg_type.possible_values.clone())
                .required(false)
                // If PACKAGE_TYPE is provided, PACKAGE_NAME must be too.
                .requires("PACKAGE_NAME"),
        )
        .arg(
            Arg::new("PACKAGE_NAME")
                .help("The name of the package")
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
                .help("Install in standalone mode, only for extension package.")
                .action(clap::ArgAction::SetTrue)
                .required(false),
        )
}

pub fn parse_sub_cmd(sub_cmd_args: &ArgMatches) -> Result<InstallCommand> {
    let mut cmd = InstallCommand {
        package_type: sub_cmd_args.get_one::<String>("PACKAGE_TYPE").cloned(),
        package_name: sub_cmd_args.get_one::<String>("PACKAGE_NAME").cloned(),
        support: PkgSupport {
            os: sub_cmd_args
                .get_one::<String>("OS")
                .and_then(|s| s.parse::<Os>().ok()),
            arch: sub_cmd_args
                .get_one::<String>("ARCH")
                .and_then(|s| s.parse::<Arch>().ok()),
        },
        local_install_mode: LocalInstallMode::Invalid,
        standalone: false,
    };

    let _ = cmd.support.set_defaults();

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

/// In standalone mode, if `.ten/app/manifest.json` does not exist, create one.
fn prepare_standalone_app_dir(extension_dir: &Path) -> Result<PathBuf> {
    let dot_ten_app_dir =
        extension_dir.join(DOT_TEN_DIR).join(APP_DIR_IN_DOT_TEN_DIR);
    if !dot_ten_app_dir.exists() {
        fs::create_dir_all(&dot_ten_app_dir)?;
    }

    let manifest_json = dot_ten_app_dir.join(MANIFEST_JSON_FILENAME);
    if !manifest_json.exists() {
        // Create a basic `manifest.json`.
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

pub async fn execute_cmd(
    tman_config: &TmanConfig,
    command_data: InstallCommand,
) -> Result<()> {
    tman_verbose_println!(tman_config, "Executing install command");
    tman_verbose_println!(tman_config, "{:?}", command_data);
    tman_verbose_println!(tman_config, "{:?}", tman_config);

    let started = Instant::now();

    let original_cwd = crate::fs::get_cwd()?;

    // Path logic for standalone mode and non-standalone mode.
    let app_dir_to_work_with = if command_data.standalone {
        // If it is standalone mode, it can only be executed in the extension
        // directory.
        check_is_extension_folder(&original_cwd)?;

        let dot_ten_app_dir_path = prepare_standalone_app_dir(&original_cwd)?;

        env::set_current_dir(&dot_ten_app_dir_path)?;

        dot_ten_app_dir_path.clone()
    } else {
        // Non-standalone mode can only be executed in the extension directory.
        // If it is an extension, it should search upwards for the nearest app;
        // if it is an app, it can be used directly.
        let app_dir = find_nearest_app_dir(original_cwd.clone())?;

        env::set_current_dir(&app_dir)?;

        app_dir
    };

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
    let mut all_installed_pkgs: Vec<PkgInfo> = vec![];
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
    let mut origin_cwd_pkg = get_pkg_info_from_path(&original_cwd, true)?;

    // We need to start looking for dependencies outward from the cwd package,
    // and the cwd package itself is considered a candidate.
    add_pkg_to_initial_pkg_to_find_candidates_and_all_candidates(
        &app_pkg_to_work_with,
        &mut initial_pkgs_to_find_candidates,
        &mut all_candidates,
    )?;

    if let Some(package_type_str) = command_data.package_type.as_ref() {
        // Case 1: tman install <package_type> <package_name>
        //
        // The `cwd` must be the base directory of a TEN app.

        let installing_pkg_type_: PkgType = package_type_str.parse()?;

        let (installing_pkg_name_, installing_pkg_version_req_) =
            parse_pkg_name_version_req(
                command_data.package_name.as_ref().unwrap(),
            )?;

        installing_pkg_type = Some(installing_pkg_type_);
        installing_pkg_name = Some(installing_pkg_name_.clone());

        all_installed_pkgs = tman_get_all_installed_pkgs_info_of_app(
            tman_config,
            &app_dir_to_work_with,
        )?;

        filter_compatible_pkgs_to_candidates(
            tman_config,
            &all_installed_pkgs,
            &mut all_compatible_installed_pkgs,
            &command_data.support,
        );

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
            dependency: PkgDependency {
                type_and_name: PkgTypeAndName {
                    pkg_type: installing_pkg_type_,
                    name: installing_pkg_name_.clone(),
                },
                version_req: installing_pkg_version_req_.clone(),
                path: None,
                base_dir: None,
            },
        });
    } else {
        // Case 2: tman install

        match origin_cwd_pkg.basic_info.type_and_name.pkg_type {
            PkgType::App => {
                // The TEN app itself is also a package. Extensions can declare
                // dependencies on a specific version of an app, so the app also
                // needs to be included in the package list for dependency tree
                // calculation.

                all_installed_pkgs = tman_get_all_installed_pkgs_info_of_app(
                    tman_config,
                    &app_dir_to_work_with,
                )?;

                filter_compatible_pkgs_to_candidates(
                    tman_config,
                    &all_installed_pkgs,
                    &mut all_compatible_installed_pkgs,
                    &command_data.support,
                );
            }

            PkgType::Extension => {
                // Install all dependencies of the extension package, but a APP
                // folder (i.e., `.ten/app`) should be created first.
                filter_compatible_pkgs_to_candidates(
                    tman_config,
                    &initial_pkgs_to_find_candidates,
                    &mut all_compatible_installed_pkgs,
                    &command_data.support,
                );
            }

            _ => {
                return Err(anyhow!(
                    "Current folder should be a TEN APP or Extension package."
                        .to_string(),
                ));
            }
        }
    }

    // Get the locked pkgs from the lock file in the app folder.
    let locked_pkgs = get_locked_pkgs(&app_dir_to_work_with);

    // Get all possible candidates according to the input packages and extra
    // dependencies.
    let all_candidates = get_all_candidates_from_deps(
        tman_config,
        &command_data.support,
        initial_pkgs_to_find_candidates,
        dep_relationship_from_cmd_line
            .as_ref()
            .map(|rel| &rel.dependency),
        &all_compatible_installed_pkgs,
        all_candidates,
        locked_pkgs.as_ref(),
    )
    .await?;

    println!("{}  Resolving packages...", Emoji("🔍", ""),);

    // Find an answer (a dependency tree) that satisfies all dependencies.
    let (usable_model, non_usable_models) = solve_all(
        tman_config,
        &app_pkg_to_work_with.basic_info.type_and_name.pkg_type,
        &app_pkg_to_work_with.basic_info.type_and_name.name,
        dep_relationship_from_cmd_line.as_ref(),
        &all_candidates,
        locked_pkgs.as_ref(),
    )?;

    // If there are answers are found, print out all the answers.
    tman_verbose_println!(tman_config, "\n");
    tman_verbose_println!(tman_config, "Result:");
    if let Some(ref usable_model) = usable_model {
        for result in usable_model {
            tman_verbose_println!(tman_config, " {:?}", result);
        }
    }
    tman_verbose_println!(tman_config, "");

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
        let has_conflict = compare_solver_results_with_existed_pkgs(
            &remaining_solver_results,
            &all_installed_pkgs,
        );

        if has_conflict && !tman_config.assume_yes {
            // "y" for continuing to install, "n" for stopping.
            let ans = Confirm::new(
                "Warning!!! Some local packages will be overwritten, \
                do you want to continue?",
            )
            .with_default(false)
            .prompt();

            match ans {
                std::result::Result::Ok(true) => {
                    // continue to install
                }
                std::result::Result::Ok(false) => {
                    // stop
                    return Ok(());
                }
                Err(_) => {
                    // stop
                    return Ok(());
                }
            }
        }

        write_pkgs_into_manifest_lock_file(
            &remaining_solver_results,
            &app_dir_to_work_with,
        )?;

        install_solver_results_in_app_folder(
            tman_config,
            &command_data,
            &remaining_solver_results,
            &app_dir_to_work_with,
        )
        .await?;

        // Write the installing package info to manifest.json.
        if installing_pkg_type.is_some() && installing_pkg_name.is_some() {
            write_installing_pkg_into_manifest_file(
                &mut origin_cwd_pkg,
                &solver_results,
                &installing_pkg_type.unwrap(),
                &installing_pkg_name.unwrap(),
            )?;
        }

        // Change back to the original folder.
        env::set_current_dir(original_cwd)?;

        println!(
            "{}  Install successfully in {}",
            Emoji("🏆", ":-)"),
            HumanDuration(started.elapsed())
        );

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
