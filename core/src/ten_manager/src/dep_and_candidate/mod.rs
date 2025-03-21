//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::Arc;

use anyhow::{anyhow, Context, Result};
use semver::{Version, VersionReq};

use ten_rust::pkg_info::dependencies::PkgDependency;
use ten_rust::pkg_info::pkg_basic_info::PkgBasicInfo;
use ten_rust::pkg_info::pkg_type::PkgType;
use ten_rust::pkg_info::pkg_type_and_name::PkgTypeAndName;
use ten_rust::pkg_info::supports::{
    is_pkg_supports_compatible_with, PkgSupport, SupportsDisplay,
};
use ten_rust::pkg_info::{get_pkg_info_from_path, PkgInfo};

use super::config::TmanConfig;
use super::registry::get_package_list;
use crate::output::TmanOutput;
use crate::registry::pkg_list_cache::{is_superset_of, PackageListCache};

// TODO(Wei): Should use the union of the semantic versioning rather than the
// union of all version requirements.
#[derive(Default)]
struct MergedVersionReq {
    version_reqs: HashSet<VersionReq>,
}

impl MergedVersionReq {
    fn new(version_req: &VersionReq) -> Self {
        MergedVersionReq {
            version_reqs: HashSet::from([version_req.clone()]),
        }
    }

    /// If there is already the same version requirement, it means there is no
    /// new information, so return false. Otherwise, return true to let the
    /// caller know that there has been a change.
    fn merge(&mut self, version_req: &VersionReq) -> Result<bool> {
        if self.version_reqs.contains(version_req) {
            return Ok(false);
        }

        self.version_reqs.insert(version_req.clone());

        Ok(true)
    }
}

/// If the contents of merged_dependencies are the same before and after the
/// merge, it means merged_dependencies has not changed, so return false.
/// Otherwise, it means there has been a change, so return true. This allows the
/// caller to know that there may be new content and that some actions may need
/// to be taken.
fn merge_dependency_to_dependencies(
    merged_dependencies: &mut HashMap<PkgTypeAndName, MergedVersionReq>,
    dependency: &PkgDependency,
) -> Result<bool> {
    let searched_target = PkgTypeAndName {
        pkg_type: dependency.type_and_name.pkg_type,
        name: dependency.type_and_name.name.clone(),
    };

    let mut changed = true;

    if let Some(existed_dependency) =
        merged_dependencies.get_mut(&searched_target)
    {
        changed = existed_dependency.merge(&dependency.version_req)?;
    } else {
        // This is the first time seeing this dependency.
        merged_dependencies.insert(
            PkgTypeAndName {
                pkg_type: dependency.type_and_name.pkg_type,
                name: dependency.type_and_name.name.clone(),
            },
            MergedVersionReq::new(&dependency.version_req),
        );
    }

    Ok(changed)
}

