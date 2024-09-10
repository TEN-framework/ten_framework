//
// This file is part of the TEN Framework project.
// See https://github.com/TEN-framework/ten_framework/LICENSE for license
// information.
//
#pragma once

#include "ten_runtime/ten_config.h"

#include "ten_utils/log/log.h"

typedef struct ten_env_t ten_env_t;

TEN_RUNTIME_PRIVATE_API void ten_log(ten_env_t *self, const char *func_name,
                                   const char *file_name, size_t lineno,
                                   TEN_LOG_LEVEL level, const char *tag,
                                   const char *fmt, ...);
