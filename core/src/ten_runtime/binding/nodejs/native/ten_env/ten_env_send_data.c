//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#include "include_internal/ten_runtime/binding/nodejs/msg/data.h"
#include "include_internal/ten_runtime/binding/nodejs/ten_env/ten_env.h"
#include "ten_utils/macro/memory.h"

typedef struct ten_env_notify_send_data_ctx_t {
  ten_shared_ptr_t *c_data;
  ten_nodejs_tsfn_t *js_cb;
} ten_env_notify_send_data_ctx_t;

typedef struct ten_nodejs_send_data_callback_call_ctx_t {
  ten_nodejs_tsfn_t *js_cb;
  ten_error_t *error;
} ten_nodejs_send_data_callback_call_ctx_t;

static ten_env_notify_send_data_ctx_t *ten_env_notify_send_data_ctx_create(
    ten_shared_ptr_t *c_data, ten_nodejs_tsfn_t *js_cb) {
  ten_env_notify_send_data_ctx_t *ctx =
      TEN_MALLOC(sizeof(ten_env_notify_send_data_ctx_t));
  TEN_ASSERT(ctx, "Failed to allocate memory.");

  ctx->c_data = c_data;
  ctx->js_cb = js_cb;

  return ctx;
}

static void ten_env_notify_send_data_ctx_destroy(
    ten_env_notify_send_data_ctx_t *ctx) {
  TEN_ASSERT(ctx, "Invalid argument.");

  if (ctx->c_data) {
    ten_shared_ptr_destroy(ctx->c_data);
    ctx->c_data = NULL;
  }

  ctx->js_cb = NULL;

  TEN_FREE(ctx);
}

static ten_nodejs_send_data_callback_call_ctx_t *
ten_nodejs_send_data_callback_call_ctx_create(ten_nodejs_tsfn_t *js_cb,
                                              ten_error_t *error) {
  ten_nodejs_send_data_callback_call_ctx_t *ctx =
      TEN_MALLOC(sizeof(ten_nodejs_send_data_callback_call_ctx_t));
  TEN_ASSERT(ctx, "Failed to allocate memory.");

  ctx->js_cb = js_cb;
  ctx->error = error;

  return ctx;
}

static void ten_nodejs_send_data_callback_call_ctx_destroy(
    ten_nodejs_send_data_callback_call_ctx_t *ctx) {
  TEN_ASSERT(ctx, "Invalid argument.");

  if (ctx->error) {
    ten_error_destroy(ctx->error);
    ctx->error = NULL;
  }

  TEN_FREE(ctx);
}

static void tsfn_proxy_send_data_callback(napi_env env, napi_value js_cb,
                                          void *context, void *data) {
  ten_nodejs_send_data_callback_call_ctx_t *ctx = data;
  TEN_ASSERT(ctx, "Should not happen.");

  napi_value js_error = NULL;

  if (ctx->error) {
    js_error = ten_nodejs_create_error(env, ctx->error);
    ASSERT_IF_NAPI_FAIL(js_error, "Failed to create JS error", NULL);
  } else {
    js_error = js_undefined(env);
  }

  napi_value argv[] = {js_error};
  napi_status status =
      napi_call_function(env, js_undefined(env), js_cb, 1, argv, NULL);
  ASSERT_IF_NAPI_FAIL(status == napi_ok, "Failed to call JS callback", NULL);

  ten_nodejs_tsfn_release(ctx->js_cb);

  ten_nodejs_send_data_callback_call_ctx_destroy(ctx);
}

static void proxy_send_data_callback(ten_env_t *ten_env, ten_shared_ptr_t *msg,
                                     void *user_data, ten_error_t *err) {
  TEN_ASSERT(ten_env && ten_env_check_integrity(ten_env, true),
             "Should not happen.");

  ten_env_notify_send_data_ctx_t *ctx = user_data;
  TEN_ASSERT(ctx, "Should not happen.");

  ten_nodejs_tsfn_t *js_cb = ctx->js_cb;
  TEN_ASSERT(js_cb && ten_nodejs_tsfn_check_integrity(js_cb, false),
             "Should not happen.");

  ten_error_t *cloned_error = NULL;
  if (err) {
    cloned_error = ten_error_create();
    ten_error_copy(cloned_error, err);
  }

  ten_nodejs_send_data_callback_call_ctx_t *call_info =
      ten_nodejs_send_data_callback_call_ctx_create(js_cb, cloned_error);
  TEN_ASSERT(call_info, "Failed to create callback call info.");

  bool rc = ten_nodejs_tsfn_invoke(ctx->js_cb, call_info);
  TEN_ASSERT(rc, "Should not happen.");

  ten_env_notify_send_data_ctx_destroy(ctx);
}