fn process_local_dependency_to_get_candidate(
    dependency: &PkgDependency,
    all_candidates: &mut HashMap<
        PkgTypeAndName,
        HashMap<PkgBasicInfo, PkgInfo>,
    >,
    new_pkgs_to_be_searched: &mut Vec<PkgInfo>,
) -> Result<()> {
    // Enforce having only one candidate: the package info parsed from
    // the specified path.

    assert!(dependency.path.is_some(), "Should not happen.");

    // Construct a `PkgInfo` to represent the package corresponding to
    // the specified path.
    let base_dir = dependency.base_dir.as_deref().unwrap_or("");
    let path = dependency.path.as_deref().unwrap();

    let abs_path =
        Path::new(base_dir)
            .join(path)
            .canonicalize()
            .with_context(|| {
                format!("Failed to canonicalize path: {} + {}", base_dir, path)
            })?;

    let mut pkg_info = get_pkg_info_from_path(&abs_path, false)?;

    pkg_info.is_local_dependency = true;
    pkg_info.local_dependency_path = dependency.path.clone();
    pkg_info.local_dependency_base_dir = dependency.base_dir.clone();

    let candidate_map = all_candidates.entry((&pkg_info).into()).or_default();

    candidate_map.insert((&pkg_info).into(), pkg_info.clone());

    new_pkgs_to_be_searched.push(pkg_info);

    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn process_non_local_dependency_to_get_candidate(
    tman_config: Arc<TmanConfig>,
    support: &PkgSupport,
    dependency: &PkgDependency,
    all_compatible_installed_pkgs: &HashMap<
        PkgTypeAndName,
        HashMap<PkgBasicInfo, PkgInfo>,
    >,
    all_candidates: &mut HashMap<
        PkgTypeAndName,
        HashMap<PkgBasicInfo, PkgInfo>,
    >,
    new_pkgs_to_be_searched: &mut Vec<PkgInfo>,
    pkg_list_cache: &mut PackageListCache,
    out: Arc<Box<dyn TmanOutput>>,
) -> Result<()> {
    // First, check whether the package list cache has already processed the
    // same or a more permissive version requirement combination.
    let key: PkgTypeAndName = dependency.into();
    if !pkg_list_cache.check_and_update(&key, &dependency.version_req) {
        // If a covering combination already exists in the package list cache,
        // skip calling `get_package_list`.
        return Ok(());
    }

    // Retrieve all packages from the registry that meet the specified
    // criteria.
    let results = get_package_list(
        tman_config.clone(),
        Some(dependency.type_and_name.pkg_type),
        Some(dependency.type_and_name.name.clone()),
        // With the current design, if there is new information, it will
        // definitely be only this version requirement. Although there may be
        // some overlap with previous version requirements, given the current
        // design, this is the best we can do for now. The answer won't be
        // wrong, but the efficiency might be somewhat lower.
        Some(dependency.version_req.clone()),
        None,
        None, // Retrieve all packages.
        out.clone(),
    )
    .await?;

    let mut candidate_pkg_infos: Vec<PkgInfo> = vec![];

    // Put all the results returned by the registry into
    // `candidate_pkg_infos`, making it convenient for `supports`
    // filtering later.
    for result in results {
        let mut candidate_pkg_info: PkgInfo = (&result).into();

        candidate_pkg_info.url = result.download_url;
        candidate_pkg_info.is_installed = false;

        candidate_pkg_infos.push(candidate_pkg_info);
    }

    // Find packages from the installed packages that meet the specified
    // criteria and add them to the candidate_pkg_infos. This action ensures
    // that the dependencies within those packages are processed.
    if let Some(candidates) =
        all_compatible_installed_pkgs.get(&dependency.into())
    {
        for candidate in candidates {
            if dependency.version_req.matches(&candidate.0.version) {
                candidate_pkg_infos.push(candidate.1.clone());
            }
        }
    }

    // Filter suitable candidate packages according to `supports`.
    for mut candidate_pkg_info in candidate_pkg_infos {
        let compatible_score = is_pkg_supports_compatible_with(
            &candidate_pkg_info.basic_info.supports,
            support,
        );

        // A package is considered a candidate only when compatible_score >= 0.
        if compatible_score >= 0 {
            // Record the package's compatible_score so that we can later select
            // the most appropriate one.
            candidate_pkg_info.compatible_score = compatible_score;

            if tman_config.verbose {
                out.normal_line(&format!(
                    "=> Found a candidate: {}:{}@{}[{}]",
                    candidate_pkg_info.basic_info.type_and_name.pkg_type,
                    candidate_pkg_info.basic_info.type_and_name.name,
                    candidate_pkg_info.basic_info.version,
                    SupportsDisplay(&candidate_pkg_info.basic_info.supports),
                ));
            }

            all_candidates.entry(dependency.into()).or_default().insert(
                (&candidate_pkg_info).into(),
                candidate_pkg_info.clone(),
            );

            new_pkgs_to_be_searched.push(candidate_pkg_info);
        }
    }

    Ok(())
}

struct DependenciesContext<'a> {
    tman_config: Arc<TmanConfig>,
    support: &'a PkgSupport,
    merged_dependencies: &'a mut HashMap<PkgTypeAndName, MergedVersionReq>,
    all_compatible_installed_pkgs:
        &'a HashMap<PkgTypeAndName, HashMap<PkgBasicInfo, PkgInfo>>,
    all_candidates:
        &'a mut HashMap<PkgTypeAndName, HashMap<PkgBasicInfo, PkgInfo>>,
    new_pkgs_to_be_searched: &'a mut Vec<PkgInfo>,
    pkg_list_cache: &'a mut PackageListCache,
}

