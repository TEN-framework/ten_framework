//
// Copyright © 2024 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#include "include_internal/ten_runtime/binding/nodejs/app/app.h"

#include "include_internal/ten_runtime/binding/nodejs/common/common.h"
#include "ten_runtime/binding/common.h"
#include "ten_utils/lib/signature.h"
#include "ten_utils/macro/memory.h"
#include "ten_utils/sanitizer/thread_check.h"

// Invoked when the JS app finalizes.
static void ten_nodejs_app_finalize(napi_env env, void *data,
                                    void *hint) {

}

static napi_value ten_nodejs_app_create(napi_env env, napi_callback_info info) {
  TEN_ASSERT(env, "Should not happen.");

  const size_t argc = 1;
  napi_value args[argc];  // this
  if (!ten_nodejs_get_js_func_args(env, info, args, argc)) {
    napi_fatal_error(NULL, NAPI_AUTO_LENGTH,
                     "Incorrect number of parameters passed.",
                     NAPI_AUTO_LENGTH);
    TEN_ASSERT(0, "Should not happen.");
    goto done;
  }

  ten_nodejs_app_t *app_bridge = TEN_MALLOC(sizeof(ten_nodejs_app_t));
  TEN_ASSERT(app_bridge, "Failed to allocate memory.");

  ten_signature_set(&app_bridge->signature, TEN_NODEJS_APP_SIGNATURE);

  // The ownership of the RTE app_bridge is the JS main thread.
  ten_sanitizer_thread_check_init_with_current_thread(
      &app_bridge->thread_check);

  // Wraps the native bridge instance ('app_bridge') in the javascript APP
  // object (args[0]). And the returned reference ('js_ref' here) is a weak
  // reference, meaning it has a reference count of 0.
  napi_status status =
      napi_wrap(env, args[0], app_bridge, ten_nodejs_app_finalize, NULL,
                &app_bridge->js_app_ref);
  GOTO_LABEL_IF_NAPI_FAIL(error, status == napi_ok,
                          "Failed to bind JS app & bridge: %d", status);

  goto done;

error:
  if (app_bridge) {
    TEN_FREE(app_bridge);
  }

done:
  return js_undefined(env);
}

napi_value ten_nodejs_app_module_init(napi_env env, napi_value exports) {
  return exports;
}
