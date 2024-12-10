//
// Copyright © 2024 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#include "include_internal/ten_runtime/binding/nodejs/common/common.h"

#include "ten_utils/lib/string.h"
#include "ten_utils/macro/check.h"
#include "ten_utils/macro/memory.h"

napi_value js_undefined(napi_env env) {
  TEN_ASSERT(env, "Should not happen.");

  napi_value js_undefined_value = NULL;
  napi_status status = napi_get_undefined(env, &js_undefined_value);
  ASSERT_IF_NAPI_FAIL(status == napi_ok,
                      "Failed to get type JS undefined value: %d", status);

  return js_undefined_value;
}

bool ten_nodejs_get_js_func_args(napi_env env, napi_callback_info info,
                                 napi_value *args, size_t argc) {
  TEN_ASSERT(env, "Should not happen.");
  TEN_ASSERT(info, "Should not happen.");
  TEN_ASSERT(args, "Should not happen.");

  size_t actual_argc = argc;
  napi_status status =
      napi_get_cb_info(env, info, &actual_argc, args, NULL, NULL);
  ASSERT_IF_NAPI_FAIL(status == napi_ok,
                      "Failed to get JS function arguments: %d", status);

  if (actual_argc != argc) {
    ten_string_t err;
    ten_string_init_formatted(&err, "Expected %zu arguments, got %zu arguments",
                              argc, actual_argc);

    TEN_LOGE("%s", ten_string_get_raw_str(&err));

    status = napi_throw_error(env, "EINVAL", ten_string_get_raw_str(&err));
    ASSERT_IF_NAPI_FAIL(status == napi_ok, "Failed to throw JS exception: %d",
                        status);

    ten_string_deinit(&err);
    return false;
  }

  return true;
}

void ten_nodejs_report_and_clear_error_(napi_env env, napi_status orig_status,
                                        const char *func, int line) {
  TEN_ASSERT(func, "Should not happen.");

  TEN_LOGE("(%s:%d) Failed to invoke napi function, status: %d", func, line,
           orig_status);

  const napi_extended_error_info *error_info = NULL;
  napi_status status = napi_get_last_error_info((env), &error_info);
  ASSERT_IF_NAPI_FAIL(status == napi_ok,
                      "Failed to get napi last error info: %d", status);

  const char *err_message = error_info->error_message;

  TEN_LOGE("napi error message: %s", err_message);

  // Check if there is any pending JS exception.
  bool pending = false;
  status = napi_is_exception_pending(env, &pending);
  ASSERT_IF_NAPI_FAIL(status == napi_ok,
                      "Failed to check if there is any pending JS exceptions "
                      "in the JS world: %d",
                      status);
  if (pending) {
    napi_value ex = NULL;
    status = napi_get_and_clear_last_exception(env, &ex);
    ASSERT_IF_NAPI_FAIL(status == napi_ok || ex != NULL,
                        "Failed to get latest JS exception: %d", status);

    napi_value ex_str = NULL;
    status = napi_coerce_to_string(env, ex, &ex_str);
    ASSERT_IF_NAPI_FAIL(status == napi_ok && ex_str != NULL,
                        "Failed to coerce JS exception string: %d", status);

    size_t str_size = 0;
    status = napi_get_value_string_utf8(env, ex_str, NULL, str_size, &str_size);
    ASSERT_IF_NAPI_FAIL(status == napi_ok,
                        "Failed to get the JS exception string length: %d",
                        status);

    char *buf = TEN_MALLOC(str_size + 1);
    TEN_ASSERT(buf, "Failed to allocate memory.");

    status = napi_get_value_string_utf8(env, ex_str, buf, str_size + 1, NULL);
    ASSERT_IF_NAPI_FAIL(status == napi_ok,
                        "Failed to get JS exception string: %d", status);

    buf[str_size] = '\0';
    TEN_LOGE("Exception: %s", buf);

    TEN_FREE(buf);

    // Trigger an 'uncaughtException' in JavaScript. Useful if an async callback
    // throws an exception with no way to recover.
    status = napi_fatal_exception(env, ex);
    ASSERT_IF_NAPI_FAIL(status == napi_ok,
                        "Failed to throw JS fatal exception: %d", status);
  } else {
    TEN_LOGW("No pending exceptions when napi API failed.");

    // Encountering napi error but can not get any exceptions. It means JS
    // runtime went wrong (one possible reason is that the JS runtime is
    // closing), so we don't throw exceptions into JS runtime here.
  }
}