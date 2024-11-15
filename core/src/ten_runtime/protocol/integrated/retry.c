//
// Copyright © 2024 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#include "include_internal/ten_runtime/protocol/integrated/retry.h"

void ten_protocol_integrated_retry_config_default_init(
    ten_protocol_integrated_retry_config_t *self) {
  // TODO(xilin): Disable the retry mechanism by default.
  self->enable = true;
  self->max_retries = 5;
  self->interval_ms = 500;
}
