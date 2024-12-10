//
// Copyright © 2024 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#pragma once

#include "ten_runtime/ten_config.h"

#include <node_api.h>

#include "ten_utils/log/log.h"

#define GOTO_LABEL_IF_NAPI_FAIL(label, expr, fmt, ...) \
  do {                                                 \
    if (!(expr)) {                                     \
      ten_nodejs_report_and_clear_error(env, status);  \
      TEN_LOGE(fmt, ##__VA_ARGS__);                    \
      goto label;                                      \
    }                                                  \
  } while (0)

#define ASSERT_IF_NAPI_FAIL(expr, fmt, ...) \
  do {                                      \
    if (!(expr)) {                          \
      TEN_LOGE(fmt, ##__VA_ARGS__);         \
      /* NOLINTNEXTLINE */                  \
      TEN_ASSERT(0, "Should not happen.");  \
      /* NOLINTNEXTLINE */                  \
      exit(-1);                             \
    }                                       \
  } while (0)

TEN_RUNTIME_PRIVATE_API napi_value js_undefined(napi_env env);

TEN_RUNTIME_PRIVATE_API bool ten_nodejs_get_js_func_args(
    napi_env env, napi_callback_info info, napi_value *args, size_t argc);

TEN_RUNTIME_PRIVATE_API void ten_nodejs_report_and_clear_error_(
    napi_env env, napi_status status, const char *func, int line);

#define ten_nodejs_report_and_clear_error(env, status) \
  ten_nodejs_report_and_clear_error_(env, status, __FUNCTION__, __LINE__)