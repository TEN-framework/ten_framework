//
// This file is part of the TEN Framework project.
// See https://github.com/TEN-framework/ten_framework/LICENSE for license
// information.
//
#include "include_internal/ten_utils/log/new.h"

#include <assert.h>
#include <stdbool.h>
#include <stdlib.h>

#include "include_internal/ten_utils/lib/string.h"
#include "include_internal/ten_utils/log/new_level.h"
#include "include_internal/ten_utils/log/new_pid.h"
#include "include_internal/ten_utils/log/new_time.h"
#include "ten_utils/lib/signature.h"
#include "ten_utils/lib/string.h"

bool ten_log_new_check_integrity(ten_log_new_t *self) {
  assert(self && "Invalid argument.");

  if (ten_signature_get(&self->signature) !=
      (ten_signature_t)TEN_LOG_NEW_SIGNATURE) {
    return false;
  }

  return true;
}

void ten_log_new_init(ten_log_new_t *self) {
  assert(self && "Invalid argument.");

  ten_signature_set(&self->signature, TEN_LOG_NEW_SIGNATURE);
}

ten_log_new_t *ten_log_new_create(void) {
  ten_log_new_t *log = malloc(sizeof(ten_log_new_t));
  assert(log && "Failed to allocate memory.");

  ten_log_new_init(log);

  return log;
}

void ten_log_new_deinit(ten_log_new_t *self) {
  assert(ten_log_new_check_integrity(self) && "Invalid argument.");
}

void ten_log_new_destroy(ten_log_new_t *self) {
  assert(ten_log_new_check_integrity(self) && "Invalid argument.");

  ten_log_new_deinit(self);
  free(self);
}

static const char *funcname(const char *func) { return func ? func : ""; }

static const char *filename(const char *path, size_t path_len,
                            size_t *filename_len) {
  assert(filename_len && "Invalid argument.");

  if (!path || path_len == 0) {
    *filename_len = 0;
    return "";
  }

  const char *filename = NULL;
  size_t pos = 0;

  // Start from the end of the path and go backwards.
  for (size_t i = path_len; i > 0; i--) {
    if (path[i - 1] == '/' || path[i - 1] == '\\') {
      filename = &path[i];
      pos = i;
      break;
    }
  }

  if (!filename) {
    filename = path;
    pos = 0;
  }

  // Calculate the length of the filename.
  *filename_len = path_len - pos;

  return filename;
}

void ten_log_new_log_from_va_list(ten_log_new_t *self, TEN_LOG_NEW_LEVEL level,
                                  const char *func_name, const char *file_name,
                                  size_t line_no, const char *fmt, va_list ap) {
  assert(ten_log_new_check_integrity(self) && "Invalid argument.");

  ten_string_t msg;
  ten_string_init_from_va_list(&msg, fmt, ap);

  ten_log_new_log(self, level, func_name, file_name, line_no,
                  ten_string_get_raw_str(&msg));

  ten_string_deinit(&msg);
}

void ten_log_new_log_with_size_from_va_list(
    ten_log_new_t *self, TEN_LOG_NEW_LEVEL level, const char *func_name,
    size_t func_name_len, const char *file_name, size_t file_name_len,
    size_t line_no, const char *fmt, va_list ap) {
  assert(ten_log_new_check_integrity(self) && "Invalid argument.");

  ten_string_t msg;
  ten_string_init_from_va_list(&msg, fmt, ap);

  ten_log_new_log_with_size(self, level, func_name, func_name_len, file_name,
                            file_name_len, line_no,
                            ten_string_get_raw_str(&msg), ten_string_len(&msg));

  ten_string_deinit(&msg);
}

void ten_log_new_log_formatted(ten_log_new_t *self, TEN_LOG_NEW_LEVEL level,
                               const char *func_name, const char *file_name,
                               size_t line_no, const char *fmt, ...) {
  assert(ten_log_new_check_integrity(self) && "Invalid argument.");

  va_list ap;
  va_start(ap, fmt);

  ten_log_new_log_from_va_list(self, level, func_name, file_name, line_no, fmt,
                               ap);

  va_end(ap);
}

void ten_log_new_log(ten_log_new_t *self, TEN_LOG_NEW_LEVEL level,
                     const char *func_name, const char *file_name,
                     size_t line_no, const char *msg) {
  ten_log_new_log_with_size(self, level, func_name, strlen(func_name),
                            file_name, strlen(file_name), line_no, msg,
                            strlen(msg));
}

void ten_log_new_log_with_size(ten_log_new_t *self, TEN_LOG_NEW_LEVEL level,
                               const char *func_name, size_t func_name_len,
                               const char *file_name, size_t file_name_len,
                               size_t line_no, const char *msg,
                               size_t msg_len) {
  assert(ten_log_new_check_integrity(self) && "Invalid argument.");

  ten_string_t buf;
  ten_string_init(&buf);

  struct tm time_info;
  size_t msec = 0;
  ten_log_new_get_time(&time_info, &msec);
  ten_log_new_add_time_string(&buf, &time_info, msec);

  int64_t pid = 0;
  int64_t tid = 0;
  ten_log_new_get_pid_tid(&pid, &tid);

  ten_string_append_formatted(&buf, " %d(%d) %c", pid, tid,
                              ten_log_new_level_char(level));

  if (func_name_len) {
    ten_string_append_formatted(&buf, " %.*s", func_name_len,
                                funcname(func_name));
  }

  size_t actual_file_name_len = 0;
  const char *actual_file_name =
      filename(file_name, file_name_len, &actual_file_name_len);
  if (actual_file_name_len) {
    ten_string_append_formatted(&buf, "@%.*s:%d", actual_file_name_len,
                                actual_file_name, line_no);
  }

  ten_string_append_formatted(&buf, " %.*s", msg_len, msg);

  (void)fprintf(stderr, "%s\n", ten_string_get_raw_str(&buf));

  ten_string_deinit(&buf);
}
