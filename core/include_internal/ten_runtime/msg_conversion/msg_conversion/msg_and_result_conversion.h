//
// Copyright © 2024 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to https://github.com/TEN-framework/ten_framework/LICENSE for more
// information.
//
#pragma once

#include "ten_runtime/ten_config.h"

#include "ten_utils/lib/smart_ptr.h"

typedef struct ten_msg_conversion_operation_t ten_msg_conversion_operation_t;

typedef struct ten_msg_and_result_conversion_t {
  ten_shared_ptr_t *msg;
  ten_msg_conversion_operation_t *operation;
} ten_msg_and_result_conversion_t;

TEN_RUNTIME_PRIVATE_API ten_msg_and_result_conversion_t *
ten_msg_and_result_conversion_create(
    ten_shared_ptr_t *msg, ten_msg_conversion_operation_t *result_conversion);

TEN_RUNTIME_PRIVATE_API void ten_msg_and_result_conversion_destroy(
    ten_msg_and_result_conversion_t *self);
