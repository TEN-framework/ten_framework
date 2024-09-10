//
// This file is part of the TEN Framework project.
// See https://github.com/TEN-framework/ten_framework/LICENSE for license
// information.
//
#pragma once

#include "ten_utils/ten_config.h"

#include <stdbool.h>

// This header file will be used by the Rust `bindgen` tool to generate the
// FFIs.

// In Rust, `ten_schema_t` will be represented as a raw pointer to an opaque
// struct. Rust cannot directly access the fields of a raw pointer, even if it
// knows the structure's layout.
typedef struct ten_schema_t ten_schema_t;

TEN_UTILS_API ten_schema_t *ten_schema_create_from_json_string_proxy(
    const char *json_string, const char **err_msg);

TEN_UTILS_API void ten_schema_destroy_proxy(const ten_schema_t *self);

TEN_UTILS_API bool ten_schema_adjust_and_validate_json_string_proxy(
    const ten_schema_t *self, const char *json_string, const char **err_msg);

TEN_UTILS_API bool ten_schema_is_compatible_proxy(const ten_schema_t *self,
                                                  const ten_schema_t *target,
                                                  const char **err_msg);
