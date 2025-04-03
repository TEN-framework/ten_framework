//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use crate::pkg_info::PkgInfo;

#[derive(Debug, Clone)]
pub struct BaseDirPkgInfo {
    pub app_pkg_info: PkgInfo,
    pub extension_pkg_info: Vec<PkgInfo>,
    pub protocol_pkg_info: Vec<PkgInfo>,
    pub addon_loader_pkg_info: Vec<PkgInfo>,
    pub system_pkg_info: Vec<PkgInfo>,
}

impl BaseDirPkgInfo {
    // Convert BaseDirPkgInfo to Vec<PkgInfo>.
    pub fn to_vec(&self) -> Vec<PkgInfo> {
        let mut all_pkgs = Vec::new();
        all_pkgs.push(self.app_pkg_info.clone());
        all_pkgs.extend(self.extension_pkg_info.clone());
        all_pkgs.extend(self.protocol_pkg_info.clone());
        all_pkgs.extend(self.addon_loader_pkg_info.clone());
        all_pkgs.extend(self.system_pkg_info.clone());
        all_pkgs
    }

    // Check if any of the vectors is not empty.
    pub fn is_empty(&self) -> bool {
        self.app_pkg_info.manifest.is_none()
            && self.extension_pkg_info.is_empty()
            && self.protocol_pkg_info.is_empty()
            && self.addon_loader_pkg_info.is_empty()
            && self.system_pkg_info.is_empty()
    }
}
