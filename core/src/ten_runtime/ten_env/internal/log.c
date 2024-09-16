//
// This file is part of the TEN Framework project.
// See https://github.com/TEN-framework/ten_framework/LICENSE for license
// information.
//
#include "include_internal/ten_runtime/ten_env/log.h"

#include "include_internal/ten_runtime/ten_env/ten_env.h"
#include "include_internal/ten_utils/log/new.h"
#include "ten_runtime/ten_env/ten_env.h"

void ten_env_log(ten_env_t *self, TEN_LOG_NEW_LEVEL level,
                 const char *func_name, const char *file_name, size_t line_no,
                 const char *msg) {
  TEN_ASSERT(self && ten_env_check_integrity(self, true), "Should not happen.");

  ten_log_new_log(&self->log, level, func_name, file_name, line_no, msg);
}

void ten_env_log_formatted(ten_env_t *self, TEN_LOG_NEW_LEVEL level,
                           const char *func_name, const char *file_name,
                           size_t line_no, const char *fmt, ...) {
  TEN_ASSERT(self && ten_env_check_integrity(self, true), "Should not happen.");

  va_list ap;
  va_start(ap, fmt);

  ten_log_new_log_from_va_list(&self->log, level, func_name, file_name, line_no,
                               fmt, ap);

  va_end(ap);
}

void ten_env_log_with_size_formatted(ten_env_t *self, TEN_LOG_NEW_LEVEL level,
                                     const char *func_name,
                                     size_t func_name_len,
                                     const char *file_name,
                                     size_t file_name_len, size_t line_no,
                                     const char *fmt, ...) {
  TEN_ASSERT(self && ten_env_check_integrity(self, true), "Should not happen.");

  va_list ap;
  va_start(ap, fmt);

  ten_log_new_log_with_size_from_va_list(&self->log, level, func_name,
                                         func_name_len, file_name,
                                         file_name_len, line_no, fmt, ap);

  va_end(ap);
}
