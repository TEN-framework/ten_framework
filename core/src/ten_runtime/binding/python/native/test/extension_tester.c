//
// Copyright © 2024 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#include "include_internal/ten_runtime/binding/python/test/extension_tester.h"

#include <stdbool.h>

#include "include_internal/ten_runtime/binding/python/common/error.h"
#include "ten_utils/macro/check.h"
#include "ten_utils/macro/mark.h"

static bool ten_py_extension_tester_check_integrity(
    ten_py_extension_tester_t *self) {
  TEN_ASSERT(self, "Should not happen.");

  if (ten_signature_get(&self->signature) !=
      (ten_signature_t)TEN_PY_EXTENSION_TESTER_SIGNATURE) {
    return false;
  }

  return true;
}

static ten_py_extension_tester_t *ten_py_extension_tester_create_internal(
    PyTypeObject *py_type) {
  if (!py_type) {
    py_type = ten_py_extension_tester_py_type();
  }

  ten_py_extension_tester_t *py_extension_tester =
      (ten_py_extension_tester_t *)py_type->tp_alloc(py_type, 0);

  ten_signature_set(&py_extension_tester->signature,
                    TEN_PY_EXTENSION_TESTER_SIGNATURE);
  py_extension_tester->c_extension_tester = NULL;

  return py_extension_tester;
}

static ten_py_extension_tester_t *ten_py_extension_tester_init(
    ten_py_extension_tester_t *py_extension_tester, TEN_UNUSED PyObject *args,
    TEN_UNUSED PyObject *kw) {
  TEN_ASSERT(py_extension_tester &&
                 ten_py_extension_tester_check_integrity(
                     (ten_py_extension_tester_t *)py_extension_tester),
             "Invalid argument.");

  // =-=-=
  py_extension_tester->c_extension_tester =
      ten_extension_tester_create(NULL, NULL);

  return py_extension_tester;
}

PyObject *ten_py_extension_tester_create(PyTypeObject *type,
                                         TEN_UNUSED PyObject *args,
                                         TEN_UNUSED PyObject *kwds) {
  ten_py_extension_tester_t *py_extension_tester =
      ten_py_extension_tester_create_internal(type);
  return (PyObject *)ten_py_extension_tester_init(py_extension_tester, args,
                                                  kwds);
}

void ten_py_extension_tester_destroy(PyObject *self) {
  ten_py_extension_tester_t *py_extension_tester =
      (ten_py_extension_tester_t *)self;
  TEN_ASSERT(py_extension_tester &&
                 ten_py_extension_tester_check_integrity(
                     (ten_py_extension_tester_t *)py_extension_tester),
             "Invalid argument.");

  ten_extension_tester_destroy(py_extension_tester->c_extension_tester);
  Py_TYPE(self)->tp_free(self);
}

static PyObject *ten_py_extension_tester_add_addon_base_dir(PyObject *self,
                                                            PyObject *args) {
  ten_py_extension_tester_t *py_extension_tester =
      (ten_py_extension_tester_t *)self;
  TEN_ASSERT(py_extension_tester &&
                 ten_py_extension_tester_check_integrity(py_extension_tester),
             "Invalid argument.");

  if (PyTuple_GET_SIZE(args) != 1) {
    return ten_py_raise_py_value_error_exception(
        "Invalid argument count when extension_tester.add_addon_base_dir.");
  }

  const char *addon_base_dir = NULL;
  if (!PyArg_ParseTuple(args, "s", &addon_base_dir)) {
    return ten_py_raise_py_value_error_exception(
        "Failed to parse arguments when extension_tester.add_addon_base_dir.");
  }

  ten_extension_tester_add_addon_base_dir(
      py_extension_tester->c_extension_tester, addon_base_dir);

  Py_RETURN_NONE;
}

PyTypeObject *ten_py_extension_tester_py_type(void) {
  static PyMethodDef py_methods[] = {
      {"add_addon_base_dir", ten_py_extension_tester_add_addon_base_dir,
       METH_VARARGS, NULL},
      {NULL, NULL, 0, NULL},
  };

  static PyTypeObject py_type = {
      PyVarObject_HEAD_INIT(NULL, 0).tp_name =
          "libten_runtime_python._ExtensionTester",
      .tp_doc = PyDoc_STR("_ExtensionTester"),
      .tp_basicsize = sizeof(ten_py_extension_tester_t),
      .tp_itemsize = 0,
      .tp_flags = Py_TPFLAGS_DEFAULT | Py_TPFLAGS_BASETYPE,
      .tp_new = ten_py_extension_tester_create,
      .tp_init = NULL,
      .tp_dealloc = ten_py_extension_tester_destroy,
      .tp_getset = NULL,
      .tp_methods = py_methods,
  };

  return &py_type;
}

bool ten_py_extension_tester_init_for_module(PyObject *module) {
  PyTypeObject *py_type = ten_py_extension_tester_py_type();
  if (PyType_Ready(py_type) < 0) {
    ten_py_raise_py_system_error_exception(
        "Python VideoFrame class is not ready.");

    TEN_ASSERT(0, "Should not happen.");
    return false;
  }

  if (PyModule_AddObjectRef(module, "_VideoFrame", (PyObject *)py_type) < 0) {
    ten_py_raise_py_import_error_exception(
        "Failed to add Python type to module.");

    TEN_ASSERT(0, "Should not happen.");
    return false;
  }
  return true;
}
