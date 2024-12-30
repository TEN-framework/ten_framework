//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#pragma once

#include "ten_runtime/ten_config.h"

#include "include_internal/ten_runtime/addon/addon.h"
#include "ten_runtime/addon/addon_manager.h"
#include "ten_utils/container/list.h"
#include "ten_utils/lib/mutex.h"
#include "ten_utils/lib/string.h"

typedef struct ten_app_t ten_app_t;

typedef struct ten_addon_registration_t {
  TEN_ADDON_TYPE addon_type;
  ten_string_t addon_name;
  ten_addon_registration_func_t func;
} ten_addon_registration_t;

typedef struct ten_addon_register_ctx_t {
  ten_app_t *app;
} ten_addon_register_ctx_t;

typedef struct ten_addon_manager_t {
  // Define a registry map to store addon registration functions.
  // The key is the addon name (string), and the value is a function that takes
  // a register_ctx.
  ten_list_t registry;  // ten_addon_registration_t*
  ten_mutex_t *mutex;
} ten_addon_manager_t;

TEN_RUNTIME_API void ten_addon_manager_register_all_addons(
    ten_addon_manager_t *self, void *register_ctx);

TEN_RUNTIME_API bool ten_addon_manager_register_specific_addon(
    ten_addon_manager_t *self, TEN_ADDON_TYPE addon_type,
    const char *addon_name, void *register_ctx);

TEN_RUNTIME_PRIVATE_API ten_addon_register_ctx_t *ten_addon_register_ctx_create(
    void);

TEN_RUNTIME_PRIVATE_API void ten_addon_register_ctx_destroy(
    ten_addon_register_ctx_t *self);
