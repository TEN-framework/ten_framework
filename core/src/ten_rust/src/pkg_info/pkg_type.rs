//
// Copyright © 2024 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::{fmt, str::FromStr};

use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};

use super::constants::{EXTENSION_DIR, GENERIC_DIR, SYSTEM_DIR};

#[derive(
    Copy,
    Clone,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
)]
pub enum PkgType {
    #[serde(rename = "system")]
    System,

    #[serde(rename = "app")]
    App,

    #[serde(rename = "extension")]
    Extension,

    #[serde(rename = "protocol")]
    Protocol,

    #[serde(rename = "lang_addon_loader")]
    LangAddonLoader,
}

impl FromStr for PkgType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "system" => Ok(PkgType::System),
            "app" => Ok(PkgType::App),
            "extension" => Ok(PkgType::Extension),
            "protocol" => Ok(PkgType::Protocol),
            "lang_addon_loader" => Ok(PkgType::LangAddonLoader),
            _ => Err(Error::msg("Failed to parse string to package type")),
        }
    }
}

impl fmt::Display for PkgType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PkgType::System => write!(f, "system"),
            PkgType::App => write!(f, "app"),
            PkgType::Extension => write!(f, "extension"),
            PkgType::Protocol => write!(f, "protocol"),
            PkgType::LangAddonLoader => write!(f, "lang_addon_loader"),
        }
    }
}

impl PkgType {
    // Returns the corresponding pkg namespace folder for each PkgType variant.
    pub fn namespace(&self) -> &'static str {
        match self {
            PkgType::System => SYSTEM_DIR,
            PkgType::Extension => EXTENSION_DIR,
            PkgType::Protocol | PkgType::LangAddonLoader => GENERIC_DIR,
            // App does not have a specific namespace folder.
            PkgType::App => "",
        }
    }
}
