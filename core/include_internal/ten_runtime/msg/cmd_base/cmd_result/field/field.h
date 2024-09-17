//
// Copyright © 2024 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to https://github.com/TEN-framework/ten_framework/LICENSE for more
// information.
//
#pragma once

#include "ten_runtime/ten_config.h"

typedef enum TEN_CMD_STATUS_FIELD {
  TEN_CMD_STATUS_FIELD_CMD_BASE_HDR,

  TEN_CMD_STATUS_FIELD_ORIGINAL_CMD_TYPE,
  TEN_CMD_STATUS_FIELD_STATUS_CODE,

  TEN_CMD_STATUS_FIELD_LAST,
} TEN_CMD_STATUS_FIELD;
