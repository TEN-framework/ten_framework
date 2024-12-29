//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#include "include_internal/ten_runtime/msg/video_frame/field/width.h"

#include "include_internal/ten_runtime/common/constant_str.h"
#include "include_internal/ten_runtime/msg/msg.h"
#include "include_internal/ten_runtime/msg/video_frame/video_frame.h"
#include "ten_runtime/msg/msg.h"
#include "ten_runtime/msg/video_frame/video_frame.h"
#include "ten_utils/macro/check.h"
#include "ten_utils/macro/mark.h"

void ten_video_frame_copy_width(ten_msg_t *self, ten_msg_t *src,
                                TEN_UNUSED ten_list_t *excluded_field_ids) {
  TEN_ASSERT(self && src && ten_raw_msg_check_integrity(src) &&
                 ten_raw_msg_get_type(src) == TEN_MSG_TYPE_VIDEO_FRAME,
             "Should not happen.");

  ten_raw_video_frame_set_width(
      (ten_video_frame_t *)self,
      ten_raw_video_frame_get_width((ten_video_frame_t *)src));
}

bool ten_video_frame_process_width(ten_msg_t *self,
                                   ten_raw_msg_process_one_field_func_t cb,
                                   void *user_data, ten_error_t *err) {
  TEN_ASSERT(self && ten_raw_msg_check_integrity(self), "Should not happen.");

  ten_msg_field_process_data_t width_field;
  ten_msg_field_process_data_init(&width_field, TEN_STR_WIDTH,
                                  &((ten_video_frame_t *)self)->width, false);

  bool rc = cb(self, &width_field, user_data, err);

  return rc;
}