/// Asynchronously processes dependencies to get candidate packages.
///
/// # Parameters
///
/// - `all_candidates`: A mutable reference to a map where the keys are package
///   type & name and the values are sets of candidate package information. This
///   map will be updated with potential candidate packages for each dependency.
///
/// # Returns
///
/// A `Result` indicating the success or failure of the operation.
///
/// # Errors
///
/// This function may return an error if there is an issue processing the
/// dependencies.
async fn process_dependencies_to_get_candidates(
    ctx: &mut DependenciesContext<'_>,
    input_dependencies: &Vec<PkgDependency>,
    out: Arc<Box<dyn TmanOutput>>,
) -> Result<()> {
    for dependency in input_dependencies {
        if dependency.is_local() {
            process_local_dependency_to_get_candidate(
                dependency,
                ctx.all_candidates,
                ctx.new_pkgs_to_be_searched,
            )?;
        } else {
            // Check if we need to get the package info from the slow path.
            let changed = merge_dependency_to_dependencies(
                ctx.merged_dependencies,
                dependency,
            )?;
            if !changed {
                // There is no new information, so we won't proceed further.
                continue;
            }

            process_non_local_dependency_to_get_candidate(
                ctx.tman_config.clone(),
                ctx.support,
                dependency,
                ctx.all_compatible_installed_pkgs,
                ctx.all_candidates,
                ctx.new_pkgs_to_be_searched,
                ctx.pkg_list_cache,
                out.clone(),
            )
            .await?;
        }
    }

    Ok(())
}

/// For a version of a package, only keep the most suitable one. The most
/// suitable one is the package with the highest compatible_score. If there are
/// multiple packages with the highest score, just pick one at random.
fn clean_up_all_candidates(
    all_candidates: &mut HashMap<
        PkgTypeAndName,
        HashMap<PkgBasicInfo, PkgInfo>,
    >,
    locked_pkgs: Option<&HashMap<PkgTypeAndName, PkgInfo>>,
) {
    for (pkg_type_name, pkg_infos) in all_candidates.iter_mut() {
        let mut version_map: HashMap<Version, &PkgInfo> = HashMap::new();
        let mut locked_pkgs_map: HashMap<Version, &PkgInfo> = HashMap::new();

        let locked_pkg =
            locked_pkgs.and_then(|locked_pkgs| locked_pkgs.get(pkg_type_name));

        for pkg_info in pkg_infos.iter() {
            // Check if the candidate is a locked one.
            if let Some(locked_pkg) = locked_pkg {
                if locked_pkg.basic_info.version
                    == pkg_info.1.basic_info.version
                    && locked_pkg.hash == pkg_info.1.hash
                {
                    locked_pkgs_map.insert(
                        pkg_info.1.basic_info.version.clone(),
                        pkg_info.1,
                    );
                }
            }

            version_map
                .entry(pkg_info.1.basic_info.version.clone())
                .and_modify(|existing_pkg_info| {
                    if pkg_info.1.compatible_score
                        > existing_pkg_info.compatible_score
                    {
                        *existing_pkg_info = pkg_info.1;
                    }
                })
                .or_insert(pkg_info.1);
        }

        // If there is a locked one, replace the candidate with the highest
        // score with the locked one.
        version_map.extend(locked_pkgs_map);

        *pkg_infos = version_map
            .into_values()
            .map(|pkg_info| (pkg_info.into(), pkg_info.clone()))
            .collect();
    }
}

