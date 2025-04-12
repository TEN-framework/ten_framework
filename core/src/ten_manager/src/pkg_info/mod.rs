//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
pub mod get_all_pkgs;
pub mod manifest;

use std::{collections::HashMap, path::Path, sync::Arc};

use anyhow::{anyhow, Result};

use crate::{config::TmanConfig, output::TmanOutput};
use ten_rust::{
    base_dir_pkg_info::{PkgsInfoInApp, PkgsInfoInAppWithBaseDir},
    graph::graph_info::GraphInfo,
    pkg_info::{get_app_installed_pkgs, pkg_type::PkgType, PkgInfo},
};

/// Retrieves information about all installed packages for the specified
/// application.
pub fn tman_get_all_installed_pkgs_info_of_app(
    tman_config: Arc<TmanConfig>,
    app_path: &Path,
    out: Arc<Box<dyn TmanOutput>>,
) -> Result<Vec<PkgInfo>> {
    let pkg_info_struct = get_app_installed_pkgs(app_path, false, &mut None)?;

    // Use the to_vec method to combine all package types into a single vector.
    let all_pkgs = pkg_info_struct.to_vec();

    if tman_config.verbose {
        out.normal_line(&format!("{:?}", all_pkgs));
    }
    Ok(all_pkgs)
}

pub fn get_pkg_in_app<'a>(
    pkgs_info_in_app: &'a PkgsInfoInApp,
    pkg_type: &PkgType,
    pkg_name: &Option<&String>,
) -> Result<Option<&'a PkgInfo>> {
    match pkg_type {
        PkgType::App => Ok(pkgs_info_in_app.app_pkg_info.as_ref()),
        PkgType::Extension => {
            assert!(pkg_name.is_some());

            match &pkgs_info_in_app.extension_pkgs_info {
                Some(pkgs) => Ok(pkgs.iter().find(|pkg| {
                    pkg.manifest.as_ref().is_some_and(|m| {
                        m.type_and_name.name == *pkg_name.unwrap()
                    })
                })),
                None => Ok(None),
            }
        }
        _ => Err(anyhow!("Unsupported package type: {:?}", pkg_type)),
    }
}

/// Creates a mapping from app URIs to PkgsInfoInAppWithBaseDir.
/// This function is used to create a hash map that can be used for graph
/// connection operations.
pub fn create_uri_to_pkg_info_map(
    pkgs_cache: &HashMap<String, PkgsInfoInApp>,
) -> Result<HashMap<Option<String>, PkgsInfoInAppWithBaseDir>, String> {
    // Create a hash map from app URIs to PkgsInfoInApp
    let mut uri_to_pkg_info: HashMap<Option<String>, PkgsInfoInAppWithBaseDir> =
        HashMap::new();

    // Process all available apps to map URIs to PkgsInfoInApp
    for (base_dir, base_dir_pkg_info) in pkgs_cache.iter() {
        if let Some(app_pkg) = &base_dir_pkg_info.app_pkg_info {
            if let Some(property) = &app_pkg.property {
                if let Some(ten) = &property._ten {
                    // Map the URI to the PkgsInfoInApp, using None if URI is
                    // None
                    let key = ten.uri.clone();

                    // Check if the key already exists
                    if let Some(existing) = uri_to_pkg_info.get(&key) {
                        let error_message = if key.is_none() {
                            format!(
                                "Found two apps with unspecified URI in both '{}' and '{}'",
                                existing.base_dir,
                                base_dir
                            )
                        } else {
                            format!(
                                "Duplicate app uri '{}' found in both '{}' and '{}'",
                                key.as_ref().unwrap(),
                                existing.base_dir,
                                base_dir
                            )
                        };

                        return Err(error_message);
                    }

                    uri_to_pkg_info.insert(
                        key,
                        PkgsInfoInAppWithBaseDir {
                            pkgs_info_in_app: base_dir_pkg_info.clone(),
                            base_dir: base_dir.clone(),
                        },
                    );
                }
            }
        }
    }

    Ok(uri_to_pkg_info)
}

