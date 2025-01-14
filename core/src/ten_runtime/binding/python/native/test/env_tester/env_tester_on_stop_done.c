//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#include "include_internal/ten_runtime/binding/python/common/common.h"
#include "include_internal/ten_runtime/binding/python/common/error.h"
#include "include_internal/ten_runtime/binding/python/test/env_tester.h"
#include "ten_runtime/test/env_tester.h"
#include "ten_runtime/test/env_tester_proxy.h"
#include "ten_utils/lib/error.h"
#include "ten_utils/macro/check.h"
#include "ten_utils/macro/mark.h"

static void ten_py_ten_env_tester_on_stop_done_proxy_notify(
    ten_env_tester_t *ten_env_tester, void *user_data) {
  TEN_ASSERT(
      ten_env_tester && ten_env_tester_check_integrity(ten_env_tester, true),
      "Invalid argument.");

  ten_py_ten_env_tester_t *py_ten_env_tester = user_data;
  TEN_ASSERT(py_ten_env_tester, "Should not happen.");

  // Notify the Python side to do the cleanup.
  //
  // About to call the Python function, so it's necessary to ensure that the
  // GIL has been acquired.
  PyGILState_STATE prev_state = ten_py_gil_state_ensure_internal();

  PyObject *py_res = PyObject_CallMethod(
      py_ten_env_tester->actual_py_ten_env_tester, "_on_release", NULL);
  Py_XDECREF(py_res);

  bool err_occurred = ten_py_check_and_clear_py_error();
  TEN_ASSERT(!err_occurred, "Should not happen.");

  // Release the ten_env_tester_proxy.
  ten_env_tester_proxy_release(py_ten_env_tester->c_ten_env_tester_proxy, NULL);
  py_ten_env_tester->c_ten_env_tester_proxy = NULL;

  ten_py_gil_state_release_internal(prev_state);

  ten_env_tester_on_stop_done(ten_env_tester, NULL);
}

PyObject *ten_py_ten_env_tester_on_stop_done(PyObject *self,
                                             TEN_UNUSED PyObject *args) {
  ten_py_ten_env_tester_t *py_ten_env_tester = (ten_py_ten_env_tester_t *)self;
  TEN_ASSERT(py_ten_env_tester &&
                 ten_py_ten_env_tester_check_integrity(py_ten_env_tester),
             "Invalid argument.");

  if (!py_ten_env_tester->c_ten_env_tester_proxy) {
    return ten_py_raise_py_value_error_exception(
        "ten_env_tester.on_stop_done() failed because ten_env_tester_proxy is "
        "invalid.");
  }

  ten_error_t err;
  ten_error_init(&err);

  bool rc = ten_env_tester_proxy_notify(
      py_ten_env_tester->c_ten_env_tester_proxy,
      ten_py_ten_env_tester_on_stop_done_proxy_notify, py_ten_env_tester, &err);
  TEN_ASSERT(rc, "Should not happen.");

  ten_error_deinit(&err);

  Py_RETURN_NONE;
}
