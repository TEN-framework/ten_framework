//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use ten_rust::pkg_info::{
    graph::node::GraphNode, pkg_type::PkgType,
    pkg_type_and_name::PkgTypeAndName,
};

use crate::designer::graphs::nodes::GraphNodesSingleResponseData;

impl From<GraphNodesSingleResponseData> for GraphNode {
    fn from(designer_extension: GraphNodesSingleResponseData) -> Self {
        GraphNode {
            type_and_name: PkgTypeAndName {
                pkg_type: PkgType::Extension,
                name: designer_extension.name,
            },
            addon: designer_extension.addon,
            extension_group: Some(designer_extension.extension_group.clone()),
            app: Some(designer_extension.app),
            property: designer_extension.property,
        }
    }
}
