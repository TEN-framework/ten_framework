//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::collections::HashMap;

use semver::{Version, VersionReq};
use ten_rust::pkg_info::pkg_type_and_name::PkgTypeAndName;

#[derive(Default)]
pub struct PackageListCache {
    // Key: PkgTypeAndName, Value: List of processed VersionReq.
    cache: HashMap<PkgTypeAndName, Vec<VersionReq>>,
}

impl PackageListCache {
    /// Checks and updates the cache.
    /// - If an existing cache entry is a superset of `new_req`, return `false`,
    ///   indicating no further processing is needed.
    /// - If `new_req` is a superset of some entries in the cache, remove those
    ///   entries and add `new_req`.
    /// - Otherwise, add `new_req` to the cache.
    pub fn check_and_update(
        &mut self,
        key: &PkgTypeAndName,
        new_req: &VersionReq,
    ) -> bool {
        let entry = self.cache.entry(key.clone()).or_default();

        // If an existing version requirement can cover `new_req`, then skip it.
        for cached_req in entry.iter() {
            if is_superset_of(cached_req, new_req) {
                // The current `new_req` is covered by a cached entry, skipping
                // processing it.
                return false;
            }
        }

        // If `new_req` is a superset of an entry in the cache, remove the
        // covered entries.
        entry.retain(|cached_req| !is_superset_of(new_req, cached_req));

        // Add `new_req` to the cache.
        entry.push(new_req.clone());

        true
    }
}

/// If the version requirement is a caret range (e.g., `^X.Y.Z`), return its
/// lower and upper bounds.
fn get_caret_bounds(req: &VersionReq) -> Option<(Version, Version)> {
    if req.comparators.len() != 1 {
        return None;
    }

    let req_str = req.to_string();
    if !req_str.starts_with('^') {
        return None;
    }

    let version_str = &req_str[1..];
    let lower_bound = Version::parse(version_str).ok()?;

    let upper_bound = if lower_bound.major > 0 {
        Version {
            major: lower_bound.major + 1,
            minor: 0,
            patch: 0,
            pre: Default::default(),
            build: Default::default(),
        }
    } else if lower_bound.minor > 0 {
        Version {
            major: 0,
            minor: lower_bound.minor + 1,
            patch: 0,
            pre: Default::default(),
            build: Default::default(),
        }
    } else {
        Version {
            major: 0,
            minor: 0,
            patch: lower_bound.patch + 1,
            pre: Default::default(),
            build: Default::default(),
        }
    };

    Some((lower_bound, upper_bound))
}

/// Determine whether `req_a` is a superset of `req_b`, i.e., whether the
/// version range matched by `req_a` includes all versions matched by `req_b`.
fn is_superset_of(req_a: &VersionReq, req_b: &VersionReq) -> bool {
    if let (Some((a_lower, a_upper)), Some((b_lower, b_upper))) =
        (get_caret_bounds(req_a), get_caret_bounds(req_b))
    {
        return a_lower <= b_lower && a_upper >= b_upper;
    }

    // If it cannot be parsed as a caret range, handle conservatively (return
    // `false`).
    false
}
