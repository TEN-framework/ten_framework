//
// Copyright © 2024 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#pragma once

#include "ten_runtime/ten_config.h"

#include <node_api.h>

#include "include_internal/ten_runtime/binding/nodejs/msg/msg.h"

typedef struct ten_nodejs_audio_frame_t {
  ten_nodejs_msg_t msg;
} ten_nodejs_audio_frame_t;

TEN_RUNTIME_PRIVATE_API napi_value
ten_nodejs_audio_frame_wrap(napi_env env, ten_shared_ptr_t *audio_frame);

TEN_RUNTIME_API napi_value
ten_nodejs_audio_frame_module_init(napi_env env, napi_value exports);