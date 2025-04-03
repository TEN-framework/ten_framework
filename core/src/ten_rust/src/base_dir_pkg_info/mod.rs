//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use crate::pkg_info::PkgInfo;

pub struct BaseDirPkgInfo {
    pub app_pkg_info: PkgInfo,
    pub extension_pkg_info: Vec<PkgInfo>,
    pub protocol_pkg_info: Vec<PkgInfo>,
    pub addon_loader_pkg_info: Vec<PkgInfo>,
    pub system_pkg_info: Vec<PkgInfo>,
}