#[allow(clippy::too_many_arguments)]
pub async fn get_all_candidates_from_deps(
    tman_config: Arc<TmanConfig>,
    support: &PkgSupport,
    mut pkgs_to_be_searched: Vec<PkgInfo>,
    extra_dep: Option<&PkgDependency>,
    all_compatible_installed_pkgs: &HashMap<
        PkgTypeAndName,
        HashMap<PkgBasicInfo, PkgInfo>,
    >,
    mut all_candidates: HashMap<PkgTypeAndName, HashMap<PkgBasicInfo, PkgInfo>>,
    locked_pkgs: Option<&HashMap<PkgTypeAndName, PkgInfo>>,
    out: Arc<Box<dyn TmanOutput>>,
) -> Result<HashMap<PkgTypeAndName, HashMap<PkgBasicInfo, PkgInfo>>> {
    let mut merged_dependencies = HashMap::new();
    let mut processed_pkgs = HashSet::<PkgBasicInfo>::new();

    // Cache used to record combinations processed by `get_package_list`.
    let mut pkg_list_cache = PackageListCache::default();

    let mut context = DependenciesContext {
        tman_config,
        support,
        merged_dependencies: &mut merged_dependencies,
        all_compatible_installed_pkgs,
        all_candidates: &mut all_candidates,
        new_pkgs_to_be_searched: &mut Vec::new(),
        pkg_list_cache: &mut pkg_list_cache,
    };

    // If there is extra dependencies (ex: specified in the command line),
    // handle those dependencies, too.
    let mut extra_dep_opt = extra_dep.cloned();

    loop {
        // Merge the dependencies of all pending packages into a single
        // `Vector`.
        let mut combined_dependencies: Vec<PkgDependency> = Vec::new();

        for pkg_to_be_search in &pkgs_to_be_searched {
            if processed_pkgs.contains(&(pkg_to_be_search).into()) {
                continue;
            }

            // Add all dependencies of the current package to the merged list.
            combined_dependencies.extend(pkg_to_be_search.dependencies.clone());

            // Mark the package as processed.
            processed_pkgs.insert(pkg_to_be_search.into());
        }

        // If `extra_dep_opt` exists, add it to `combined_dependencies` (process
        // only once, i.e., the `take()` API).
        if let Some(extra) = extra_dep_opt.take() {
            combined_dependencies.push(extra);
        }

        // Sort the merged `dependencies` by the coverage range of
        // `version_req`, with broader scopes ranked first.
        combined_dependencies.sort_by(|a, b| {
            if is_superset_of(&a.version_req, &b.version_req)
                && !is_superset_of(&b.version_req, &a.version_req)
            {
                std::cmp::Ordering::Less
            } else if is_superset_of(&b.version_req, &a.version_req)
                && !is_superset_of(&a.version_req, &b.version_req)
            {
                std::cmp::Ordering::Greater
            } else {
                a.version_req.to_string().cmp(&b.version_req.to_string())
            }
        });

        // Process all merged dependencies in a single batch.
        process_dependencies_to_get_candidates(
            &mut context,
            &combined_dependencies,
            out.clone(),
        )
        .await?;

        pkgs_to_be_searched.clear();

        if context.new_pkgs_to_be_searched.is_empty() {
            break;
        }
        pkgs_to_be_searched.append(context.new_pkgs_to_be_searched);
    }

    clean_up_all_candidates(&mut all_candidates, locked_pkgs);

    Ok(all_candidates)
}

pub fn get_pkg_info_from_candidates(
    pkg_type: &str,
    pkg_name: &str,
    version: &str,
    all_candidates: &HashMap<PkgTypeAndName, HashMap<PkgBasicInfo, PkgInfo>>,
) -> Result<PkgInfo> {
    let pkg_type_name = PkgTypeAndName {
        pkg_type: pkg_type.parse::<PkgType>()?,
        name: pkg_name.to_string(),
    };
    let version_parsed = Version::parse(version)?;
    let pkg_info = all_candidates
        .get(&pkg_type_name)
        .and_then(|set| {
            set.iter()
                .find(|pkg| pkg.1.basic_info.version == version_parsed)
        })
        .ok_or_else(|| {
            anyhow!(
                "PkgInfo not found for [{}]{}@{}",
                pkg_type,
                pkg_name,
                version
            )
        })?;
    Ok(pkg_info.1.clone())
}
