//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use serde::{Deserialize, Serialize};

use crate::pkg_info::supports::{Arch, Os, PkgSupport};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ManifestSupport {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub os: Option<Os>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub arch: Option<Arch>,
}

impl From<&PkgSupport> for ManifestSupport {
    fn from(support: &PkgSupport) -> Self {
        ManifestSupport {
            os: support.os.clone(),
            arch: support.arch.clone(),
        }
    }
}
