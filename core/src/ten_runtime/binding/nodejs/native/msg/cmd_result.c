//
// Copyright © 2024 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#include "include_internal/ten_runtime/binding/nodejs/msg/cmd_result.h"

#include "include_internal/ten_runtime/binding/nodejs/common/common.h"
#include "include_internal/ten_runtime/msg/msg.h"
#include "js_native_api.h"
#include "ten_runtime/common/status_code.h"
#include "ten_runtime/msg/cmd_result/cmd_result.h"
#include "ten_utils/macro/memory.h"

static napi_ref js_cmd_result_constructor_ref = NULL;  // NOLINT

static napi_value ten_nodejs_cmd_result_register_class(
    napi_env env, napi_callback_info info) {
  const size_t argc = 1;
  napi_value args[argc];  // CmdResult constructor
  if (!ten_nodejs_get_js_func_args(env, info, args, argc)) {
    napi_fatal_error(NULL, NAPI_AUTO_LENGTH,
                     "Incorrect number of parameters passed.",
                     NAPI_AUTO_LENGTH);
    TEN_ASSERT(0, "Should not happen.");
    return NULL;
  }

  napi_status status =
      napi_create_reference(env, args[0], 1, &js_cmd_result_constructor_ref);
  if (status != napi_ok) {
    napi_fatal_error(
        NULL, NAPI_AUTO_LENGTH,
        "Failed to create JS reference to JS CmdResult constructor.",
        NAPI_AUTO_LENGTH);
    TEN_ASSERT(0,
               "Failed to create JS reference to JS CmdResult constructor: %d",
               status);
  }

  return js_undefined(env);
}

static void ten_nodejs_cmd_result_destroy(ten_nodejs_msg_t *self) {
  TEN_ASSERT(self, "Should not happen.");

  ten_nodejs_msg_deinit(self);

  TEN_FREE(self);
}

static void ten_nodejs_cmd_result_finalize(napi_env env, void *data,
                                           void *hint) {
  ten_nodejs_cmd_result_t *cmd_result_bridge = data;
  TEN_ASSERT(cmd_result_bridge, "Should not happen.");

  napi_delete_reference(env, cmd_result_bridge->msg.bridge.js_instance_ref);

  ten_nodejs_cmd_result_destroy(&cmd_result_bridge->msg);
}

static napi_value ten_nodejs_cmd_result_create(napi_env env,
                                               napi_callback_info info) {
  const size_t argc = 2;
  napi_value args[argc];  // this, status_code
  if (!ten_nodejs_get_js_func_args(env, info, args, argc)) {
    napi_fatal_error(NULL, NAPI_AUTO_LENGTH,
                     "Incorrect number of parameters passed.",
                     NAPI_AUTO_LENGTH);
    TEN_ASSERT(0, "Should not happen.");
  }

  TEN_STATUS_CODE status_code = TEN_STATUS_CODE_INVALID;
  napi_status status =
      napi_get_value_uint32(env, args[1], (uint32_t *)&status_code);
  if (status != napi_ok) {
    napi_fatal_error(NULL, NAPI_AUTO_LENGTH, "Failed to get status_code.",
                     NAPI_AUTO_LENGTH);
    TEN_ASSERT(0, "Should not happen.");
  }

  ten_error_t error;
  ten_error_init(&error);

  ten_shared_ptr_t *c_cmd_result = ten_cmd_result_create(status_code);
  TEN_ASSERT(c_cmd_result, "Failed to create cmd_result.");

  ten_nodejs_cmd_result_t *cmd_result_bridge =
      TEN_MALLOC(sizeof(ten_nodejs_cmd_result_t));
  TEN_ASSERT(cmd_result_bridge, "Failed to allocate memory.");

  ten_nodejs_msg_init_from_c_msg(&cmd_result_bridge->msg, c_cmd_result);
  // Decrement the reference count of c_cmd_result to indicate that the JS cmd
  // takes the full ownership of this c_cmd_result, in other words, when the JS
  // cmd is finalized, its C cmd would be destroyed, too.
  ten_shared_ptr_destroy(c_cmd_result);

  status =
      napi_wrap(env, args[0], cmd_result_bridge, ten_nodejs_cmd_result_finalize,
                NULL, &cmd_result_bridge->msg.bridge.js_instance_ref);
  if (status != napi_ok) {
    napi_fatal_error(NULL, NAPI_AUTO_LENGTH,
                     "Failed to wrap JS CmdResult object.", NAPI_AUTO_LENGTH);
    TEN_ASSERT(0, "Failed to wrap JS CmdResult object: %d", status);
  }

  return js_undefined(env);
}

napi_value ten_nodejs_cmd_result_wrap(napi_env env,
                                      ten_shared_ptr_t *cmd_result) {
  TEN_ASSERT(cmd_result && ten_msg_check_integrity(cmd_result),
             "Should not happen.");

  ten_nodejs_cmd_result_t *cmd_result_bridge =
      TEN_MALLOC(sizeof(ten_nodejs_cmd_result_t));
  TEN_ASSERT(cmd_result_bridge, "Failed to allocate memory.");

  ten_nodejs_msg_init_from_c_msg(&cmd_result_bridge->msg, cmd_result);

  napi_value js_status_code = NULL;
  napi_value js_create_shell_only_flag = NULL;

  TEN_STATUS_CODE status_code = ten_cmd_result_get_status_code(cmd_result);
  napi_status status =
      napi_create_uint32(env, (uint32_t)status_code, &js_status_code);
  ASSERT_IF_NAPI_FAIL(status == napi_ok, "Failed to create status_code.");

  status = napi_get_boolean(env, true, &js_create_shell_only_flag);
  ASSERT_IF_NAPI_FAIL(status == napi_ok, "Failed to create shell_only_flag.");

  napi_value argv[] = {js_status_code, js_create_shell_only_flag};

  napi_value js_cmd_result = ten_nodejs_create_new_js_object_and_wrap(
      env, js_cmd_result_constructor_ref, cmd_result_bridge,
      ten_nodejs_cmd_result_finalize,
      &cmd_result_bridge->msg.bridge.js_instance_ref, 2, argv);
  ASSERT_IF_NAPI_FAIL(js_cmd_result != NULL, "Failed to create JS Cmd object.");

  return js_cmd_result;
}

napi_value ten_nodejs_cmd_result_module_init(napi_env env, napi_value exports) {
  EXPORT_FUNC(env, exports, ten_nodejs_cmd_result_register_class);
  EXPORT_FUNC(env, exports, ten_nodejs_cmd_result_create);

  return exports;
}