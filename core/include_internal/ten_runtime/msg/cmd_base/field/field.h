//
// This file is part of the TEN Framework project.
// See https://github.com/TEN-framework/ten_framework/LICENSE for license
// information.
//
#pragma once

#include "ten_runtime/ten_config.h"

typedef enum TEN_CMD_BASE_FIELD {
  TEN_CMD_BASE_FIELD_MSGHDR,

  TEN_CMD_BASE_FIELD_CMD_ID,
  TEN_CMD_BASE_FIELD_SEQ_ID,

  TEN_CMD_BASE_FIELD_ORIGINAL_CONNECTION,

  TEN_CMD_BASE_FIELD_RESPONSE_HANDLER,
  TEN_CMD_BASE_FIELD_RESPONSE_HANDLER_DATA,

  TEN_CMD_BASE_FIELD_LAST,
} TEN_CMD_BASE_FIELD;
