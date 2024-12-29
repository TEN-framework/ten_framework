//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#include "include_internal/ten_runtime/msg/audio_frame/field/number_of_channel.h"

#include "include_internal/ten_runtime/common/constant_str.h"
#include "include_internal/ten_runtime/msg/audio_frame/audio_frame.h"
#include "include_internal/ten_runtime/msg/msg.h"
#include "ten_runtime/msg/audio_frame/audio_frame.h"
#include "ten_runtime/msg/msg.h"
#include "ten_utils/macro/check.h"
#include "ten_utils/macro/mark.h"

void ten_audio_frame_copy_number_of_channel(
    ten_msg_t *self, ten_msg_t *src,
    TEN_UNUSED ten_list_t *excluded_field_ids) {
  TEN_ASSERT(self && src && ten_raw_msg_check_integrity(src) &&
                 ten_raw_msg_get_type(src) == TEN_MSG_TYPE_AUDIO_FRAME,
             "Should not happen.");

  ten_raw_audio_frame_set_number_of_channel(
      (ten_audio_frame_t *)self,
      ten_raw_audio_frame_get_number_of_channel((ten_audio_frame_t *)src));
}

bool ten_audio_frame_process_number_of_channel(
    ten_msg_t *self, ten_raw_msg_process_one_field_func_t cb, void *user_data,
    ten_error_t *err) {
  TEN_ASSERT(self && ten_raw_msg_check_integrity(self), "Should not happen.");

  ten_msg_field_process_data_t number_of_channel_field;
  ten_msg_field_process_data_init(
      &number_of_channel_field, TEN_STR_NUMBER_OF_CHANNEL,
      &((ten_audio_frame_t *)self)->number_of_channel, false);

  bool rc = cb(self, &number_of_channel_field, user_data, err);

  return rc;
}
