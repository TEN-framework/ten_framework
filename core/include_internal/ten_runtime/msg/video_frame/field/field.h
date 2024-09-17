//
// Copyright © 2024 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to https://github.com/TEN-framework/ten_framework/LICENSE for more
// information.
//
#pragma once

typedef enum TEN_VIDEO_FRAME_FIELD {
  TEN_VIDEO_FRAME_FIELD_MSGHDR,

  TEN_VIDEO_FRAME_FIELD_PIXEL_FMT,
  TEN_VIDEO_FRAME_FIELD_TIMESTAMP,
  TEN_VIDEO_FRAME_FIELD_WIDTH,
  TEN_VIDEO_FRAME_FIELD_HEIGHT,
  TEN_VIDEO_FRAME_FIELD_BUF,

  TEN_VIDEO_FRAME_FIELD_LAST,
} TEN_VIDEO_FRAME_FIELD;
