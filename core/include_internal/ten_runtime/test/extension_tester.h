//
// Copyright © 2024 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#include "ten_runtime/ten_config.h"

#include "include_internal/ten_utils/io/runloop.h"
#include "ten_runtime/ten_env_proxy/ten_env_proxy.h"
#include "ten_utils/lib/signature.h"
#include "ten_utils/lib/smart_ptr.h"

#define TEN_EXTENSION_TESTER_SIGNATURE 0x2343E0B8559B7147U

typedef struct ten_extension_tester_t ten_extension_tester_t;
typedef struct ten_env_tester_t ten_env_tester_t;

typedef void (*ten_extension_tester_on_start_func_t)(
    ten_extension_tester_t *self, ten_env_tester_t *ten_env);

typedef void (*ten_extension_tester_on_cmd_func_t)(ten_extension_tester_t *self,
                                                   ten_env_tester_t *ten_env,
                                                   ten_shared_ptr_t *cmd);

struct ten_extension_tester_t {
  ten_signature_t signature;
  ten_sanitizer_thread_check_t thread_check;

  ten_thread_t *tester_app_thread;
  ten_env_proxy_t *tester_app_ten_env_proxy;
  ten_event_t *tester_app_ten_env_proxy_create_completed;

  ten_env_proxy_t *tester_extension_ten_env_proxy;
  ten_event_t *tester_extension_ten_env_proxy_create_completed;

  ten_string_t target_extension_addon_name;

  ten_extension_tester_on_start_func_t on_start;
  ten_extension_tester_on_cmd_func_t on_cmd;

  ten_env_tester_t *ten_env_tester;
  ten_runloop_t *tester_runloop;

  void *user_data;
};

TEN_RUNTIME_PRIVATE_API bool ten_extension_tester_check_integrity(
    ten_extension_tester_t *self, bool check_thread);

TEN_RUNTIME_API ten_extension_tester_t *ten_extension_tester_create(
    ten_extension_tester_on_start_func_t on_start,
    ten_extension_tester_on_cmd_func_t on_cmd);

TEN_RUNTIME_API void ten_extension_tester_destroy(ten_extension_tester_t *self);

TEN_RUNTIME_API void ten_extension_tester_run(ten_extension_tester_t *self);

TEN_RUNTIME_API void ten_extension_tester_add_addon(
    ten_extension_tester_t *self, const char *addon_name);

TEN_RUNTIME_PRIVATE_API void test_app_ten_env_send_cmd(ten_env_t *ten_env,
                                                       void *user_data);
