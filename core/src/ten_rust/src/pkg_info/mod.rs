//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
mod binding;
pub mod constants;
pub mod hash;
pub mod language;
pub mod manifest;
pub mod message;
pub mod pkg_basic_info;
pub mod pkg_type;
pub mod pkg_type_and_name;
pub mod predefined_graphs;
pub mod property;
pub mod supports;
mod utils;
pub mod value_type;

use std::{collections::HashMap, path::Path};

use crate::graph::Graph;
use anyhow::{anyhow, Result};

use crate::base_dir_pkg_info::BaseDirPkgInfo;
use crate::schema::store::SchemaStore;
use constants::{
    ADDON_LOADER_DIR, EXTENSION_DIR, MANIFEST_JSON_FILENAME, PROTOCOL_DIR,
    SYSTEM_DIR, TEN_PACKAGES_DIR,
};
use manifest::{
    dependency::ManifestDependency, parse_manifest_from_file,
    parse_manifest_in_folder, Manifest,
};
use pkg_type::PkgType;
use property::{
    parse_property_in_folder, predefined_graph::PredefinedGraph, Property,
};

pub fn localhost() -> &'static str {
    "localhost"
}

#[derive(Clone, Debug)]
pub struct PkgInfo {
    pub manifest: Option<Manifest>,
    pub property: Option<Property>,

    pub compatible_score: i32,

    /// This indicates that the package has been installed in the correct
    /// location. For example, in the case of an extension, it means it has
    /// been installed under the `ten_packages/` directory.
    pub is_installed: bool,

    /// A string that represents the location or path where the package is
    /// installed or can be accessed.
    ///
    /// - If `PkgInfo represents a package in the registry, then the `url`
    ///   field represents the download URL that can be used to download the
    ///   package from the registry.
    /// - If PkgInfo represents a package that already exists locally, then the
    ///   url field represents the base directory of the package.
    pub url: String,

    pub hash: String,

    pub schema_store: Option<SchemaStore>,

    /// Indicates that the `pkg_info` represents a local dependency package.
    pub is_local_dependency: bool,
    pub local_dependency_path: Option<String>,
    pub local_dependency_base_dir: Option<String>,
}

impl PkgInfo {
    pub fn from_metadata(
        manifest: &Manifest,
        property: &Option<Property>,
    ) -> Result<Self> {
        let mut pkg_info = PkgInfo {
            compatible_score: -1,

            is_installed: false,
            url: String::new(),
            hash: String::new(),

            manifest: Some(manifest.clone()),
            property: property.clone(),
            schema_store: SchemaStore::from_manifest(manifest)?,

            is_local_dependency: false,
            local_dependency_path: None,
            local_dependency_base_dir: None,
        };

        pkg_info.hash = pkg_info.gen_hash_hex();

        Ok(pkg_info)
    }

    pub fn get_predefined_graphs(&self) -> Option<&Vec<PredefinedGraph>> {
        if let Some(property) = &self.property {
            if let Some(ten) = &property._ten {
                return ten.predefined_graphs.as_ref();
            }
        }

        None
    }

    pub fn update_predefined_graph(&mut self, new_graph: &PredefinedGraph) {
        if let Some(property) = &mut self.property {
            if let Some(ten) = &mut property._ten {
                if let Some(predefined_graphs) = &mut ten.predefined_graphs {
                    if let Some(old_graph) = predefined_graphs
                        .iter_mut()
                        .find(|g| g.name == new_graph.name)
                    {
                        *old_graph = new_graph.clone();
                    }
                }
            }
        }
    }

    pub fn get_dependency_by_type_and_name(
        &self,
        pkg_type: &str,
        pkg_name: &str,
    ) -> Option<&ManifestDependency> {
        if let Some(manifest) = &self.manifest {
            if let Some(dependencies) = &manifest.dependencies {
                return dependencies.iter().find(|dep| {
                    match dep {
                        ManifestDependency::RegistryDependency {
                            pkg_type: dep_type,
                            name,
                            ..
                        } => {
                            dep_type.to_string() == pkg_type && name == pkg_name
                        }
                        ManifestDependency::LocalDependency { .. } => {
                            // For local dependencies, we would need to resolve
                            // the actual type and name by examining the
                            // manifest at the local path, which is beyond the
                            // scope of this method, so we return false.
                            false
                        }
                    }
                });
            }
        }
        None
    }
}

