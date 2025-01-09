//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#include "include_internal/ten_runtime/app/app.h"
#include "include_internal/ten_runtime/test/extension_tester.h"
#include "ten_runtime/ten_env/ten_env.h"
#include "ten_utils/container/list.h"
#include "ten_utils/lib/string.h"
#include "ten_utils/macro/mark.h"

static void test_app_on_configure(TEN_UNUSED ten_app_t *app,
                                  ten_env_t *ten_env) {
  // Since the tester will wait for the
  // `test_app_ten_env_proxy_create_completed` event after the app starts, it
  // means the tester is currently in a blocking state and not running, so
  // accessing the tester instance here is safe.
  ten_extension_tester_t *tester = app->user_data;
  TEN_ASSERT(tester && ten_extension_tester_check_integrity(tester, false),
             "Should not happen.");

  bool rc = false;

  if (!ten_string_is_empty(&tester->test_app_property_json)) {
    rc = ten_env_init_property_from_json(
        ten_env, ten_string_get_raw_str(&tester->test_app_property_json), NULL);
    TEN_ASSERT(rc, "Should not happen.");
  } else {
    // The default property.json content of the test app.
    rc = ten_env_init_property_from_json(ten_env,
                                         "{\
                                               \"_ten\": {\
                                                 \"log_level\": 2\
                                               }\
                                             }",
                                         NULL);
    TEN_ASSERT(rc, "Should not happen.");
  }

  rc = ten_env_on_configure_done(ten_env, NULL);
  TEN_ASSERT(rc, "Should not happen.");
}

// Store the tester as a property of the app so that the extensions within the
// app can access the tester's pointer through this app property.
static void store_tester_as_app_property(ten_extension_tester_t *tester,
                                         ten_env_t *ten_env) {
  TEN_ASSERT(tester && ten_extension_tester_check_integrity(tester, false),
             "Invalid argument.");
  TEN_ASSERT(ten_env && ten_env_check_integrity(ten_env, true),
             "Invalid argument.");

  // Store the tester as a property of the app so that the extensions within the
  // app can access the tester's pointer through this app property.
  ten_value_t *test_info_ptr_value =
      ten_value_create_ptr(tester, NULL, NULL, NULL);

  bool rc =
      ten_env_set_property(ten_env, "tester_ptr", test_info_ptr_value, NULL);
  TEN_ASSERT(rc, "Should not happen.");
}

static void create_ten_env_proxy_for_tester(ten_extension_tester_t *tester,
                                            ten_env_t *ten_env) {
  TEN_ASSERT(tester && ten_extension_tester_check_integrity(tester, false),
             "Invalid argument.");
  TEN_ASSERT(ten_env && ten_env_check_integrity(ten_env, true),
             "Invalid argument.");

  tester->test_app_ten_env_proxy = ten_env_proxy_create(ten_env, 1, NULL);
  TEN_ASSERT(tester->test_app_ten_env_proxy, "Should not happen.");

  ten_event_set(tester->test_app_ten_env_proxy_create_completed);
}

static void test_app_on_init(ten_app_t *app, ten_env_t *ten_env) {
  ten_extension_tester_t *tester = app->user_data;

  // Since the tester will wait for the
  // `test_app_ten_env_proxy_create_completed` event after the app starts,
  // using the tester here is thread-safe.
  TEN_ASSERT(tester && ten_extension_tester_check_integrity(tester, false),
             "Should not happen.");

  store_tester_as_app_property(tester, ten_env);
  create_ten_env_proxy_for_tester(tester, ten_env);

  ten_env_on_init_done(ten_env, NULL);
}

static void ten_extension_tester_on_test_app_deinit_task(void *self_,
                                                         TEN_UNUSED void *arg) {
  ten_extension_tester_t *tester = self_;
  TEN_ASSERT(tester && ten_extension_tester_check_integrity(tester, true),
             "Invalid argument.");

  // Since the tester uses the app's `ten_env_proxy` to interact with
  // `test_app`, it is necessary to release the app's `ten_env_proxy` within
  // the tester thread to ensure thread safety.
  //
  // Releasing the app's `ten_env_proxy` within the tester thread also
  // guarantees that `test_app` is still active at that time (As long as the
  // `ten_env_proxy` exists, the app will not be destroyed.), ensuring that all
  // operations using the app's `ten_env_proxy` before the releasing of
  // ten_env_proxy are valid.
  bool rc = ten_env_proxy_release(tester->test_app_ten_env_proxy, NULL);
  TEN_ASSERT(rc, "Should not happen.");

  tester->test_app_ten_env_proxy = NULL;

  ten_runloop_stop(tester->tester_runloop);
}

static void test_app_on_deinit(ten_app_t *app, ten_env_t *ten_env) {
  ten_extension_tester_t *tester = app->user_data;
  TEN_ASSERT(tester && ten_extension_tester_check_integrity(tester, false),
             "Should not happen.");

  int rc = ten_runloop_post_task_tail(
      tester->tester_runloop, ten_extension_tester_on_test_app_deinit_task,
      tester, NULL);
  TEN_ASSERT(!rc, "Should not happen.");

  ten_env_on_deinit_done(ten_env, NULL);
}

void *ten_builtin_test_app_thread_main(void *args) {
  ten_error_t err;
  ten_error_init(&err);

  ten_app_t *test_app = ten_app_create(test_app_on_configure, test_app_on_init,
                                       test_app_on_deinit, &err);
  TEN_ASSERT(test_app, "Failed to create app.");

  ten_extension_tester_t *tester = args;
  TEN_ASSERT(tester && ten_extension_tester_check_integrity(tester, false),
             "Invalid argument.");

  test_app->user_data = tester;

  ten_list_foreach (&tester->addon_base_dirs, iter) {
    ten_string_t *addon_base_dir = ten_str_listnode_get(iter.node);
    TEN_ASSERT(addon_base_dir, "Should not happen.");

    ten_app_add_ten_package_base_dir(test_app,
                                     ten_string_get_raw_str(addon_base_dir));
  }

  bool rc = ten_app_run(test_app, false, &err);
  TEN_ASSERT(rc, "Should not happen.");

  ten_app_destroy(test_app);

  return NULL;
}
