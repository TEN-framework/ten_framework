//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::hash::Hash;
use std::{fmt, str::FromStr};

use anyhow::{Context, Error, Result};
use serde::{Deserialize, Serialize};

use crate::env::get_env;
use crate::pkg_info::manifest::support::ManifestSupport;

#[derive(
    Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize,
)]
pub struct PkgSupport {
    // Unspecified fields represent 'don't care', so we need to use `Option`
    // to express that they are not specified.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub os: Option<Os>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub arch: Option<Arch>,
}

impl fmt::Display for PkgSupport {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let os_str = match &self.os {
            Some(os) => os.to_string(),
            None => "".to_string(),
        };

        let arch_str = match &self.arch {
            Some(arch) => arch.to_string(),
            None => "".to_string(),
        };

        write!(f, "{}, {}", os_str, arch_str)
    }
}

pub struct SupportsDisplay<'a>(pub &'a [PkgSupport]);

impl fmt::Display for SupportsDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.is_empty() {
            write!(f, "all environments")
        } else {
            for (i, support) in self.0.iter().enumerate() {
                if i > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "({})", support)?;
            }
            Ok(())
        }
    }
}

impl TryFrom<&ManifestSupport> for PkgSupport {
    type Error = anyhow::Error;

    fn try_from(manifest_support: &ManifestSupport) -> Result<Self> {
        Ok(PkgSupport {
            os: manifest_support
                .os
                .as_deref()
                .map(Os::from_str)
                .transpose()
                .context("Failed to parse OS from manifest")?,
            arch: manifest_support
                .arch
                .as_deref()
                .map(Arch::from_str)
                .transpose()
                .context("Failed to parse Arch from manifest")?,
        })
    }
}

impl PkgSupport {
    fn is_support_compatible(&self, pivot: &PkgSupport) -> i32 {
        let mut compatible_score = 0;

        if let Some(os) = &pivot.os {
            if let Some(support_os) = &self.os {
                if os != support_os {
                    // If a specific OS support is required, and the current
                    // support combination does not list the desired OS, then it
                    // is not compatible.
                    return -1;
                }
                compatible_score += 1;
            }
        }

        if let Some(arch) = &pivot.arch {
            if let Some(support_arch) = &self.arch {
                if arch != support_arch {
                    // Same as above.
                    return -1;
                }
                compatible_score += 1;
            }
        }

        compatible_score
    }

    pub fn set_defaults(&mut self) -> Result<()> {
        let current_env = get_env()?;

        if self.os.is_none() {
            self.os = Some(current_env.os);
        }
        if self.arch.is_none() {
            self.arch = Some(current_env.arch);
        }

        Ok(())
    }
}

pub fn get_pkg_supports_from_manifest_supports(
    manifest_supports: &Option<Vec<ManifestSupport>>,
) -> Result<Vec<PkgSupport>> {
    if let Some(supports) = &manifest_supports {
        Ok(supports
            .iter()
            .map(PkgSupport::try_from)
            .collect::<Result<Vec<_>>>()?)
    } else {
        Ok(vec![])
    }
}

pub fn get_manifest_supports_from_pkg(
    support: &[PkgSupport],
) -> Vec<ManifestSupport> {
    support.iter().map(|v| v.into()).collect()
}

pub fn is_pkg_supports_compatible_with(
    pkg_supports: &Vec<PkgSupport>,
    pivot: &PkgSupport,
) -> i32 {
    if pkg_supports.is_empty() {
        // If no support combinations are listed, then it is compatible with
        // everything.
        return 0;
    }

    let mut largest_compatible_score = -1;

    // Loop through all listed support combinations to see if one of them
    // matches the specified support.
    for support in pkg_supports {
        let new_score = support.is_support_compatible(pivot);
        if new_score > largest_compatible_score {
            largest_compatible_score = new_score;
        }
    }

    largest_compatible_score
}

#[derive(
    Clone, Debug, Serialize, Hash, Deserialize, PartialEq, Eq, PartialOrd, Ord,
)]
pub enum Os {
    #[serde(rename = "win")]
    Win,

    #[serde(rename = "mac")]
    Mac,

    #[serde(rename = "linux")]
    Linux,
}

impl FromStr for Os {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "win" => Ok(Os::Win),
            "windows" => Ok(Os::Win),
            "mac" => Ok(Os::Mac),
            "macos" => Ok(Os::Mac),
            "linux" => Ok(Os::Linux),
            _ => Err(Error::msg("Failed to parse string to OS")),
        }
    }
}

impl fmt::Display for Os {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Os::Win => write!(f, "win"),
            Os::Mac => write!(f, "mac"),
            Os::Linux => write!(f, "linux"),
        }
    }
}

#[derive(
    Clone, Debug, Serialize, Hash, Deserialize, PartialEq, Eq, PartialOrd, Ord,
)]
pub enum Arch {
    #[serde(rename = "x86")]
    X86,

    #[serde(rename = "x64")]
    X64,

    #[serde(rename = "arm")]
    Arm,

    #[serde(rename = "arm64")]
    Arm64,
}

impl FromStr for Arch {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "x86" => Ok(Arch::X86),
            "x64" => Ok(Arch::X64),
            "x86_64" => Ok(Arch::X64),
            "arm" => Ok(Arch::Arm),
            "arm64" => Ok(Arch::Arm64),
            "aarch64" => Ok(Arch::Arm64),
            _ => Err(Error::msg("Failed to parse string to CPU")),
        }
    }
}

impl fmt::Display for Arch {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Arch::X86 => write!(f, "x86"),
            Arch::X64 => write!(f, "x64"),
            Arch::Arm => write!(f, "arm"),
            Arch::Arm64 => write!(f, "arm64"),
        }
    }
}