/// Retrieves package information from the specified path.
///
/// This function reads and parses the manifest.json and property.json files
/// from the given path, then constructs a PkgInfo object with the parsed data.
///
/// # Arguments
/// * `path` - The base directory of the package containing manifest.json and
///   optionally property.json.
/// * `is_installed` - Boolean flag indicating whether the package is already
///   installed in the system.
///
/// # Returns
/// * `Result<PkgInfo>` - The constructed package information on success, or an
///   error if the manifest couldn't be parsed or the PkgInfo couldn't be
///   created.
///
/// # Errors
/// * Returns an error if manifest.json is missing or invalid.
/// * Returns an error if property.json exists but is invalid.
/// * Returns an error if PkgInfo creation fails.
pub fn get_pkg_info_from_path(
    path: &Path,
    is_installed: bool,
) -> Result<PkgInfo> {
    let manifest = parse_manifest_in_folder(path)?;
    let property = parse_property_in_folder(path)?;

    let mut pkg_info: PkgInfo = PkgInfo::from_metadata(&manifest, &property)?;

    pkg_info.is_installed = is_installed;

    // This package comes from a path, not from a registry, so the value of
    // `url` will be the path.
    pkg_info.url = path.to_string_lossy().to_string();

    Ok(pkg_info)
}

/// Collect the corresponding package from the information within the specified
/// path, add it to the collection provided as a parameter, and return the newly
/// collected package.
fn collect_pkg_info_from_path<'a>(
    path: &Path,
    pkgs_info: &'a mut BaseDirPkgInfo,
) -> Result<&'a PkgInfo> {
    let pkg_info = get_pkg_info_from_path(path, true)?;

    if let Some(manifest) = &pkg_info.manifest {
        match manifest.type_and_name.pkg_type {
            PkgType::App => {
                pkgs_info.app_pkg_info = pkg_info;
                Ok(&pkgs_info.app_pkg_info)
            }
            PkgType::Extension => {
                pkgs_info.extension_pkg_info.push(pkg_info);
                Ok(pkgs_info.extension_pkg_info.last().unwrap())
            }
            PkgType::Protocol => {
                pkgs_info.protocol_pkg_info.push(pkg_info);
                Ok(pkgs_info.protocol_pkg_info.last().unwrap())
            }
            PkgType::AddonLoader => {
                pkgs_info.addon_loader_pkg_info.push(pkg_info);
                Ok(pkgs_info.addon_loader_pkg_info.last().unwrap())
            }
            PkgType::System => {
                pkgs_info.system_pkg_info.push(pkg_info);
                Ok(pkgs_info.system_pkg_info.last().unwrap())
            }
            _ => Err(anyhow!("Unknown package type")),
        }
    } else {
        Err(anyhow!("Package missing manifest"))
    }
}

/// Retrieves information about all installed packages related to a specific
/// application and stores this information in a BaseDirPkgInfo struct.
pub fn get_app_installed_pkgs(app_path: &Path) -> Result<BaseDirPkgInfo> {
    let mut pkgs_info = BaseDirPkgInfo {
        app_pkg_info: PkgInfo {
            manifest: None,
            property: None,
            compatible_score: 0,
            is_installed: false,
            url: String::new(),
            hash: String::new(),
            schema_store: None,
            is_local_dependency: false,
            local_dependency_path: None,
            local_dependency_base_dir: None,
        },
        extension_pkg_info: Vec::new(),
        protocol_pkg_info: Vec::new(),
        addon_loader_pkg_info: Vec::new(),
        system_pkg_info: Vec::new(),
    };

    // Process the manifest.json file in the root path.
    let app_pkg = collect_pkg_info_from_path(app_path, &mut pkgs_info)?;
    if let Some(manifest) = &app_pkg.manifest {
        if manifest.type_and_name.pkg_type != PkgType::App {
            return Err(anyhow!(
                "The current working directory does not belong to the `app`."
            ));
        }
    } else {
        return Err(anyhow!("App package missing manifest"));
    }

    // Define the sub-folders for searching packages.
    let addon_type_dirs =
        vec![EXTENSION_DIR, PROTOCOL_DIR, ADDON_LOADER_DIR, SYSTEM_DIR];

    for addon_type_dir in addon_type_dirs {
        let addon_type_dir_path =
            app_path.join(TEN_PACKAGES_DIR).join(addon_type_dir);

        if addon_type_dir_path.exists() && addon_type_dir_path.is_dir() {
            for entry in addon_type_dir_path.read_dir()?.flatten() {
                let path = entry.path();

                if path.is_dir() {
                    // An essential component of a TEN package folder is its
                    // `manifest.json` file that adheres to the TEN standard.
                    let manifest_path = path.join(MANIFEST_JSON_FILENAME);

                    if manifest_path.exists() && manifest_path.is_file() {
                        let manifest =
                            parse_manifest_from_file(&manifest_path)?;

                        manifest.check_fs_location(
                            Some(addon_type_dir),
                            path.file_name().and_then(|os_str| os_str.to_str()),
                        )?;

                        // This folder contains a manifest.json file and
                        // that manifest.json file is a correct TEN
                        // manifest.json file, so that it means the folder
                        // represents a valid TEN package, and this package is a
                        // _local_ dependency of app.

                        collect_pkg_info_from_path(&path, &mut pkgs_info)?;
                    }
                }
            }
        }
    }

    Ok(pkgs_info)
}

