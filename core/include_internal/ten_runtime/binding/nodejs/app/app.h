//
// Copyright © 2024 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#include "ten_runtime/ten_config.h"

#include <node_api.h>

#include "include_internal/ten_runtime/binding/nodejs/common/tsfn.h"
#include "ten_runtime/app/app.h"
#include "ten_utils/lib/ref.h"
#include "ten_utils/lib/signature.h"
#include "ten_utils/sanitizer/thread_check.h"

#define TEN_NODEJS_APP_SIGNATURE 0x133066C5116F5EA4U

typedef struct ten_nodejs_app_t {
  ten_signature_t signature;
  ten_sanitizer_thread_check_t thread_check;

  ten_ref_t ref;
  ten_app_t *c_app;     // The corresponding C app.
  napi_ref js_app_ref;  // The JS app object.
  napi_ref js_rte_ref;  // companion RTE of the JS app

  // @{
  // The following function is the Javascript functions corresponding to the
  // app's interface API.
  ten_nodejs_tsfn_t *js_on_init;
  ten_nodejs_tsfn_t *js_on_close;
  // @}
} ten_nodejs_app_t;

TEN_RUNTIME_API napi_value ten_nodejs_app_module_init(napi_env env,
                                                      napi_value exports);
