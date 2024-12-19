//
// Copyright © 2024 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#include "include_internal/ten_runtime/binding/nodejs/msg/msg.h"

#include "include_internal/ten_runtime/msg/msg.h"
#include "ten_utils/lib/signature.h"

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

napi_value ten_nodejs_msg_module_init(napi_env env, napi_value exports) {
  EXPORT_FUNC(env, exports, ten_nodejs_msg_get_name);

  return exports;
}