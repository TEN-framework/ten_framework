//
// This file is part of the TEN Framework project.
// See https://github.com/TEN-framework/ten_framework/LICENSE for license
// information.
//
#include "ten_runtime/ten_env/internal/log.h"

#include "include_internal/ten_runtime/ten_env/ten_env.h"
#include "include_internal/ten_utils/log/log.h"
#include "ten_runtime/ten_env/ten_env.h"
#include "ten_utils/lib/string.h"

// =-=-=
void ten_env_log_without_check_thread(ten_env_t *self, TEN_LOG_LEVEL level,
                                      const char *func_name,
                                      const char *file_name, size_t line_no,
                                      const char *msg) {
  TEN_ASSERT(self && ten_env_check_integrity(self, false),
             "Should not happen.");

  ten_string_t final_msg;
  ten_string_init_formatted(&final_msg, "[%s] %s",
                            ten_env_get_attached_instance_name(self), msg);

  ten_log_log(&ten_global_log, level, func_name, file_name, line_no,
              ten_string_get_raw_str(&final_msg));

  ten_string_deinit(&final_msg);
}

void ten_env_log(ten_env_t *self, TEN_LOG_LEVEL level, const char *func_name,
                 const char *file_name, size_t line_no, const char *msg) {
  TEN_ASSERT(self && ten_env_check_integrity(self, true), "Should not happen.");

  ten_string_t final_msg;
  ten_string_init_formatted(&final_msg, "[%s] %s",
                            ten_env_get_attached_instance_name(self), msg);

  ten_log_log(&ten_global_log, level, func_name, file_name, line_no,
              ten_string_get_raw_str(&final_msg));

  ten_string_deinit(&final_msg);
}

void ten_env_log_with_size_formatted(ten_env_t *self, TEN_LOG_LEVEL level,
                                     const char *func_name,
                                     size_t func_name_len,
                                     const char *file_name,
                                     size_t file_name_len, size_t line_no,
                                     const char *fmt, ...) {
  TEN_ASSERT(self && ten_env_check_integrity(self, true), "Should not happen.");

  ten_string_t final_msg;
  ten_string_init_formatted(&final_msg, "[%s] ",
                            ten_env_get_attached_instance_name(self));

  va_list ap;
  va_start(ap, fmt);

  ten_string_append_from_va_list(&final_msg, fmt, ap);

  va_end(ap);

  ten_log_log_with_size(&ten_global_log, level, func_name, func_name_len,
                        file_name, file_name_len, line_no,
                        ten_string_get_raw_str(&final_msg),
                        ten_string_len(&final_msg));

  ten_string_deinit(&final_msg);
}
