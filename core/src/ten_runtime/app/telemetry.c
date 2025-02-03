//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#include "include_internal/ten_runtime/app/telemetry.h"

#include "include_internal/ten_runtime/app/app.h"
#include "include_internal/ten_runtime/common/constant_str.h"

#if defined(TEN_ENABLE_TEN_RUST_APIS)

#include "include_internal/ten_rust/ten_rust.h"

bool ten_app_init_telemetry_system(ten_app_t *self, ten_value_t *value) {
  TEN_ASSERT(self && ten_app_check_integrity(self, true), "Should not happen.");
  TEN_ASSERT(value && ten_value_check_integrity(value), "Should not happen.");

  if (!ten_value_is_object(value)) {
    TEN_LOGE("Invalid value type for property: telemetry. Expected an object.");
    return false;
  }

  ten_value_t *enabled_value = ten_value_object_peek(value, TEN_STR_ENABLED);
  if (!enabled_value || !ten_value_is_bool(enabled_value) ||
      !ten_value_get_bool(enabled_value, NULL)) {
    // If the `enabled` field does not exist or is not set to `true`, the
    // telemetry system will not be activated.
    return true;
  }

  // Check if the `telemetry` object contains the `endpoint` field.
  ten_value_t *endpoint_value = ten_value_object_peek(value, TEN_STR_ENDPOINT);
  if (endpoint_value && ten_value_is_string(endpoint_value)) {
    const char *endpoint = ten_value_peek_raw_str(endpoint_value, NULL);
    if (endpoint && strlen(endpoint) > 0) {
      self->telemetry_system = ten_telemetry_system_create(endpoint, NULL);
      if (!self->telemetry_system) {
        TEN_LOGE("Failed to create telemetry system with endpoint: %s",
                 endpoint);
        exit(EXIT_FAILURE);
      }
      return true;
    }
  }

  // If a valid `endpoint` is not provided, call
  // `ten_telemetry_system_create(NULL, NULL)`.
  self->telemetry_system = ten_telemetry_system_create(NULL, NULL);
  if (!self->telemetry_system) {
    TEN_LOGE("Failed to create telemetry system with default endpoint.");
    exit(EXIT_FAILURE);
  }

  return true;
}

TelemetrySystem *ten_app_get_telemetry_system(ten_app_t *self) {
  TEN_ASSERT(self && ten_app_check_integrity(self, false), "Invalid argument.");

  return self->telemetry_system;
}

#endif
