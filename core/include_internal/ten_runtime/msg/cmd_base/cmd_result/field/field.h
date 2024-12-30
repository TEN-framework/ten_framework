//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#pragma once

#include "ten_runtime/ten_config.h"

typedef enum TEN_CMD_STATUS_FIELD {
  TEN_CMD_STATUS_FIELD_CMD_BASE_HDR,

  TEN_CMD_STATUS_FIELD_ORIGINAL_CMD_TYPE,
  TEN_CMD_STATUS_FIELD_STATUS_CODE,
  TEN_CMD_STATUS_FIELD_IS_FINAL,

  TEN_CMD_STATUS_FIELD_LAST,
} TEN_CMD_STATUS_FIELD;
