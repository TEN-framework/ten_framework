//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#pragma once

#include "ten_runtime/ten_config.h"

typedef struct ten_app_t ten_app_t;
typedef struct MetricSystem MetricSystem;

#if defined(TEN_ENABLE_TEN_RUST_APIS)

MetricSystem *ten_app_get_metric_system(ten_app_t *self);

#endif