static void ten_env_proxy_notify_send_data(ten_env_t *ten_env,
                                           void *user_data) {
  TEN_ASSERT(user_data, "Invalid argument.");
  TEN_ASSERT(ten_env && ten_env_check_integrity(ten_env, true),
             "Should not happen.");

  ten_env_notify_send_data_ctx_t *ctx = user_data;
  TEN_ASSERT(ctx, "Should not happen.");

  ten_error_t err;
  ten_error_init(&err);

  bool rc = ten_env_send_data(ten_env, ctx->c_data, proxy_send_data_callback,
                              ctx, &err);
  if (!rc) {
    proxy_send_data_callback(ten_env, NULL, ctx, &err);
  }

  ten_error_deinit(&err);
}

napi_value ten_nodejs_ten_env_send_data(napi_env env, napi_callback_info info) {
  const size_t argc = 3;
  napi_value args[argc];  // this, data, callback
  if (!ten_nodejs_get_js_func_args(env, info, args, argc)) {
    napi_fatal_error(NULL, NAPI_AUTO_LENGTH,
                     "Incorrect number of parameters passed.",
                     NAPI_AUTO_LENGTH);
    TEN_ASSERT(0, "Should not happen.");
    return NULL;
  }

  ten_nodejs_ten_env_t *ten_env_bridge = NULL;
  napi_status status = napi_unwrap(env, args[0], (void **)&ten_env_bridge);
  RETURN_UNDEFINED_IF_NAPI_FAIL(status == napi_ok && ten_env_bridge != NULL,
                                "Failed to unwrap TenEnv object");

  TEN_ASSERT(ten_env_bridge &&
                 ten_nodejs_ten_env_check_integrity(ten_env_bridge, true),
             "Should not happen.");

  ten_nodejs_data_t *data_bridge = NULL;
  status = napi_unwrap(env, args[1], (void **)&data_bridge);
  RETURN_UNDEFINED_IF_NAPI_FAIL(status == napi_ok && data_bridge != NULL,
                                "Failed to unwrap Data object");

  ten_nodejs_tsfn_t *cb_tsfn =
      ten_nodejs_tsfn_create(env, "[TSFN] TenEnv::send_data callback", args[2],
                             tsfn_proxy_send_data_callback);
  RETURN_UNDEFINED_IF_NAPI_FAIL(cb_tsfn, "Failed to create TSFN");

  ten_error_t err;
  ten_error_init(&err);

  ten_env_notify_send_data_ctx_t *notify_info =
      ten_env_notify_send_data_ctx_create(
          ten_shared_ptr_clone(data_bridge->msg.msg), cb_tsfn);
  TEN_ASSERT(notify_info, "Failed to create notify info.");

  bool rc = ten_env_proxy_notify(ten_env_bridge->c_ten_env_proxy,
                                 ten_env_proxy_notify_send_data, notify_info,
                                 false, &err);
  if (!rc) {
    ten_string_t code_str;
    ten_string_init_formatted(&code_str, "%d", ten_error_errno(&err));

    status = napi_throw_error(env, ten_string_get_raw_str(&code_str),
                              ten_error_errmsg(&err));
    RETURN_UNDEFINED_IF_NAPI_FAIL(status == napi_ok, "Failed to throw error");

    ten_string_deinit(&code_str);

    // The JS callback will not be called, so release the TSFN here.
    ten_nodejs_tsfn_release(cb_tsfn);

    ten_env_notify_send_data_ctx_destroy(notify_info);
  }

  ten_error_deinit(&err);

  return js_undefined(env);
}
