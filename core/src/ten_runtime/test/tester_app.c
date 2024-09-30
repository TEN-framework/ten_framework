//
// Copyright © 2024 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#include "include_internal/ten_runtime/app/app.h"
#include "include_internal/ten_runtime/test/extension_tester.h"
#include "ten_utils/macro/mark.h"

static void tester_app_on_configure(TEN_UNUSED ten_app_t *app,
                                    ten_env_t *ten_env) {
  bool rc = ten_env_init_property_from_json(ten_env,
                                            "{\
                                               \"_ten\": {\
                                                 \"log_level\": 2\
                                               }\
                                             }",
                                            NULL);
  TEN_ASSERT(rc, "Should not happen.");

  rc = ten_env_on_configure_done(ten_env, NULL);
  TEN_ASSERT(rc, "Should not happen.");
}

static void tester_app_on_init(ten_app_t *app, ten_env_t *ten_env) {
  ten_extension_tester_t *tester = app->user_data;
  TEN_ASSERT(tester, "Should not happen.");

  ten_value_t *test_info_ptr_value =
      ten_value_create_ptr(tester, NULL, NULL, NULL);

  bool rc = ten_env_set_property(ten_env, "test_extension_test_info_ptr",
                                 test_info_ptr_value, NULL);
  TEN_ASSERT(rc, "Should not happen.");

  tester->tester_app_ten_env_proxy = ten_env_proxy_create(ten_env, 1, NULL);
  TEN_ASSERT(tester->tester_app_ten_env_proxy, "Should not happen.");

  ten_event_set(tester->tester_app_ten_env_proxy_create_completed);

  ten_env_on_init_done(ten_env, NULL);
}

static void ten_extension_tester_on_tester_app_deinit(void *self_,
                                                      TEN_UNUSED void *arg) {
  ten_extension_tester_t *tester = self_;
  TEN_ASSERT(tester, "Invalid argument.");

  bool rc = ten_env_proxy_release(tester->tester_app_ten_env_proxy, NULL);
  TEN_ASSERT(rc, "Should not happen.");

  tester->tester_app_ten_env_proxy = NULL;
}

static void tester_app_on_deinit(ten_app_t *app, ten_env_t *ten_env) {
  ten_extension_tester_t *tester = app->user_data;
  TEN_ASSERT(tester, "Should not happen.");

  ten_runloop_post_task_tail(tester->tester_runloop,
                             ten_extension_tester_on_tester_app_deinit, tester,
                             NULL);

  ten_env_on_deinit_done(ten_env, NULL);
}

void *ten_builtin_tester_app_thread_main(void *args) {
  ten_error_t err;
  ten_error_init(&err);

  ten_extension_tester_t *tester = args;

  ten_app_t *test_app = ten_app_create(
      tester_app_on_configure, tester_app_on_init, tester_app_on_deinit, &err);
  TEN_ASSERT(test_app, "Failed to create app.");

  test_app->user_data = tester;

  bool rc = ten_app_run(test_app, false, &err);
  TEN_ASSERT(rc, "Should not happen.");

  ten_app_destroy(test_app);

  return NULL;
}
