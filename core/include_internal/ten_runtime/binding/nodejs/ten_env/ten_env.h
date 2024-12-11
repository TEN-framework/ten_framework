//
// Copyright © 2024 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#include "ten_runtime/ten_config.h"

#include <node_api.h>

#include "include_internal/ten_runtime/binding/nodejs/common/common.h"
#include "ten_runtime/ten_env/ten_env.h"
#include "ten_runtime/ten_env_proxy/ten_env_proxy.h"
#include "ten_utils/lib/signature.h"
#include "ten_utils/sanitizer/thread_check.h"

#define TEN_NODEJS_TEN_ENV_SIGNATURE 0x180B00AACEEF06E0U

typedef struct ten_nodejs_ten_env_t {
  ten_signature_t signature;
  ten_sanitizer_thread_check_t thread_check;

  ten_nodejs_bridge_t bridge;

  ten_env_t *c_ten_env;  // Point to the corresponding C ten_env.
  ten_env_proxy_t
      *c_ten_env_proxy;  // Point to the corresponding C ten_env_proxy if any.
} ten_nodejs_ten_env_t;

TEN_RUNTIME_API napi_value ten_nodejs_ten_env_module_init(napi_env env,
                                                          napi_value exports);

TEN_RUNTIME_API napi_value ten_nodejs_ten_env_create_new_js_object_and_wrap(
    napi_env env, ten_env_t *ten_env,
    ten_nodejs_ten_env_t **out_ten_env_bridge);

TEN_RUNTIME_PRIVATE_API bool ten_nodejs_ten_env_check_integrity(
    ten_nodejs_ten_env_t *self, bool check_thread);

TEN_RUNTIME_PRIVATE_API napi_value
ten_nodejs_ten_env_on_configure_done(napi_env env, napi_callback_info info);

TEN_RUNTIME_PRIVATE_API napi_value
ten_nodejs_ten_env_on_init_done(napi_env env, napi_callback_info info);

TEN_RUNTIME_PRIVATE_API napi_value
ten_nodejs_ten_env_on_deinit_done(napi_env env, napi_callback_info info);