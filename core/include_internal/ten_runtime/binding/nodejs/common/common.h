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

typedef struct ten_smart_ptr_t ten_shared_ptr_t;

typedef struct ten_nodejs_bridge_t {
  // The following two fields are used to prevent the bridge instance from being
  // finalized. The bridge instance is finalized when both of the following two
  // fields are destroyed.
  ten_shared_ptr_t *sp_ref_by_c;
  ten_shared_ptr_t *sp_ref_by_js;

  // The reference to the JS instance.
  napi_ref js_instance_ref;
} ten_nodejs_bridge_t;

#define EXPORT_FUNC(env, exports, func)                \
  do {                                                 \
    ten_nodejs_export_func(env, exports, #func, func); \
  } while (0)

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

TEN_RUNTIME_PRIVATE_API napi_value ten_nodejs_get_property(
    napi_env env, napi_value js_obj, const char *property_name);

TEN_RUNTIME_PRIVATE_API void ten_nodejs_export_func(napi_env env,
                                                    napi_value exports,
                                                    const char *func_name,
                                                    napi_callback func);