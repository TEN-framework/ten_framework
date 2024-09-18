//
// Copyright © 2024 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#pragma once

#include "ten_runtime/ten_config.h"

#include <stdbool.h>

#include "ten_utils/container/list.h"
#include "ten_utils/value/value.h"

typedef struct ten_msg_t ten_msg_t;
typedef struct ten_error_t ten_error_t;
typedef struct ten_c_value_t ten_c_value_t;

TEN_RUNTIME_API ten_list_t *ten_raw_msg_get_properties(ten_msg_t *self);

TEN_RUNTIME_API bool ten_msg_del_property(ten_shared_ptr_t *self,
                                          const char *path);

TEN_RUNTIME_API ten_value_t *ten_raw_msg_peek_property(ten_msg_t *self,
                                                       const char *path,
                                                       ten_error_t *err);

TEN_RUNTIME_API bool ten_raw_msg_properties_to_json(ten_msg_t *self,
                                                    ten_json_t *json,
                                                    ten_error_t *err);

TEN_RUNTIME_API bool ten_raw_msg_properties_from_json(ten_msg_t *self,
                                                      ten_json_t *json,
                                                      ten_error_t *err);

TEN_RUNTIME_API void ten_raw_msg_properties_copy(
    ten_msg_t *self, ten_msg_t *src, ten_list_t *excluded_field_ids);
