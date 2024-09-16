//
// This file is part of the TEN Framework project.
// See https://github.com/TEN-framework/ten_framework/LICENSE for license
// information.
//
#include "ten_utils/ten_config.h"

#include "include_internal/ten_utils/log/new_level.h"

#include <assert.h>

char ten_log_new_level_char(const TEN_LOG_NEW_LEVEL level) {
  switch (level) {
    case TEN_LOG_NEW_VERBOSE:
      return 'V';
    case TEN_LOG_NEW_DEBUG:
      return 'D';
    case TEN_LOG_NEW_INFO:
      return 'I';
    case TEN_LOG_NEW_WARN:
      return 'W';
    case TEN_LOG_NEW_ERROR:
      return 'E';
    case TEN_LOG_NEW_FATAL:
      return 'F';
    default:
      return '?';
  }
}
