//
// Copyright © 2024 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#pragma once

#include "ten_runtime/ten_config.h"

#include <stdbool.h>

#include "include_internal/ten_runtime/msg/loop_fields.h"
#include "ten_utils/container/list.h"
#include "ten_utils/lib/error.h"
#include "ten_utils/value/value.h"

typedef struct ten_msg_t ten_msg_t;
typedef struct ten_cmd_t ten_cmd_t;

TEN_RUNTIME_PRIVATE_API bool ten_raw_cmd_custom_set_ten_property(
    ten_msg_t *self, ten_list_t *paths, ten_value_t *value, ten_error_t *err);

TEN_RUNTIME_PRIVATE_API bool ten_raw_cmd_custom_check_type_and_name(
    ten_msg_t *self, const char *type_str, const char *name_str,
    ten_error_t *err);

TEN_RUNTIME_PRIVATE_API bool ten_raw_cmd_custom_as_msg_init_from_json(
    ten_msg_t *self, ten_json_t *json, ten_error_t *err);

TEN_RUNTIME_PRIVATE_API void ten_raw_cmd_custom_as_msg_destroy(ten_msg_t *self);

TEN_RUNTIME_PRIVATE_API ten_msg_t *ten_raw_cmd_custom_as_msg_clone(
    ten_msg_t *self, ten_list_t *excluded_field_ids);

TEN_RUNTIME_PRIVATE_API ten_json_t *ten_raw_cmd_custom_to_json(
    ten_msg_t *self, ten_error_t *err);

TEN_RUNTIME_API ten_shared_ptr_t *ten_cmd_custom_create(void);

TEN_RUNTIME_PRIVATE_API ten_cmd_t *ten_raw_cmd_custom_create(
    const char *cmd_name);

TEN_RUNTIME_PRIVATE_API bool ten_raw_cmd_custom_loop_all_fields(
    ten_msg_t *self, ten_raw_msg_process_one_field_func_t cb, void *user_data,
    ten_error_t *err);
