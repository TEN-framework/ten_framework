//
// Copyright © 2024 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#pragma once

#include "ten_runtime/ten_config.h"

#include <stdint.h>

#include "include_internal/ten_runtime/msg/msg.h"
#include "ten_runtime/msg/video_frame/video_frame.h"

#define TEN_VIDEO_FRAME_SIGNATURE 0xD03493E95807AC78U

typedef struct ten_video_frame_payload_t {
} ten_video_frame_payload_t;

typedef struct ten_video_frame_t {
  ten_msg_t msg_hdr;
  ten_signature_t signature;

  ten_value_t pixel_fmt;  // int32 (TEN_PIXEL_FMT)
  ten_value_t timestamp;  // int64
  ten_value_t width;      // int32
  ten_value_t height;     // int32
  ten_value_t is_eof;     // bool
  ten_value_t data;       // buf
} ten_video_frame_t;

TEN_RUNTIME_PRIVATE_API bool ten_raw_video_frame_check_integrity(
    ten_video_frame_t *self);

TEN_RUNTIME_API ten_video_frame_payload_t *ten_raw_video_frame_raw_payload(
    ten_video_frame_t *self);

TEN_RUNTIME_PRIVATE_API ten_msg_t *ten_raw_video_frame_as_msg_clone(
    ten_msg_t *self, ten_list_t *excluded_field_ids);

TEN_RUNTIME_PRIVATE_API ten_json_t *ten_raw_video_frame_as_msg_to_json(
    ten_msg_t *self, ten_error_t *err);

TEN_RUNTIME_PRIVATE_API void ten_raw_video_frame_init(ten_video_frame_t *self);

TEN_RUNTIME_PRIVATE_API void ten_raw_video_frame_destroy(
    ten_video_frame_t *self);

TEN_RUNTIME_PRIVATE_API bool ten_raw_video_frame_check_type_and_name(
    ten_msg_t *self, const char *type_str, const char *name_str,
    ten_error_t *err);

TEN_RUNTIME_PRIVATE_API TEN_PIXEL_FMT
ten_raw_video_frame_get_pixel_fmt(ten_video_frame_t *self);

TEN_RUNTIME_PRIVATE_API bool ten_raw_video_frame_set_pixel_fmt(
    ten_video_frame_t *self, TEN_PIXEL_FMT pixel_fmt);

TEN_RUNTIME_PRIVATE_API int64_t
ten_raw_video_frame_get_timestamp(ten_video_frame_t *self);

TEN_RUNTIME_PRIVATE_API int32_t
ten_raw_video_frame_get_width(ten_video_frame_t *self);

TEN_RUNTIME_PRIVATE_API int32_t
ten_raw_video_frame_get_height(ten_video_frame_t *self);

TEN_RUNTIME_PRIVATE_API size_t
ten_raw_video_frame_get_data_size(ten_video_frame_t *self);

TEN_RUNTIME_PRIVATE_API bool ten_raw_video_frame_is_eof(
    ten_video_frame_t *self);

TEN_RUNTIME_PRIVATE_API bool ten_raw_video_frame_set_width(
    ten_video_frame_t *self, int32_t width);

TEN_RUNTIME_PRIVATE_API bool ten_raw_video_frame_set_height(
    ten_video_frame_t *self, int32_t height);

TEN_RUNTIME_PRIVATE_API bool ten_raw_video_frame_set_timestamp(
    ten_video_frame_t *self, int64_t timestamp);

TEN_RUNTIME_PRIVATE_API bool ten_raw_video_frame_set_eof(
    ten_video_frame_t *self, bool is_eof);

TEN_RUNTIME_PRIVATE_API bool ten_raw_video_frame_as_msg_init_from_json(
    ten_msg_t *self, ten_json_t *json, ten_error_t *err);

TEN_RUNTIME_PRIVATE_API bool ten_raw_video_frame_set_ten_property(
    ten_msg_t *self, ten_list_t *paths, ten_value_t *value, ten_error_t *err);

TEN_RUNTIME_PRIVATE_API ten_value_t *ten_raw_video_frame_peek_ten_property(
    ten_msg_t *self, ten_list_t *paths, ten_error_t *err);

TEN_RUNTIME_PRIVATE_API bool ten_raw_video_frame_loop_all_fields(
    ten_msg_t *self, ten_raw_msg_process_one_field_func_t cb, void *user_data,
    ten_error_t *err);
