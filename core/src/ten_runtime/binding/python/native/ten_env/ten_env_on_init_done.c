//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#include "include_internal/ten_runtime/binding/python/common/error.h"
#include "include_internal/ten_runtime/binding/python/ten_env/ten_env.h"
#include "include_internal/ten_runtime/metadata/metadata_info.h"
#include "include_internal/ten_runtime/ten_env/ten_env.h"
#include "ten_runtime/ten_env/internal/on_xxx_done.h"
#include "ten_runtime/ten_env_proxy/ten_env_proxy.h"
#include "ten_utils/macro/check.h"
#include "ten_utils/macro/mark.h"

static void ten_env_proxy_notify_on_init_done(ten_env_t *ten_env,
                                              TEN_UNUSED void *user_data) {
  TEN_ASSERT(
      ten_env &&
          ten_env_check_integrity(
              ten_env,
              ten_env->attach_to != TEN_ENV_ATTACH_TO_ADDON ? true : false),
      "Should not happen.");

  ten_error_t err;
  ten_error_init(&err);

  bool rc = ten_env_on_init_done(ten_env, &err);
  TEN_ASSERT(rc, "Should not happen.");

  ten_error_deinit(&err);
}

PyObject *ten_py_ten_env_on_init_done(PyObject *self, PyObject *args) {
  ten_py_ten_env_t *py_ten_env = (ten_py_ten_env_t *)self;
  TEN_ASSERT(py_ten_env && ten_py_ten_env_check_integrity(py_ten_env),
             "Invalid argument.");

  if (!py_ten_env->c_ten_env_proxy && !py_ten_env->c_ten_env) {
    return ten_py_raise_py_value_error_exception(
        "ten_env.on_init_done() failed because ten_env(_proxy) is invalid.");
  }

  ten_error_t err;
  ten_error_init(&err);

  bool rc = true;

  if (py_ten_env->c_ten_env_proxy) {
    rc = ten_env_proxy_notify_async(py_ten_env->c_ten_env_proxy,
                                    ten_env_proxy_notify_on_init_done, NULL,
                                    &err);
  } else {
    // TODO(Wei): This function is currently specifically designed for the addon
    // because the addon currently does not have a main thread, so it's unable
    // to use the ten_env_proxy mechanism to maintain thread safety. Once the
    // main thread for the addon is determined in the future, these hacks made
    // specifically for the addon can be completely removed, and comprehensive
    // thread safety mechanism can be implemented.
    TEN_ASSERT(py_ten_env->c_ten_env->attach_to == TEN_ENV_ATTACH_TO_ADDON,
               "Should not happen.");

    rc = ten_env_on_init_done(py_ten_env->c_ten_env, &err);
  }

  ten_error_deinit(&err);

  if (!rc) {
    return ten_py_raise_py_runtime_error_exception(
        "Failed to notify on init done.");
  }

  Py_RETURN_NONE;
}
