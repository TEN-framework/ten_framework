//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#include "ten_runtime/ten_config.h"

#include <Python.h>

#include "ten_runtime/app/app.h"

#define TEN_PY_APP_SIGNATURE 0x3227E7A2722B6BB2U

typedef struct ten_py_app_t {
  PyObject_HEAD
  ten_signature_t signature;
  ten_app_t *c_app;
} ten_py_app_t;

TEN_RUNTIME_PRIVATE_API bool ten_py_app_init_for_module(PyObject *module);
