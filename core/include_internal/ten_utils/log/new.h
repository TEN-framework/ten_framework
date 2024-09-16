//
// This file is part of the TEN Framework project.
// See https://github.com/TEN-framework/ten_framework/LICENSE for license
// information.
//
#pragma once

#include "ten_utils/ten_config.h"

#include <stdarg.h>
#include <stdbool.h>
#include <stddef.h>

#include "ten_utils/lib/signature.h"

#define TEN_LOG_NEW_SIGNATURE 0xC0EE0CE92149D61AU

typedef enum TEN_LOG_NEW_LEVEL {
  TEN_LOG_NEW_INVALID,

  TEN_LOG_NEW_VERBOSE,
  TEN_LOG_NEW_DEBUG,
  TEN_LOG_NEW_INFO,
  TEN_LOG_NEW_WARN,
  TEN_LOG_NEW_ERROR,
  TEN_LOG_NEW_FATAL,
} TEN_LOG_NEW_LEVEL;

typedef struct ten_log_new_t {
  ten_signature_t signature;
} ten_log_new_t;

#define TEN_LOG_NEW(log, level, fmt, ...)                                  \
  ten_log_new_log_formatted(log, level, __func__, __FILE__, __LINE__, fmt, \
                            ##__VA_ARGS__)

TEN_UTILS_PRIVATE_API bool ten_log_new_check_integrity(ten_log_new_t *self);

TEN_UTILS_API void ten_log_new_init(ten_log_new_t *self);

TEN_UTILS_PRIVATE_API ten_log_new_t *ten_log_new_create(void);

TEN_UTILS_API void ten_log_new_deinit(ten_log_new_t *self);

TEN_UTILS_PRIVATE_API void ten_log_new_destroy(ten_log_new_t *self);

TEN_UTILS_API void ten_log_new_log_formatted(
    ten_log_new_t *self, TEN_LOG_NEW_LEVEL level, const char *func_name,
    const char *file_name, size_t line_no, const char *fmt, ...);

TEN_UTILS_API void ten_log_new_log_with_size_from_va_list(
    ten_log_new_t *self, TEN_LOG_NEW_LEVEL level, const char *func_name,
    size_t func_name_len, const char *file_name, size_t file_name_len,
    size_t line_no, const char *fmt, va_list ap);

TEN_UTILS_API void ten_log_new_log_from_va_list(
    ten_log_new_t *self, TEN_LOG_NEW_LEVEL level, const char *func_name,
    const char *file_name, size_t line_no, const char *fmt, va_list ap);

TEN_UTILS_API void ten_log_new_log(ten_log_new_t *self, TEN_LOG_NEW_LEVEL level,
                                   const char *func_name, const char *file_name,
                                   size_t line_no, const char *msg);

TEN_UTILS_PRIVATE_API void ten_log_new_log_with_size(
    ten_log_new_t *self, TEN_LOG_NEW_LEVEL level, const char *func_name,
    size_t func_name_len, const char *file_name, size_t file_name_len,
    size_t line_no, const char *msg, size_t msg_len);