pub fn pkg_info_find_by_graph_info<'a>(
    pkgs_cache: &'a HashMap<String, PkgsInfoInApp>,
    graph_info: &GraphInfo,
) -> Result<Option<&'a PkgInfo>> {
    match &graph_info.app_base_dir {
        Some(app_base_dir) => {
            let pkgs_info_in_app = pkgs_cache.get(app_base_dir);
            assert!(pkgs_info_in_app.is_some());

            if let Some(pkgs_info_in_app) = pkgs_info_in_app {
                match graph_info.belonging_pkg_type {
                    Some(PkgType::App) => {
                        Ok(pkgs_info_in_app.app_pkg_info.as_ref())
                    }
                    Some(PkgType::Extension) => {
                        assert!(graph_info.belonging_pkg_name.is_some());
                        assert!(pkgs_info_in_app.extension_pkgs_info.is_some());

                        if let Some(pkg_name) = &graph_info.belonging_pkg_name {
                            if let Some(extension_pkgs_info) =
                                pkgs_info_in_app.extension_pkgs_info.as_ref()
                            {
                                Ok(extension_pkgs_info.iter().find(|pkg| {
                                    pkg.manifest.as_ref().is_some_and(|m| {
                                        m.type_and_name.name == *pkg_name
                                    })
                                }))
                            } else {
                                Err(anyhow!(
                                    "Package name not found: {:?}",
                                    graph_info.belonging_pkg_name
                                ))
                            }
                        } else {
                            Err(anyhow!(
                                "Package name not found: {:?}",
                                graph_info.belonging_pkg_name
                            ))
                        }
                    }
                    _ => Err(anyhow!(
                        "Unsupported package type: {:?}",
                        graph_info.belonging_pkg_type
                    )),
                }
            } else {
                Err(anyhow!("App base dir not found: {:?}", app_base_dir))
            }
        }
        None => Ok(None),
    }
}

pub fn pkg_info_find_by_graph_info_mut<'a>(
    pkgs_cache: &'a mut HashMap<String, PkgsInfoInApp>,
    graph_info: &GraphInfo,
) -> Result<Option<&'a mut PkgInfo>> {
    match &graph_info.app_base_dir {
        Some(app_base_dir) => {
            let app_base_dir = app_base_dir.clone();
            let pkgs_info_in_app = pkgs_cache.get_mut(&app_base_dir);
            assert!(pkgs_info_in_app.is_some());

            if let Some(pkgs_info_in_app) = pkgs_info_in_app {
                match graph_info.belonging_pkg_type {
                    Some(PkgType::App) => {
                        Ok(pkgs_info_in_app.app_pkg_info.as_mut())
                    }
                    Some(PkgType::Extension) => {
                        assert!(graph_info.belonging_pkg_name.is_some());
                        assert!(pkgs_info_in_app.extension_pkgs_info.is_some());

                        if let Some(pkg_name) = &graph_info.belonging_pkg_name {
                            if let Some(extension_pkgs_info) =
                                pkgs_info_in_app.extension_pkgs_info.as_mut()
                            {
                                Ok(extension_pkgs_info.iter_mut().find(|pkg| {
                                    pkg.manifest.as_ref().is_some_and(|m| {
                                        m.type_and_name.name == *pkg_name
                                    })
                                }))
                            } else {
                                Err(anyhow!(
                                    "Package name not found: {:?}",
                                    graph_info.belonging_pkg_name
                                ))
                            }
                        } else {
                            Err(anyhow!(
                                "Package name not found: {:?}",
                                graph_info.belonging_pkg_name
                            ))
                        }
                    }
                    _ => Err(anyhow!(
                        "Unsupported package type: {:?}",
                        graph_info.belonging_pkg_type
                    )),
                }
            } else {
                Err(anyhow!("App base dir not found: {:?}", app_base_dir))
            }
        }
        None => Ok(None),
    }
}
