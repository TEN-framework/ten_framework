//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#include "include_internal/ten_runtime/binding/nodejs/addon/addon.h"
#include "include_internal/ten_runtime/binding/nodejs/common/common.h"
#include "ten_runtime/addon/extension/extension.h"

static void ten_nodejs_addon_create_and_attach_callbacks(
    napi_env env, ten_nodejs_addon_t *addon_bridge) {
  TEN_ASSERT(
      addon_bridge && ten_nodejs_addon_check_integrity(addon_bridge, true),
      "Should not happen.");

  napi_value js_addon = NULL;
  napi_status status = napi_get_reference_value(
      env, addon_bridge->bridge.js_instance_ref, &js_addon);
  ASSERT_IF_NAPI_FAIL(status == napi_ok && js_addon != NULL,
                      "Failed to get JS addon instance.");

  napi_value js_on_init_proxy =
      ten_nodejs_get_property(env, js_addon, "onInitProxy");
  CREATE_JS_CB_TSFN(addon_bridge->js_on_init, env, "[TSFN] addon::onInit",
                    js_on_init_proxy, ten_nodejs_invoke_addon_js_on_init);

  napi_value js_on_deinit_proxy =
      ten_nodejs_get_property(env, js_addon, "onDeinitProxy");
  CREATE_JS_CB_TSFN(addon_bridge->js_on_deinit, env, "[TSFN] addon::onDeinit",
                    js_on_deinit_proxy, ten_nodejs_invoke_addon_js_on_deinit);

  napi_value js_on_create_instance_proxy =
      ten_nodejs_get_property(env, js_addon, "onCreateInstanceProxy");
  CREATE_JS_CB_TSFN(addon_bridge->js_on_create_instance, env,
                    "[TSFN] addon::onCreateInstance",
                    js_on_create_instance_proxy,
                    ten_nodejs_invoke_addon_js_on_create_instance);
}

napi_value ten_nodejs_addon_manager_register_addon_as_extension(
    napi_env env, napi_callback_info info) {
  TEN_ASSERT(env && info, "Should not happen.");

  TEN_LOGD(
      "TEN JS Addon: ten_nodejs_addon_manager_register_addon_as_extension");

  ten_string_t addon_name;
  ten_string_init(&addon_name);

  const size_t argc = 2;
  napi_value argv[2];  // name, addon_instance
  if (!ten_nodejs_get_js_func_args(env, info, argv, argc)) {
    napi_fatal_error(NULL, NAPI_AUTO_LENGTH,
                     "Incorrect number of parameters passed.",
                     NAPI_AUTO_LENGTH);
    TEN_ASSERT(0, "Should not happen.");
  }

  if (!ten_nodejs_get_str_from_js(env, argv[0], &addon_name)) {
    napi_fatal_error(NULL, NAPI_AUTO_LENGTH,
                     "Failed to get addon name from JS string.",
                     NAPI_AUTO_LENGTH);
    TEN_ASSERT(0, "Should not happen.");
  }

  TEN_LOGI("Registering addon: %s", ten_string_get_raw_str(&addon_name));

  ten_nodejs_addon_t *addon_bridge = NULL;
  napi_status status = napi_unwrap(env, argv[1], (void **)&addon_bridge);
  RETURN_UNDEFINED_IF_NAPI_FAIL(status == napi_ok && addon_bridge != NULL,
                                "Failed to get addon bridge: %d", status);

  ten_string_copy(&addon_bridge->addon_name, &addon_name);

  ten_nodejs_addon_create_and_attach_callbacks(env, addon_bridge);

  ten_addon_host_t *c_addon_host = ten_addon_register_extension(
      ten_string_get_raw_str(&addon_name), NULL, &addon_bridge->c_addon,
      /* register_ctx= */ NULL);
  if (!c_addon_host) {
    napi_fatal_error(
        NULL, NAPI_AUTO_LENGTH,
        "Failed to register addon in ten_addon_register_extension.",
        NAPI_AUTO_LENGTH);
    TEN_ASSERT(0, "Should not happen.");
  }

  addon_bridge->c_addon_host = c_addon_host;

  ten_string_deinit(&addon_name);

  return js_undefined(env);
}
