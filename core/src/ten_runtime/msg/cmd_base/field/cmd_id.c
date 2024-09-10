//
// This file is part of the TEN Framework project.
// See https://github.com/TEN-framework/ten_framework/LICENSE for license
// information.
//
#include "include_internal/ten_runtime/msg/cmd_base/field/cmd_id.h"

#include "include_internal/ten_runtime/common/constant_str.h"
#include "include_internal/ten_runtime/msg/cmd_base/cmd_base.h"
#include "include_internal/ten_runtime/msg/msg.h"
#include "ten_utils/lib/json.h"
#include "ten_utils/lib/string.h"
#include "ten_utils/log/log.h"
#include "ten_utils/macro/check.h"

bool ten_cmd_base_put_cmd_id_to_json(ten_msg_t *self, ten_json_t *json,
                                     ten_error_t *err) {
  TEN_ASSERT(self && ten_raw_msg_check_integrity(self) && json,
             "Should not happen.");

  ten_json_t *ten_json =
      ten_json_object_peek_object_forcibly(json, TEN_STR_UNDERLINE_TEN);
  TEN_ASSERT(ten_json, "Should not happen.");

  ten_cmd_base_t *cmd = (ten_cmd_base_t *)self;
  ten_json_t *cmd_id_json =
      ten_json_create_string(ten_string_get_raw_str(&cmd->cmd_id));
  TEN_ASSERT(cmd_id_json, "Should not happen.");

  ten_json_object_set_new(ten_json, TEN_STR_CMD_ID, cmd_id_json);

  return true;
}

bool ten_cmd_base_get_cmd_id_from_json(ten_msg_t *self, ten_json_t *json,
                                       TEN_UNUSED ten_error_t *err) {
  TEN_ASSERT(self && ten_raw_msg_check_integrity(self) &&
                 ten_raw_msg_is_cmd_base(self),
             "Should not happen.");
  TEN_ASSERT(ten_json_check_integrity(json), "Should not happen.");

  ten_json_t *ten_json =
      ten_json_object_peek_object(json, TEN_STR_UNDERLINE_TEN);
  if (!ten_json) {
    // There is no 'ten' field in json, skip the from_json process.
    return true;
  }

  ten_json_t *cmd_id_json = ten_json_object_peek(ten_json, TEN_STR_CMD_ID);
  if (!cmd_id_json) {
    // There is no 'ten::cmd_id' field in json, skip the from_json process.
    return true;
  }

  if (ten_json_is_string(cmd_id_json)) {
    ten_raw_cmd_base_set_cmd_id((ten_cmd_base_t *)self,
                                ten_json_peek_string_value(cmd_id_json));
  } else {
    TEN_LOGW("cmd_id should be a string.");
  }

  return true;
}

void ten_cmd_base_copy_cmd_id(ten_msg_t *self, ten_msg_t *src,
                              TEN_UNUSED ten_list_t *excluded_field_ids) {
  TEN_ASSERT(src && ten_raw_msg_check_integrity(src), "Should not happen.");

  ten_string_init_formatted(
      &((ten_cmd_base_t *)self)->cmd_id, "%s",
      ten_string_get_raw_str(&((ten_cmd_base_t *)src)->cmd_id));
}
