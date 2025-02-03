//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#include "include_internal/ten_runtime/app/metric.h"

#include "include_internal/ten_runtime/app/app.h"

#if defined(TEN_ENABLE_TEN_RUST_APIS)

MetricSystem *ten_app_get_metric_system(ten_app_t *self) {
  TEN_ASSERT(self && ten_app_check_integrity(self, false), "Invalid argument.");

  return self->metric_system;
}

#endif
