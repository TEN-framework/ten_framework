//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::collections::HashMap;

use ten_rust::pkg_info::manifest::api::{
    ManifestApiCmdLike, ManifestApiDataLike, ManifestPropertyAttributes,
};

use super::graphs::nodes::{
    DesignerApiCmdLike, DesignerApiDataLike, DesignerPropertyAttributes,
};

pub fn get_designer_property_hashmap_from_pkg(
    items: HashMap<String, ManifestPropertyAttributes>,
) -> HashMap<String, DesignerPropertyAttributes> {
    items.into_iter().map(|(k, v)| (k, v.into())).collect()
}

pub fn get_designer_api_cmd_likes_from_pkg(
    items: Vec<ManifestApiCmdLike>,
) -> Vec<DesignerApiCmdLike> {
    items.into_iter().map(|v| v.into()).collect()
}

pub fn get_designer_api_data_likes_from_pkg(
    items: Vec<ManifestApiDataLike>,
) -> Vec<DesignerApiDataLike> {
    items.into_iter().map(|v| v.into()).collect()
}
