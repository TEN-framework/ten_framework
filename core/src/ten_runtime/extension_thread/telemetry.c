//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#include "include_internal/ten_runtime/app/telemetry.h"

#include "include_internal/ten_runtime/app/app.h"
#include "include_internal/ten_runtime/engine/engine.h"
#include "include_internal/ten_runtime/extension_context/extension_context.h"
#include "include_internal/ten_runtime/extension_thread/extension_thread.h"
#include "include_internal/ten_runtime/extension_thread/telemetry.h"
#include "ten_utils/lib/time.h"

#if defined(TEN_ENABLE_TEN_RUST_APIS)
#include "include_internal/ten_rust/ten_rust.h"
#endif

#if defined(TEN_ENABLE_TEN_RUST_APIS)

static MetricHandle *ten_extension_thread_get_metric_msg_queue_stay_time_us(
    ten_extension_thread_t *self) {
  TEN_ASSERT(self && ten_extension_thread_check_integrity(self, true),
             "Invalid argument.");

  ten_extension_context_t *extension_context = self->extension_context;
  TEN_ASSERT(extension_context && ten_extension_context_check_integrity(
                                      extension_context, false),
             "Should not happen.");

  ten_engine_t *engine = extension_context->engine;
  TEN_ASSERT(engine && ten_engine_check_integrity(engine, false),
             "Should not happen.");

  ten_app_t *app = engine->app;
  TEN_ASSERT(app && ten_app_check_integrity(app, false), "Should not happen.");

  return app->metric_msg_queue_stay_time_us;
}

void ten_extension_thread_record_msg_queue_stay_time(
    ten_extension_thread_t *self, int64_t timestamp) {
  TEN_ASSERT(self, "Invalid argument.");
  TEN_ASSERT(ten_extension_thread_check_integrity(self, true),
             "Invalid use of extension_thread %p.", self);

  int64_t duration_us = ten_current_time_us() - timestamp;
  MetricHandle *msg_queue_stay_time =
      ten_extension_thread_get_metric_msg_queue_stay_time_us(self);
  if (msg_queue_stay_time) {
    ten_metric_gauge_set(msg_queue_stay_time, (double)duration_us);
  }
}

#endif
