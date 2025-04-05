//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#include "include_internal/ten_utils/value/bindings/rust/value_proxy.h"

#include <stdlib.h>
#include <string.h>

#include "ten_utils/value/value.h"
#include "ten_utils/value/value_json.h"

// Creates a ten_value_t from a JSON string.
// The returned pointer must be freed with ten_value_destroy_proxy.
ten_value_t *ten_value_create_from_json_str_proxy(const char *json_str,
                                                  const char **err_msg) {
  // We use the existing ten_value_from_json_str function from the C API.
  ten_value_t *value = ten_value_from_json_str(json_str);
  if (value == NULL) {
    // If creation fails, set the error message.
    if (err_msg != NULL) {
      const char *error = "Failed to create TEN value from JSON string";
      *err_msg = strdup(error);
    }
    return NULL;
  }
  return value;
}

// Destroys a ten_value_t and frees associated memory.
void ten_value_destroy_proxy(const ten_value_t *value) {
  // Directly call the C function to destroy the ten_value_t.
  // Note: The const cast is safe here because we know this is the end of
  // the value's lifecycle and the existing C API requires a non-const pointer
  ten_value_destroy((ten_value_t *)value);
}
