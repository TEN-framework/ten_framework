//
// Copyright © 2024 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to https://github.com/TEN-framework/ten_framework/LICENSE for more
// information.
//
#pragma once

#include "ten_utils/ten_config.h"

#include <stdbool.h>

#include "ten_utils/lib/json.h"

typedef struct ten_value_t ten_value_t;

TEN_UTILS_API ten_value_t *ten_value_from_json(ten_json_t *json);

TEN_UTILS_API ten_json_t *ten_value_to_json(ten_value_t *self);
