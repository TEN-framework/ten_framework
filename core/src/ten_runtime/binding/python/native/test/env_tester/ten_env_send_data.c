//
// Copyright © 2024 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#include "include_internal/ten_runtime/binding/python/common/error.h"
#include "include_internal/ten_runtime/binding/python/msg/data.h"
#include "include_internal/ten_runtime/binding/python/msg/msg.h"
#include "include_internal/ten_runtime/binding/python/test/env_tester.h"
#include "ten_runtime/test/env_tester.h"
#include "ten_utils/macro/check.h"

PyObject *ten_py_ten_env_tester_send_data(PyObject *self, PyObject *args) {
  ten_py_ten_env_tester_t *py_ten_env_tester = (ten_py_ten_env_tester_t *)self;
  TEN_ASSERT(py_ten_env_tester &&
                 ten_py_ten_env_tester_check_integrity(py_ten_env_tester),
             "Invalid argument.");

  if (PyTuple_GET_SIZE(args) != 1) {
    return ten_py_raise_py_value_error_exception(
        "Invalid argument count when ten_env_tester.send_data.");
  }

  bool success = true;

  ten_error_t err;
  ten_error_init(&err);

  ten_py_data_t *py_data = NULL;

  if (!PyArg_ParseTuple(args, "O", ten_py_data_py_type(), &py_data)) {
    success = false;
    ten_py_raise_py_type_error_exception(
        "Invalid argument type when send data.");
    goto done;
  }

  ten_env_tester_send_data(py_ten_env_tester->c_ten_env_tester,
                           py_data->msg.c_msg);

  // Destroy the C message from the Python message as the ownership has been
  // transferred to the notify_info.
  ten_py_msg_destroy_c_msg(&py_data->msg);

done:
  ten_error_deinit(&err);

  if (success) {
    Py_RETURN_NONE;
  } else {
    return NULL;
  }
}
