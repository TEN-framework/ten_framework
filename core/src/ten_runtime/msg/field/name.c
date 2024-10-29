//
// Copyright © 2024 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#include "include_internal/ten_runtime/common/constant_str.h"
#include "include_internal/ten_runtime/msg/loop_fields.h"
#include "include_internal/ten_runtime/msg/msg.h"
#include "ten_utils/lib/json.h"
#include "ten_utils/lib/string.h"
#include "ten_utils/macro/check.h"
#include "ten_utils/macro/mark.h"
#include "ten_utils/value/value.h"
#include "ten_utils/value/value_get.h"
#include "ten_utils/value/value_kv.h"

bool ten_raw_msg_name_to_json(ten_msg_t *self, ten_json_t *json,
                              ten_error_t *err) {
  TEN_ASSERT(self && ten_raw_msg_check_integrity(self), "Should not happen.");
  TEN_ASSERT(json, "Should not happen.");

  ten_json_t *ten_json =
      ten_json_object_peek_object_forcibly(json, TEN_STR_UNDERLINE_TEN);
  TEN_ASSERT(ten_json, "Should not happen.");

  const char *msg_name = ten_raw_msg_get_name(self);

  ten_json_t *name_json = ten_json_create_string(msg_name);
  TEN_ASSERT(name_json, "Should not happen.");

  ten_json_object_set_new(ten_json, TEN_STR_NAME, name_json);

  return true;
}

bool ten_raw_msg_name_from_json(ten_msg_t *self, ten_json_t *json,
                                TEN_UNUSED ten_error_t *err) {
  TEN_ASSERT(self && ten_raw_msg_check_integrity(self), "Should not happen.");
  TEN_ASSERT(json && ten_json_check_integrity(json), "Should not happen.");

  ten_json_t *ten_json =
      ten_json_object_peek_object(json, TEN_STR_UNDERLINE_TEN);
  if (!ten_json) {
    return true;
  }

  ten_json_t *name_json = ten_json_object_peek(ten_json, TEN_STR_NAME);
  if (!name_json) {
    return true;
  }

  if (ten_json_is_string(name_json)) {
    const char *msg_name = ten_json_peek_string_value(name_json);
    ten_raw_msg_set_name(self, msg_name, NULL);
  } else {
    TEN_LOGW("command should be a string.");
  }

  return true;
}

void ten_raw_msg_name_copy(ten_msg_t *self, ten_msg_t *src,
                           TEN_UNUSED ten_list_t *excluded_field_ids) {
  TEN_ASSERT(src && ten_raw_msg_check_integrity(src), "Should not happen.");
  ten_string_copy(ten_value_peek_string(&self->name),
                  ten_value_peek_string(&src->name));
}

bool ten_raw_msg_name_process(ten_msg_t *self,
                              ten_raw_msg_process_one_field_func_t cb,
                              void *user_data, ten_error_t *err) {
  TEN_ASSERT(self && ten_raw_msg_check_integrity(self), "Should not happen.");

  ten_msg_field_process_data_t name_field;
  ten_msg_field_process_data_init(&name_field, TEN_STR_NAME, &self->name,
                                  false);

  bool rc = cb(self, &name_field, user_data, err);

  if (name_field.value_is_changed_after_process) {
    TEN_MSG_TYPE msg_type_spec_by_name = ten_msg_type_from_unique_name_string(
        ten_value_peek_raw_str(name_field.field_value));
    if (msg_type_spec_by_name != TEN_MSG_TYPE_INVALID) {
      self->type = msg_type_spec_by_name;
    }
  }

  return rc;
}
