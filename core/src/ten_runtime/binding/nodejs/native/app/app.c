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
#include "ten_utils/macro/check.h"
#include "ten_utils/macro/mark.h"
#include "ten_utils/macro/memory.h"
#include "ten_utils/sanitizer/thread_check.h"

static void invoke_app_js_on_configure(napi_env env, napi_value fn,
                                       TEN_UNUSED void *context, void *data) {}

static void invoke_app_js_on_init(napi_env env, napi_value fn,
                                  TEN_UNUSED void *context, void *data) {}

static void invoke_app_js_on_close(napi_env env, napi_value fn,
                                   TEN_UNUSED void *context, void *data) {}

static void proxy_on_configure(ten_app_t *app, ten_env_t *ten_env) {}

static void proxy_on_init(ten_app_t *app, ten_env_t *ten_env) {}

static void proxy_on_deinit(ten_app_t *app, ten_env_t *ten_env) {}

static bool ten_nodejs_app_check_integrity(ten_nodejs_app_t *self,
                                           bool check_thread) {
  TEN_ASSERT(self, "Should not happen.");

  if (ten_signature_get(&self->signature) != TEN_NODEJS_APP_SIGNATURE) {
    return false;
  }

  if (check_thread &&
      !ten_sanitizer_thread_check_do_check(&self->thread_check)) {
    return false;
  }

  return true;
}

static void ten_nodejs_app_create_and_attach_callbacks(
    napi_env env, ten_nodejs_app_t *app_bridge) {
  TEN_ASSERT(app_bridge && ten_nodejs_app_check_integrity(app_bridge, true),
             "Should not happen.");

  napi_value js_app = NULL;
  napi_status status = napi_get_reference_value(
      env, app_bridge->bridge.js_instance_ref, &js_app);
  ASSERT_IF_NAPI_FAIL(status == napi_ok && js_app != NULL,
                      "Failed to get JS app instance.");

  napi_value js_on_configure_proxy =
      ten_nodejs_get_property(env, js_app, "onConfigureProxy");
  CREATE_JS_CB_TSFN(app_bridge->js_on_init, env, "[TSFN] app::onInit",
                    js_on_configure_proxy, invoke_app_js_on_configure);

  napi_value js_on_init_proxy =
      ten_nodejs_get_property(env, js_app, "onInitProxy");
  CREATE_JS_CB_TSFN(app_bridge->js_on_init, env, "[TSFN] app::onInit",
                    js_on_init_proxy, invoke_app_js_on_init);

  napi_value js_on_close_proxy =
      ten_nodejs_get_property(env, js_app, "onCloseProxy");
  CREATE_JS_CB_TSFN(app_bridge->js_on_close, env, "[TSFN] app::onClose",
                    js_on_close_proxy, invoke_app_js_on_close);
}

// Invoked when the JS app finalizes.
static void ten_nodejs_app_finalize(napi_env env, void *data, void *hint) {}

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
                &app_bridge->bridge.js_instance_ref);
  GOTO_LABEL_IF_NAPI_FAIL(error, status == napi_ok,
                          "Failed to bind JS app & bridge: %d", status);

  // Create the underlying TEN C app.
  app_bridge->c_app =
      ten_app_create(proxy_on_configure, proxy_on_init, proxy_on_deinit, NULL);
  ten_binding_handle_set_me_in_target_lang(
      (ten_binding_handle_t *)(app_bridge->c_app), app_bridge);

  ten_nodejs_app_create_and_attach_callbacks(env, app_bridge);

  goto done;

error:
  if (app_bridge) {
    TEN_FREE(app_bridge);
  }

done:
  return js_undefined(env);
}

napi_value ten_nodejs_app_module_init(napi_env env, napi_value exports) {
  TEN_ASSERT(env && exports, "Should not happen.");

  EXPORT_FUNC(env, exports, ten_nodejs_app_create);

  return exports;
}
