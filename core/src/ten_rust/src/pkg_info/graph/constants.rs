//
// Copyright © 2024 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
pub const ERR_MSG_APP_LOCALHOST_SINGLE_APP: &str =
    "'localhost' is not allowed in graph definition, and the graph seems to be a single-app graph, just remove the 'app' field";

pub const ERR_MSG_APP_LOCALHOST_MULTI_APP: &str =
    "'localhost' is not allowed in graph definition, change the content of 'app' field to be consistent with '_ten::uri'";

pub const ERR_MSG_APP_CONTENT_MIXED: &str = "Either all nodes should have 'app' declared, or none should, but not a mix of both.";

pub const ERR_MSG_APP_SHOULD_NOT_DECLARED: &str =
    "the 'app' should not be declared, as not any node has declared it";

pub const ERR_MSG_APP_SHOULD_DECLARED: &str =
    "the 'app' can not be none, as it has been declared in nodes";
