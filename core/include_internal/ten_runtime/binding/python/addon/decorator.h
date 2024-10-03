//
// Copyright © 2024 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#pragma once

#include "ten_runtime/ten_config.h"

#include <stdbool.h>

#include "include_internal/ten_runtime/binding/python/common/python_stuff.h"
#include "ten_utils/lib/string.h"

typedef struct ten_py_decorator_register_addon_t {
  PyObject_HEAD
  ten_string_t addon_name;
  ten_string_t base_dir;
} ten_py_decorator_register_addon_t;

TEN_RUNTIME_PRIVATE_API bool
ten_py_decorator_register_addon_as_extension_init_for_module(PyObject *module);

TEN_RUNTIME_PRIVATE_API bool
ten_py_decorator_register_addon_as_extension_group_init_for_module(
    PyObject *module);
