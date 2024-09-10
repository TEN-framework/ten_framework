//
// This file is part of the TEN Framework project.
// See https://github.com/TEN-framework/ten_framework/LICENSE for license
// information.
//
#pragma once

#include "ten_runtime/ten_config.h"

#include "include_internal/ten_runtime/msg/cmd_base/cmd/cmd.h"

typedef struct ten_cmd_timeout_t {
  ten_cmd_t cmd_hdr;

  uint32_t timer_id;
} ten_cmd_timeout_t;

TEN_RUNTIME_PRIVATE_API void ten_cmd_timeout_set_timer_id(
    ten_shared_ptr_t *self, uint32_t timer_id);

TEN_RUNTIME_PRIVATE_API bool ten_raw_cmd_timeout_as_msg_init_from_json(
    ten_msg_t *self, ten_json_t *json, ten_error_t *err);

TEN_RUNTIME_PRIVATE_API ten_msg_t *ten_raw_cmd_timeout_as_msg_create_from_json(
    ten_json_t *json, ten_error_t *err);

TEN_RUNTIME_PRIVATE_API ten_shared_ptr_t *ten_cmd_timeout_create(
    uint32_t timer_id);

TEN_RUNTIME_PRIVATE_API void ten_raw_cmd_timeout_as_msg_destroy(
    ten_msg_t *self);

TEN_RUNTIME_PRIVATE_API void ten_raw_cmd_timeout_set_timer_id(
    ten_cmd_timeout_t *self, uint32_t timer_id);

TEN_RUNTIME_PRIVATE_API uint32_t
ten_raw_cmd_timeout_get_timer_id(ten_cmd_timeout_t *self);

TEN_RUNTIME_PRIVATE_API ten_json_t *ten_raw_cmd_timeout_as_msg_to_json(
    ten_msg_t *self, ten_error_t *err);

TEN_RUNTIME_API uint32_t ten_cmd_timeout_get_timer_id(ten_shared_ptr_t *self);
