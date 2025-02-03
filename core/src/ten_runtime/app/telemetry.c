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
#endif

#if defined(TEN_ENABLE_TEN_RUST_APIS)

static void ten_app_create_metric(ten_app_t *self) {
  TEN_ASSERT(self, "Invalid argument.");
  TEN_ASSERT(ten_app_check_integrity(self, true), "Invalid use of app %p.",
             self);
  TEN_ASSERT(!self->metric_msg_queue_stay_time_us, "Should not happen.");

  if (self->telemetry_system) {
    self->metric_msg_queue_stay_time_us = ten_metric_create(
        self->telemetry_system, 1, "msg_queue_stay_time",
        "The duration (in micro-seconds) that a message instance stays in the "
        "message queue before being processed.",
        NULL, 0);
    TEN_ASSERT(self->metric_msg_queue_stay_time_us, "Should not happen.");
  }
}

static void ten_app_destroy_metric(ten_app_t *self) {
  TEN_ASSERT(self, "Invalid argument.");
  TEN_ASSERT(ten_app_check_integrity(self, true),
             "Invalid use of extension_thread %p.", self);

  if (self->metric_msg_queue_stay_time_us) {
    ten_metric_destroy(self->metric_msg_queue_stay_time_us);
    self->metric_msg_queue_stay_time_us = NULL;
  }
}

#endif

bool ten_app_init_telemetry_system(ten_app_t *self, ten_value_t *value) {
#if defined(TEN_ENABLE_TEN_RUST_APIS)
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

  ten_app_create_metric(self);
#endif

  return true;
}

void ten_app_deinit_telemetry_system(ten_app_t *self) {
#if defined(TEN_ENABLE_TEN_RUST_APIS)
  if (self->telemetry_system) {
    ten_app_destroy_metric(self);
    ten_telemetry_system_shutdown(self->telemetry_system);
  }
#endif
}
