//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use crate::pkg_info::PkgInfo;

#[derive(Debug, Clone)]
pub struct BaseDirPkgInfo {
    pub app_pkg_info: Option<PkgInfo>,
    pub extension_pkg_info: Option<Vec<PkgInfo>>,
    pub protocol_pkg_info: Option<Vec<PkgInfo>>,
    pub addon_loader_pkg_info: Option<Vec<PkgInfo>>,
    pub system_pkg_info: Option<Vec<PkgInfo>>,
}

impl BaseDirPkgInfo {
    // Convert BaseDirPkgInfo to Vec<PkgInfo>.
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
}
