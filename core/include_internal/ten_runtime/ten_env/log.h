//
// This file is part of the TEN Framework project.
// See https://github.com/TEN-framework/ten_framework/LICENSE for license
// information.
//
#pragma once

#include "ten_runtime/ten_config.h"

#include "include_internal/ten_utils/log/new.h"

typedef struct ten_env_t ten_env_t;

TEN_RUNTIME_API void ten_env_log(ten_env_t *self, TEN_LOG_NEW_LEVEL level,
                                 const char *func_name, const char *file_name,
                                 size_t line_no, const char *msg);

TEN_RUNTIME_API void ten_env_log_formatted(
    ten_env_t *self, TEN_LOG_NEW_LEVEL level, const char *func_name,
    const char *file_name, size_t line_no, const char *fmt, ...);

TEN_RUNTIME_API void ten_env_log_with_size_formatted(
    ten_env_t *self, TEN_LOG_NEW_LEVEL level, const char *func_name,
    size_t func_name_len, const char *file_name, size_t file_name_len,
    size_t line_no, const char *fmt, ...);
