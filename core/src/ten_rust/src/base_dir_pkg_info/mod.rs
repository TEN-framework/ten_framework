//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use crate::pkg_info::{pkg_type::PkgType, PkgInfo};

#[derive(Debug, Clone, Default)]
pub struct PkgsInfoInApp {
    pub app_pkg_info: Option<PkgInfo>,
    pub extension_pkg_info: Option<Vec<PkgInfo>>,
    pub protocol_pkg_info: Option<Vec<PkgInfo>>,
    pub addon_loader_pkg_info: Option<Vec<PkgInfo>>,
    pub system_pkg_info: Option<Vec<PkgInfo>>,
}

/// A struct that contains both PkgsInfoInApp and its base_dir.
#[derive(Clone)]
pub struct PkgsInfoInAppWithBaseDir {
    pub pkgs_info_in_app: PkgsInfoInApp,
    pub base_dir: String,
}

impl PkgsInfoInApp {
    // Get a reference to the extension packages or an empty slice if none.
    pub fn get_extensions(&self) -> &[PkgInfo] {
        if let Some(extensions) = &self.extension_pkg_info {
            extensions.as_slice()
        } else {
            &[]
        }
    }

    /// Combines all package types into a single vector.
    pub fn to_vec(&self) -> Vec<PkgInfo> {
        let mut all_pkgs = Vec::new();
        if let Some(app_info) = &self.app_pkg_info {
            all_pkgs.push(app_info.clone());
        }
        if let Some(ext_info) = &self.extension_pkg_info {
            all_pkgs.extend(ext_info.clone());
        }
        if let Some(proto_info) = &self.protocol_pkg_info {
            all_pkgs.extend(proto_info.clone());
        }
        if let Some(addon_info) = &self.addon_loader_pkg_info {
            all_pkgs.extend(addon_info.clone());
        }
        if let Some(sys_info) = &self.system_pkg_info {
            all_pkgs.extend(sys_info.clone());
        }
        all_pkgs
    }

    // Check if all fields are None or empty vectors.
    pub fn is_empty(&self) -> bool {
        self.app_pkg_info.is_none()
            && (self.extension_pkg_info.is_none()
                || self.extension_pkg_info.as_ref().unwrap().is_empty())
            && (self.protocol_pkg_info.is_none()
                || self.protocol_pkg_info.as_ref().unwrap().is_empty())
            && (self.addon_loader_pkg_info.is_none()
                || self.addon_loader_pkg_info.as_ref().unwrap().is_empty())
            && (self.system_pkg_info.is_none()
                || self.system_pkg_info.as_ref().unwrap().is_empty())
    }

    /// Find a package by its type and name.
    /// Returns None if no matching package is found.
    pub fn find_pkg_by_type_and_name(
        &self,
        pkg_type: PkgType,
        name: &str,
    ) -> Option<&PkgInfo> {
        match pkg_type {
            PkgType::App => {
                if let Some(app_pkg) = &self.app_pkg_info {
                    if let Some(manifest) = &app_pkg.manifest {
                        if manifest.type_and_name.pkg_type == pkg_type
                            && manifest.type_and_name.name == name
                        {
                            return Some(app_pkg);
                        }
                    }
                }
                None
            }
            PkgType::Extension => {
                if let Some(extensions) = &self.extension_pkg_info {
                    extensions.iter().find(|pkg| {
                        if let Some(manifest) = &pkg.manifest {
                            manifest.type_and_name.pkg_type == pkg_type
                                && manifest.type_and_name.name == name
                        } else {
                            false
                        }
                    })
                } else {
                    None
                }
            }
            PkgType::Protocol => {
                if let Some(protocols) = &self.protocol_pkg_info {
                    protocols.iter().find(|pkg| {
                        if let Some(manifest) = &pkg.manifest {
                            manifest.type_and_name.pkg_type == pkg_type
                                && manifest.type_and_name.name == name
                        } else {
                            false
                        }
                    })
                } else {
                    None
                }
            }
            PkgType::AddonLoader => {
                if let Some(addon_loaders) = &self.addon_loader_pkg_info {
                    addon_loaders.iter().find(|pkg| {
                        if let Some(manifest) = &pkg.manifest {
                            manifest.type_and_name.pkg_type == pkg_type
                                && manifest.type_and_name.name == name
                        } else {
                            false
                        }
                    })
                } else {
                    None
                }
            }
            PkgType::System => {
                if let Some(systems) = &self.system_pkg_info {
                    systems.iter().find(|pkg| {
                        if let Some(manifest) = &pkg.manifest {
                            manifest.type_and_name.pkg_type == pkg_type
                                && manifest.type_and_name.name == name
                        } else {
                            false
                        }
                    })
                } else {
                    None
                }
            }
            PkgType::Invalid => None,
        }
    }

    /// Returns the total number of packages in this PkgsInfoInApp.
    pub fn len(&self) -> usize {
        let mut count = 0;
        if self.app_pkg_info.is_some() {
            count += 1;
        }
        if let Some(extensions) = &self.extension_pkg_info {
            count += extensions.len();
        }
        if let Some(protocols) = &self.protocol_pkg_info {
            count += protocols.len();
        }
        if let Some(addon_loaders) = &self.addon_loader_pkg_info {
            count += addon_loaders.len();
        }
        if let Some(systems) = &self.system_pkg_info {
            count += systems.len();
        }
        count
    }
}
