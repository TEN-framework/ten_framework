//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#pragma once

#include "ten_runtime/ten_config.h"

#include <stdbool.h>

#include "ten_utils/value/value.h"

typedef struct ten_app_t ten_app_t;
typedef struct TelemetrySystem TelemetrySystem;

TEN_RUNTIME_PRIVATE_API bool ten_app_init_telemetry_system(ten_app_t *self,
                                                           ten_value_t *value);

TEN_RUNTIME_PRIVATE_API void ten_app_deinit_telemetry_system(ten_app_t *self);
