//
// This file is part of the TEN Framework project.
// See https://github.com/TEN-framework/ten_framework/LICENSE for license
// information.
//
#pragma once

#include "ten_runtime/ten_config.h"

#include <stdbool.h>

#include "ten_utils/lib/string.h"

typedef struct ten_app_t ten_app_t;
typedef struct ten_protocol_t ten_protocol_t;

TEN_RUNTIME_PRIVATE_API bool ten_app_create_endpoint(ten_app_t *self,
                                                     ten_string_t *uri);

TEN_RUNTIME_PRIVATE_API bool ten_app_is_endpoint_closed(ten_app_t *self);

TEN_RUNTIME_PRIVATE_API void ten_app_create_protocol_context_store(
    ten_app_t *self);
