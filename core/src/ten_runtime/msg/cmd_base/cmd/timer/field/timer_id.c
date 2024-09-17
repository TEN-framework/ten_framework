//
// This file is part of the TEN Framework project.
// See https://github.com/TEN-framework/ten_framework/LICENSE for license
// information.
//
#include "include_internal/ten_runtime/common/constant_str.h"
#include "include_internal/ten_runtime/msg/cmd_base/cmd/timer/cmd.h"
#include "include_internal/ten_runtime/msg/msg.h"
#include "ten_utils/lib/json.h"
#include "ten_utils/macro/check.h"
#include "ten_utils/macro/mark.h"

bool ten_cmd_timer_put_timer_id_to_json(ten_msg_t *self, ten_json_t *json,
                                        ten_error_t *err) {
  TEN_ASSERT(self && ten_raw_msg_check_integrity(self) && json &&
                 ten_raw_msg_get_type(self) == TEN_MSG_TYPE_CMD_TIMER,
             "Should not happen.");

  ten_json_t *ten_json =
      ten_json_object_peek_object_forcibly(json, TEN_STR_UNDERLINE_TEN);
  TEN_ASSERT(ten_json, "Should not happen.");

  ten_json_object_set_new(
      ten_json, TEN_STR_TIMER_ID,
      ten_json_create_integer(
          ten_raw_cmd_timer_get_timer_id((ten_cmd_timer_t *)self)));

  return true;
}

bool ten_cmd_timer_get_timer_id_from_json(ten_msg_t *self, ten_json_t *json,
                                          TEN_UNUSED ten_error_t *err) {
  TEN_ASSERT(self && ten_raw_msg_check_integrity(self) &&
                 ten_raw_msg_get_type(self) == TEN_MSG_TYPE_CMD_TIMER,
             "Should not happen.");
  TEN_ASSERT(json && ten_json_check_integrity(json), "Should not happen.");

  ten_json_t *ten_json =
      ten_json_object_peek_object(json, TEN_STR_UNDERLINE_TEN);
  if (!ten_json) {
    return true;
  }

  ten_json_t *timer_id_json = ten_json_object_peek(ten_json, TEN_STR_TIMER_ID);
  if (!timer_id_json) {
    return true;
  }

  if (ten_json_is_integer(timer_id_json)) {
    ten_cmd_timer_t *cmd = (ten_cmd_timer_t *)self;
    cmd->timer_id = ten_json_get_integer_value(timer_id_json);
  } else {
    TEN_LOGW("timer_id should be an integer.");
  }

  return true;
}