/// Identifies local packages that are not tracked in the dependencies list. It
/// iterates over the local packages and checks if each package is present in
/// the dependencies list. If a package is not found in the dependencies list,
/// it is added to the list of untracked local packages.
///
/// # Arguments
/// * `dependencies` - A list of dependencies, required.
/// * `local_pkgs` - A list of local packages, required.
///
/// # Returns
/// A list of untracked local packages.
pub fn find_untracked_local_packages<'a>(
    dependencies: &[&'a PkgInfo],
    local_pkgs: &[&'a PkgInfo],
) -> Vec<&'a PkgInfo> {
    let mut untracked_pkgs = Vec::new();

    for pkg in local_pkgs {
        let Some(pkg_manifest) = &pkg.manifest else {
            continue;
        };

        // Check all dependencies to see if this local package is tracked.
        let is_tracked = dependencies.iter().any(|dep| {
            let Some(dep_manifest) = &dep.manifest else {
                return false;
            };

            // Compare type, name, and version
            pkg_manifest.type_and_name.pkg_type
                == dep_manifest.type_and_name.pkg_type
                && pkg_manifest.type_and_name.name
                    == dep_manifest.type_and_name.name
                && pkg_manifest.version == dep_manifest.version
        });

        if !is_tracked {
            untracked_pkgs.push(*pkg);
        }
    }

    untracked_pkgs
}

/// Return a list of tuples, each tuple contains a local installed package
/// and the package with the same identity but not installed locally in the
/// dependencies list.
///
/// The possible reason could be that the 'supports' is incompatible, or the
/// version calculated by the dependency tree is different.
pub fn find_to_be_replaced_local_pkgs<'a>(
    dependencies: &[&'a PkgInfo],
    local_pkgs: &[&'a PkgInfo],
) -> Vec<(&'a PkgInfo, &'a PkgInfo)> {
    let mut to_be_replaced = Vec::new();

    for dep in dependencies {
        let Some(dep_manifest) = &dep.manifest else {
            continue;
        };

        // For each dependency, look for a local package with the same type and
        // name.
        for pkg in local_pkgs {
            let Some(pkg_manifest) = &pkg.manifest else {
                continue;
            };

            // If type and name match but versions differ, this package will be
            // replaced.
            if dep_manifest.type_and_name.pkg_type
                == pkg_manifest.type_and_name.pkg_type
                && dep_manifest.type_and_name.name
                    == pkg_manifest.type_and_name.name
                && dep_manifest.version != pkg_manifest.version
            {
                to_be_replaced.push((*dep, *pkg));
            }
        }
    }

    to_be_replaced
}

/// Check the graph for current app.
///
/// # Arguments
/// * `app_base_dir` - The absolute path of the app base directory, required.
/// * `graph_json` - The graph definition in JSON format, required.
/// * `app_uri` - The `_ten::uri` of the app, required. We do not read this
///   content from property.json of app, because the property.json might not
///   exist. Ex: the property of app might be customized using
///   `init_property_from_json` in `on_configure` method.
pub fn ten_rust_check_graph_for_app(
    app_base_dir: &str,
    graph_json: &str,
    app_uri: &str,
) -> Result<()> {
    // Get all installed packages.
    let app_path = Path::new(app_base_dir);
    let pkgs_info = get_app_installed_pkgs(app_path)?;

    // Create a map of all installed packages across all apps.
    let mut installed_pkgs_of_all_apps: HashMap<String, Vec<PkgInfo>> =
        HashMap::new();

    // Insert packages for this app - collect all types into a single vector.
    let mut all_pkgs = Vec::new();
    all_pkgs.push(pkgs_info.app_pkg_info.clone());
    all_pkgs.extend(pkgs_info.extension_pkg_info.clone());
    all_pkgs.extend(pkgs_info.protocol_pkg_info.clone());
    all_pkgs.extend(pkgs_info.addon_loader_pkg_info.clone());
    all_pkgs.extend(pkgs_info.system_pkg_info.clone());

    installed_pkgs_of_all_apps.insert(app_uri.to_string(), all_pkgs);

    // Parse the graph JSON.
    // let graph = Graph::from_str(graph_json)?;
    let graph: Graph = serde_json::from_str(graph_json)?;

    graph.check_for_single_app(&installed_pkgs_of_all_apps)
}
