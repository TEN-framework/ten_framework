//
// Copyright © 2024 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#include "include_internal/ten_runtime/binding/nodejs/msg/msg.h"

#include "include_internal/ten_runtime/msg/msg.h"
#include "ten_utils/lib/json.h"
#include "ten_utils/lib/signature.h"
#include "ten_utils/lib/string.h"
#include "ten_utils/macro/memory.h"

void ten_nodejs_msg_init_from_c_msg(ten_nodejs_msg_t *self,
                                    ten_shared_ptr_t *msg) {
  TEN_ASSERT(self, "Should not happen.");
  TEN_ASSERT(msg && ten_msg_check_integrity(msg), "Should not happen.");

  ten_signature_set(&self->signature, TEN_NODEJS_MSG_SIGNATURE);

  self->msg = ten_shared_ptr_clone(msg);
}

void ten_nodejs_msg_deinit(ten_nodejs_msg_t *self) {
  TEN_ASSERT(self, "Should not happen.");

  if (self->msg) {
    ten_shared_ptr_destroy(self->msg);
    self->msg = NULL;
  }

  ten_signature_set(&self->signature, 0);
}

static napi_value ten_nodejs_msg_get_name(napi_env env,
                                          napi_callback_info info) {
  const size_t argc = 1;
  napi_value args[argc];  // this
  if (!ten_nodejs_get_js_func_args(env, info, args, argc)) {
    napi_fatal_error(NULL, NAPI_AUTO_LENGTH,
                     "Incorrect number of parameters passed.",
                     NAPI_AUTO_LENGTH);
    TEN_ASSERT(0, "Should not happen.");
    return js_undefined(env);
  }

  ten_nodejs_msg_t *msg_bridge = NULL;
  napi_status status = napi_unwrap(env, args[0], (void **)&msg_bridge);
  RETURN_UNDEFINED_IF_NAPI_FAIL(status == napi_ok && msg_bridge != NULL,
                                "Failed to get msg bridge: %d", status);
  TEN_ASSERT(msg_bridge, "Should not happen.");

  ten_shared_ptr_t *msg = msg_bridge->msg;
  TEN_ASSERT(msg && ten_msg_check_integrity(msg), "Should not happen.");

  const char *name = ten_msg_get_name(msg);
  TEN_ASSERT(name, "Should not happen.");

  napi_value js_msg_name = NULL;
  status = napi_create_string_utf8(env, name, NAPI_AUTO_LENGTH, &js_msg_name);
  RETURN_UNDEFINED_IF_NAPI_FAIL(status == napi_ok && js_msg_name != NULL,
                                "Failed to create JS string: %d", status);

  return js_msg_name;
}

static napi_value ten_nodejs_msg_set_property_from_json(
    napi_env env, napi_callback_info info) {
  const size_t argc = 3;
  napi_value args[argc];  // this, path, json_str
  if (!ten_nodejs_get_js_func_args(env, info, args, argc)) {
    napi_fatal_error(NULL, NAPI_AUTO_LENGTH,
                     "Incorrect number of parameters passed.",
                     NAPI_AUTO_LENGTH);
    TEN_ASSERT(0, "Should not happen.");
    return js_undefined(env);
  }

  ten_nodejs_msg_t *msg_bridge = NULL;
  napi_status status = napi_unwrap(env, args[0], (void **)&msg_bridge);
  RETURN_UNDEFINED_IF_NAPI_FAIL(status == napi_ok && msg_bridge != NULL,
                                "Failed to get msg bridge: %d", status);
  TEN_ASSERT(msg_bridge, "Should not happen.");

  ten_error_t err;
  ten_error_init(&err);

  ten_string_t path;
  ten_string_init(&path);

  ten_json_t *c_json = NULL;

  bool rc = ten_nodejs_get_str_from_js(env, args[1], &path);
  RETURN_UNDEFINED_IF_NAPI_FAIL(rc, "Failed to get property path", NULL);

  ten_string_t json_str;
  ten_string_init(&json_str);

  rc = ten_nodejs_get_str_from_js(env, args[2], &json_str);
  RETURN_UNDEFINED_IF_NAPI_FAIL(rc, "Failed to get property value JSON", NULL);

  c_json = ten_json_from_string(ten_string_get_raw_str(&json_str), &err);
  if (!c_json) {
    ten_string_t code_str;
    ten_string_init_formatted(&code_str, "%d", ten_error_errno(&err));

    napi_throw_error(env, ten_string_get_raw_str(&code_str),
                     ten_error_errmsg(&err));

    ten_string_deinit(&code_str);

    goto done;
  }

  ten_value_t *value = ten_value_from_json(c_json);

  rc = ten_msg_set_property(msg_bridge->msg, ten_string_get_raw_str(&path),
                            value, &err);
  if (!rc) {
    ten_string_t code_str;
    ten_string_init_formatted(&code_str, "%d", ten_error_errno(&err));

    napi_throw_error(env, ten_string_get_raw_str(&code_str),
                     ten_error_errmsg(&err));

    ten_string_deinit(&code_str);
  }

done:
  ten_string_deinit(&path);
  ten_string_deinit(&json_str);
  ten_error_deinit(&err);
  if (c_json) {
    ten_json_destroy(c_json);
  }

  return js_undefined(env);
}

