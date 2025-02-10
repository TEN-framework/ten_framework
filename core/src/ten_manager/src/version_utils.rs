//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use anyhow::{anyhow, Result};
use semver::VersionReq;

/// Used to parse a pattern like `aaa@3.0.0` and return `aaa` and `3.0.0`.
pub fn parse_pkg_name_version_req(
    pkg_name_version: &str,
) -> Result<(String, VersionReq)> {
    let parts: Vec<&str> = pkg_name_version.split('@').collect();
    if parts.len() == 2 {
        // Currently, tman uses the Rust semver crate, while the cloud store
        // uses the npm semver package. The semver requirement specifications of
        // these two packages are not completely identical. For example:
        //
        // - The Rust semver crate uses "," to separate different ranges,
        //   whereas the npm semver package uses a space (" ") to separate
        //   different requirement ranges.
        // - The npm semver package uses "||" to unify different ranges, but the
        //   Rust semver crate does not support this feature.
        //
        // Since TEN is a cross-language system, it needs to define its own
        // semver requirement specification. This specification could follow
        // either the Rust or npm format or other spec, but in either case, tman
        // or the cloud store would need to make adaptations.
        //
        // Therefore, the current approach is to simplify the specification to
        // only support a single-range semver requirement, which is the common
        // subset of both the npm semver package and the Rust semver crate.
        if parts[1].contains("||")
            || parts[1].chars().any(|c| c.is_whitespace())
            || parts[1].contains(",")
        {
            return Err(anyhow!(
              "Invalid version requirement '{}' in package name version string: contains forbidden characters (||, whitespace, or ,)",
          parts[1]
            ));
        }

        Ok((parts[0].to_string(), VersionReq::parse(parts[1])?))
    } else {
        Ok((pkg_name_version.to_string(), VersionReq::STAR))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use semver::{Version, VersionReq};

    #[test]
    fn test_parse_version_rc() -> Result<()> {
        let input = "aaa@3.0.0-rc1";
        let (pkg, ver_req) = parse_pkg_name_version_req(input)?;
        assert_eq!(pkg, "aaa");

        let expected = VersionReq::parse("3.0.0-rc1")?;
        assert_eq!(ver_req, expected);

        Ok(())
    }

    #[test]
    fn test_pre_release_version_matches_1() {
        let version_str = "0.8.0-rc.1";
        let req_str = "^0.8.0-rc.1";

        let req = VersionReq::parse(req_str)
            .expect("Failed to parse version requirement");
        let version =
            Version::parse(version_str).expect("Failed to parse version");

        assert!(
            req.matches(&version),
            "Version {:?} does not match requirement {}",
            version,
            req_str
        );
    }

    #[test]
    fn test_pre_release_version_matches_2() {
        let version_str = "0.8.0-rc.1";
        let req_str = "=0.8.0-rc.1";

        let req = VersionReq::parse(req_str)
            .expect("Failed to parse version requirement");
        let version =
            Version::parse(version_str).expect("Failed to parse version");

        assert!(
            req.matches(&version),
            "Version {:?} does not match requirement {}",
            version,
            req_str
        );
    }
}
