//
// Copyright © 2024 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to https://github.com/TEN-framework/ten_framework/LICENSE for more
// information.
//
#pragma once

#include "ten_utils/ten_config.h"

#include <stdint.h>

TEN_UTILS_PRIVATE_API void ten_log_get_pid_tid(int64_t *pid, int64_t *tid);