static napi_value ten_nodejs_msg_get_property_to_json(napi_env env,
                                                      napi_callback_info info) {
  const size_t argc = 2;
  napi_value args[argc];  // this, path
  if (!ten_nodejs_get_js_func_args(env, info, args, argc)) {
    napi_fatal_error(NULL, NAPI_AUTO_LENGTH,
                     "Incorrect number of parameters passed.",
                     NAPI_AUTO_LENGTH);
    TEN_ASSERT(0, "Should not happen.");
    return js_undefined(env);
  }

  ten_nodejs_msg_t *msg_bridge = NULL;
  napi_status status = napi_unwrap(env, args[0], (void **)&msg_bridge);
  RETURN_UNDEFINED_IF_NAPI_FAIL(status == napi_ok && msg_bridge != NULL,
                                "Failed to get msg bridge: %d", status);
  TEN_ASSERT(msg_bridge, "Should not happen.");

  ten_error_t err;
  ten_error_init(&err);

  ten_string_t path;
  ten_string_init(&path);

  napi_value js_res = NULL;

  bool rc = ten_nodejs_get_str_from_js(env, args[1], &path);
  RETURN_UNDEFINED_IF_NAPI_FAIL(rc, "Failed to get property path", NULL);

  ten_value_t *c_value = ten_msg_peek_property(
      msg_bridge->msg, ten_string_get_raw_str(&path), &err);
  if (!c_value) {
    ten_string_t code_str;
    ten_string_init_formatted(&code_str, "%d", ten_error_errno(&err));

    napi_throw_error(env, ten_string_get_raw_str(&code_str),
                     ten_error_errmsg(&err));

    ten_string_deinit(&code_str);
    goto done;
  }

  ten_json_t *c_json = ten_value_to_json(c_value);
  TEN_ASSERT(c_json, "Should not happen.");

  bool must_free = false;
  const char *json_str = ten_json_to_string(c_json, NULL, &must_free);
  TEN_ASSERT(json_str, "Should not happen.");

  ten_json_destroy(c_json);

  status = napi_create_string_utf8(env, json_str, NAPI_AUTO_LENGTH, &js_res);
  RETURN_UNDEFINED_IF_NAPI_FAIL(status == napi_ok && js_res != NULL,
                                "Failed to create JS string: %d", status);

  if (must_free) {
    TEN_FREE(json_str);
  }

done:
  ten_string_deinit(&path);
  ten_error_deinit(&err);

  if (!js_res) {
    js_res = js_undefined(env);
  }

  return js_res;
}

napi_value ten_nodejs_msg_module_init(napi_env env, napi_value exports) {
  EXPORT_FUNC(env, exports, ten_nodejs_msg_get_name);
  EXPORT_FUNC(env, exports, ten_nodejs_msg_set_property_from_json);
  EXPORT_FUNC(env, exports, ten_nodejs_msg_get_property_to_json);

  return exports;
}