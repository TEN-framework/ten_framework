//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#include "include_internal/ten_runtime/msg/field/type.h"

#include "include_internal/ten_runtime/common/constant_str.h"
#include "include_internal/ten_runtime/msg/msg.h"
#include "ten_utils/lib/json.h"
#include "ten_utils/macro/check.h"
#include "ten_utils/macro/mark.h"
#include "ten_utils/value/value.h"
#include "ten_utils/value/value_get.h"
#include "ten_utils/value/value_kv.h"

bool ten_raw_msg_type_from_json(ten_msg_t *self, ten_json_t *json,
                                TEN_UNUSED ten_error_t *err) {
  TEN_ASSERT(self && ten_raw_msg_check_integrity(self), "Should not happen.");
  TEN_ASSERT(json && ten_json_check_integrity(json), "Should not happen.");

  ten_json_t *ten_json =
      ten_json_object_peek_object(json, TEN_STR_UNDERLINE_TEN);
  if (!ten_json) {
    return true;
  }

  ten_json_t *type_json = ten_json_object_peek(ten_json, TEN_STR_TYPE);
  if (!type_json) {
    return true;
  }

  ten_json_t *name_json = ten_json_object_peek(ten_json, TEN_STR_NAME);

  self->type = ten_msg_type_from_type_and_name_string(
      ten_json_peek_string_value(type_json),
      name_json ? ten_json_peek_string_value(name_json) : NULL);

  return true;
}

bool ten_raw_msg_type_to_json(ten_msg_t *self, ten_json_t *json,
                              ten_error_t *err) {
  TEN_ASSERT(self && ten_raw_msg_check_integrity(self) && json,
             "Should not happen.");

  ten_json_t *ten_json =
      ten_json_object_peek_object_forcibly(json, TEN_STR_UNDERLINE_TEN);
  TEN_ASSERT(ten_json, "Should not happen.");

  ten_json_t *type_json = ten_json_create_string(
      ten_msg_type_to_string(ten_raw_msg_get_type(self)));
  TEN_ASSERT(type_json, "Should not happen.");

  ten_json_object_set_new(ten_json, TEN_STR_TYPE, type_json);

  return true;
}

void ten_raw_msg_type_copy(ten_msg_t *self, ten_msg_t *src,
                           ten_list_t *excluded_field_ids) {
  TEN_ASSERT(src && ten_raw_msg_check_integrity(src), "Should not happen.");
  self->type = src->type;
}

bool ten_raw_msg_type_process(ten_msg_t *self,
                              ten_raw_msg_process_one_field_func_t cb,
                              void *user_data, ten_error_t *err) {
  TEN_ASSERT(self && ten_raw_msg_check_integrity(self), "Should not happen.");

  ten_value_t *type_value = ten_value_create_string(
      ten_msg_type_to_string(ten_raw_msg_get_type(self)));

  ten_msg_field_process_data_t type_field;
  ten_msg_field_process_data_init(&type_field, TEN_STR_TYPE, type_value, false);

  bool rc = cb(self, &type_field, user_data, err);

  if (type_field.value_is_changed_after_process) {
    self->type = ten_msg_type_from_type_string(
        ten_value_peek_raw_str(type_field.field_value, err));
  }

  ten_value_destroy(type_value);
  return rc;
}
