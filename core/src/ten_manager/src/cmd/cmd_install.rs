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
    str::FromStr,
    time::Instant,
};

use anyhow::Result;
use clap::{Arg, ArgMatches, Command};
use console::Emoji;
use indicatif::HumanDuration;
use inquire::Confirm;

use ten_rust::pkg_info::{
    dependencies::PkgDependency,
    get_pkg_info_from_path,
    pkg_type::PkgType,
    pkg_type_and_name::PkgTypeAndName,
    supports::{Arch, Os, PkgSupport},
    PkgInfo,
};
use ten_rust::pkg_info::{
    manifest::parse_manifest_in_folder, pkg_basic_info::PkgBasicInfo,
};

use crate::{
    config::TmanConfig,
    constants::{APP_DIR_IN_DOT_TEN_DIR, DOT_TEN_DIR},
    dep_and_candidate::get_all_candidates_from_deps,
    error::TmanError,
    fs::check_is_app_folder,
    install::{
        compare_solver_results_with_existed_pkgs,
        filter_compatible_pkgs_to_candidates, is_installing_package_standalone,
        is_package_installable_in_cwd, update_package_manifest,
        write_pkgs_into_lock,
    },
    log::tman_verbose_println,
    manifest_lock::parse_manifest_lock_in_folder,
    package_info::tman_get_all_existed_pkgs_info_of_app,
    solver::{
        introducer::extract_introducer_relations_from_raw_solver_results,
        solve::{solve_all, DependencyRelationship},
        solver_error::{parse_error_statement, print_conflict_info},
        solver_result::{
            extract_solver_results_from_raw_solver_results,
            filter_solver_results_by_type_and_name,
            install_solver_results_in_app_folder,
            install_solver_results_in_standalone_mode,
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
    };

    let _ = cmd.support.set_defaults();

    if let Some(mode_str) = sub_cmd_args.get_one::<String>("LOCAL_INSTALL_MODE")
    {
        cmd.local_install_mode = mode_str.parse()?;
    }

    Ok(cmd)
}

pub async fn execute_cmd(
    tman_config: &TmanConfig,
    command_data: InstallCommand,
) -> Result<()> {
    tman_verbose_println!(tman_config, "Executing install command");
    tman_verbose_println!(tman_config, "{:?}", command_data);
    tman_verbose_println!(tman_config, "{:?}", tman_config);

    let started = Instant::now();

    let cwd = crate::fs::get_cwd()?;

    // The package affected by tman install command, which is the root declared
    // in the resolver.
    //
    // 1. If installing a package in standalone mode, the affected package is
    //    the installed package itself, ex: app or extension.
    //
    // 2. Otherwise, the affected package is always the app, as the installing
    //    package must be installed in a TEN app in this case.
    let affected_pkg_name;
    let mut affected_pkg_type = PkgType::App;

    let mut is_standalone_installing = false;
    let mut preinstall_chdir_path: Option<PathBuf> = None;

    // If `tman install` is run within the scope of an app, then the app and
    // those addons (extensions, ...) installed in the app directory are all
    // considered initial_input_pkgs.
    let mut initial_input_pkgs = vec![];
    let mut all_candidates: HashMap<
        PkgTypeAndName,
        HashMap<PkgBasicInfo, PkgInfo>,
    > = HashMap::new();

    // 'all_existing_local_pkgs' contains all the packages which are already
    // located in the app directory.
    //
    // After the completed dependency tree is resolved,
    // 'all_existing_local_pkgs' will be compared with the solver results:
    //
    // *) If some of these packages are not included in the dependency tree,
    // then users will be prompted that these packages can be added to the
    // dependencies.
    //
    // *) If some of these packages are not compatible with packages in
    // the dependency tree, then users will be questioned whether to overwrite
    // them with the new packages or quit the installation.
    //
    // 'all_existing_local_pkgs' is only used in non-standalone mode and is not
    // used for standalone installation (execute 'tman install' in an extension
    // folder).
    let mut all_existing_local_pkgs: Vec<PkgInfo> = vec![];

    // The initial value of extra_dependencies is the package specified to be
    // installed via the command line (if any).
    let mut extra_dependencies_specified_in_cmd_line = vec![];

    // `extra_dependency_relationships` contain TEN packages, and each TEN
    // package is the main entity depended upon by its corresponding
    // extra_dependencies."
    let mut extra_dependency_relationships = vec![];

    // The locked_pkgs comes from a lock file in the app folder.
    let mut locked_pkgs: Option<HashMap<PkgTypeAndName, PkgInfo>> = None;

    let mut app_pkg: Option<PkgInfo> = None;
    let mut installing_pkg_type: Option<PkgType> = None;
    let mut installing_pkg_name: Option<String> = None;

    if let Some(package_type_str) = command_data.package_type.as_ref() {
        // Case 1: tman install <package_type> <package_name>

        let installing_pkg_type_: PkgType = package_type_str.parse()?;
        let (installing_pkg_name_, installing_pkg_version_req_) =
            parse_pkg_name_version_req(
                command_data.package_name.as_ref().unwrap(),
            )?;

        installing_pkg_type = Some(installing_pkg_type_);
        installing_pkg_name = Some(installing_pkg_name_.clone());

        // First, check that the package we want to install can be installed
        // within the current directory structure.
        is_package_installable_in_cwd(&cwd, &installing_pkg_type_)?;

        is_standalone_installing =
            is_installing_package_standalone(&cwd, &installing_pkg_type_)?;
        if is_standalone_installing {
            affected_pkg_type = installing_pkg_type_;
            affected_pkg_name = installing_pkg_name_.clone();

            preinstall_chdir_path =
                Some(PathBuf::from_str(&installing_pkg_name_)?);
        } else {
            // If it is not a standalone install, then the `cwd` must be within
            // the base directory of a TEN app.
            let app_pkg_ = get_pkg_info_from_path(&cwd, true)?;
            affected_pkg_name = app_pkg_.basic_info.type_and_name.name.clone();

            // Push the app itself into the initial_input_pkgs.
            initial_input_pkgs.push(app_pkg_.clone());

            all_existing_local_pkgs =
                tman_get_all_existed_pkgs_info_of_app(tman_config, &cwd)?;

            // Add existing packages into all_candidates only if the compatible
            // score of the package is >= 0.
            filter_compatible_pkgs_to_candidates(
                tman_config,
                &all_existing_local_pkgs,
                &mut all_candidates,
                &command_data.support,
            );

            let extra_dependency_relationship = DependencyRelationship {
                type_and_name: PkgTypeAndName {
                    pkg_type: app_pkg_.basic_info.type_and_name.pkg_type,
                    name: app_pkg_.basic_info.type_and_name.name.clone(),
                },
                version: app_pkg_.basic_info.version.clone(),
                dependency: PkgDependency {
                    type_and_name: PkgTypeAndName {
                        pkg_type: installing_pkg_type_,
                        name: installing_pkg_name_.clone(),
                    },
                    version_req: installing_pkg_version_req_.clone(),
                    path: None,
                    base_dir: None,
                },
            };
            extra_dependency_relationships.push(extra_dependency_relationship);

            app_pkg = Some(app_pkg_);
        }

        let dep = PkgDependency::new(
            installing_pkg_type_,
            installing_pkg_name_.clone(),
            installing_pkg_version_req_,
        );
        extra_dependencies_specified_in_cmd_line.push(dep);
    } else {
        // Case 2: tman install

        let manifest = parse_manifest_in_folder(&cwd)?;
        affected_pkg_type = manifest.type_and_name.pkg_type;
        affected_pkg_name = manifest.type_and_name.name.clone();

        match affected_pkg_type {
            PkgType::App => {
                check_is_app_folder(&cwd)?;

                // Push the app itself into the initial_input_pkgs.
                initial_input_pkgs.push(get_pkg_info_from_path(&cwd, true)?);

                all_existing_local_pkgs =
                    tman_get_all_existed_pkgs_info_of_app(tman_config, &cwd)?;
                filter_compatible_pkgs_to_candidates(
                    tman_config,
                    &all_existing_local_pkgs,
                    &mut all_candidates,
                    &command_data.support,
                );
            }

            PkgType::Extension => {
                // Install all dependencies of the extension package, but a APP
                // folder should be created first.
                preinstall_chdir_path =
                    Some(Path::new(DOT_TEN_DIR).join(APP_DIR_IN_DOT_TEN_DIR));

                if let Some(ref path) = preinstall_chdir_path {
                    fs::create_dir_all(path)?;
                }

                initial_input_pkgs.push(get_pkg_info_from_path(&cwd, true)?);

                filter_compatible_pkgs_to_candidates(
                    tman_config,
                    &initial_input_pkgs,
                    &mut all_candidates,
                    &command_data.support,
                );
            }

            _ => {
                return Err(TmanError::Custom(
                    "Current folder should be a TEN APP or Extension package."
                        .to_string(),
                )
                .into());
            }
        }
    }

    // Get the locked pkgs from the lock file in the app folder.
    if !is_standalone_installing {
        let mut app_dir = cwd.clone();
        if let Some(preinstall_chdir_path) = &preinstall_chdir_path {
            app_dir = cwd.join(preinstall_chdir_path);
        }

        let manifest_lock = parse_manifest_lock_in_folder(&app_dir);

        if manifest_lock.is_ok() {
            locked_pkgs = Some(manifest_lock.unwrap().get_pkgs());
        }
    }

    // Get all possible candidates according to the input packages and extra
    // dependencies.
    let all_candidates = get_all_candidates_from_deps(
        tman_config,
        &command_data.support,
        initial_input_pkgs,
        all_candidates,
        &extra_dependencies_specified_in_cmd_line,
        locked_pkgs.as_ref(),
    )
    .await?;

    println!("{}  Resolving packages...", Emoji("🔍", ""),);

    // Find the answer that satisfies all dependencies.
    let (usable_model, non_usable_models) = solve_all(
        tman_config,
        &affected_pkg_name,
        &affected_pkg_type,
        &extra_dependency_relationships,
        &all_candidates,
        locked_pkgs.as_ref(),
    )?;

    // Print out the answer.
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

        if is_standalone_installing {
            // If installing a standalone package (ex: app or extension), the
            // package itself must be installed first.

            let affected_pkg = filter_solver_results_by_type_and_name(
                &solver_results,
                Some(&affected_pkg_type),
                Some(&affected_pkg_name),
                true,
            )?;

            if affected_pkg.is_empty() {
                return Err(TmanError::Custom(format!(
                    "Failed to find any of {}:{}.",
                    affected_pkg_type, affected_pkg_name,
                ))
                .into());
            }
            if affected_pkg.len() > 1 {
                return Err(TmanError::Custom(format!(
                    "Found the possibility of multiple {}:{} being incorrect.",
                    affected_pkg_type, affected_pkg_name
                ))
                .into());
            }

            // TODO(Liu): Do some check after the installation. Ex: the package
            // is installed in the template mode, however the
            // package might not have a manifest.json template file
            // to rerender the package name. In this
            // case, the installed package is invalid, because as a spec of TEN
            // package, the installed folder name should be the same as the
            // package name. Anyway, the package name in
            // manifest.json should be forced to replace with the
            // `package-name` arg?
            install_solver_results_in_standalone_mode(
                tman_config,
                &command_data,
                &affected_pkg,
                &cwd,
            )
            .await?;
        }

        // If we need to switch to a specific folder before installing other
        // packages, do so now.
        let mut app_dir = cwd.clone();
        if let Some(preinstall_chdir_path) = &preinstall_chdir_path {
            env::set_current_dir(cwd.join(preinstall_chdir_path))?;
            app_dir = cwd.join(preinstall_chdir_path);
        }

        // We have the following three cases:
        //
        // Case 1: install a standalone app, i.e.: tman install app ...
        // Case 2: install a standalone extension, i.e.: tman install extension
        // ... Case 3: install dependencies of a package, i.e.: run
        // `tman install` in a package folder, which is either a TEN app
        // or extension.
        //
        // In case 2, after the package has been installed, its dependencies can
        // NOT be installed automatically, as the installation directory
        // of dependencies is different in this case.
        if affected_pkg_type == PkgType::App || !is_standalone_installing {
            // Install all the dependencies which the app depends on.
            let remaining_solver_results =
                filter_solver_results_by_type_and_name(
                    &solver_results,
                    Some(&affected_pkg_type),
                    Some(&affected_pkg_name),
                    false,
                )?;

            // Compare the remaining_solver_results with the
            // all_existing_local_pkgs to check if there are any
            // local packages that need to be deleted or replaced.
            let has_conflict = compare_solver_results_with_existed_pkgs(
                &remaining_solver_results,
                &all_existing_local_pkgs,
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

            write_pkgs_into_lock(&remaining_solver_results, &app_dir)?;

            install_solver_results_in_app_folder(
                tman_config,
                &command_data,
                &remaining_solver_results,
                &app_dir,
            )
            .await?;
        }

        // Change back to the original folder.
        if preinstall_chdir_path.is_some() {
            env::set_current_dir(cwd)?;
        }

        // Write package info to manifest.json.
        if let Some(mut app_pkg) = app_pkg {
            if installing_pkg_type.is_some() && installing_pkg_name.is_some() {
                let suitable_pkgs = filter_solver_results_by_type_and_name(
                    &solver_results,
                    installing_pkg_type.as_ref(),
                    installing_pkg_name.as_ref(),
                    true,
                )?;

                if suitable_pkgs.is_empty() {
                    return Err(TmanError::Custom(format!(
                        "Failed to find any of {}:{}.",
                        installing_pkg_type.unwrap(),
                        installing_pkg_name.unwrap(),
                    ))
                    .into());
                }

                if suitable_pkgs.len() > 1 {
                    return Err(TmanError::Custom(format!(
                    "Found the possibility of multiple {}:{} being incorrect.",
                    installing_pkg_type.unwrap(),
                    installing_pkg_name.unwrap()
                ))
                    .into());
                }

                update_package_manifest(&mut app_pkg, suitable_pkgs[0])?;
            }
        }

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
                return Err(TmanError::Custom(
                    "Dependency resolution failed.".to_string(),
                )
                .into());
            }
        }

        // If there are no error models or unable to parse, return a generic
        // error.
        Err(TmanError::Custom(
            "Dependency resolution failed without specific error details."
                .to_string(),
        )
        .into())
    }
}
