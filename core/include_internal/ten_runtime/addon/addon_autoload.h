//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#pragma once

#include "ten_runtime/ten_config.h"

#include <stdbool.h>

#include "include_internal/ten_runtime/addon/addon.h"
#include "ten_utils/lib/error.h"

typedef struct ten_app_t ten_app_t;

typedef void (*ten_addon_register_func_t)(void *register_ctx);

TEN_RUNTIME_PRIVATE_API bool
ten_addon_load_all_protocols_and_addon_loaders_from_app_base_dir(
    const char *app_base_dir, ten_error_t *err);

TEN_RUNTIME_PRIVATE_API bool
ten_addon_try_load_specific_addon_using_native_addon_loader(
    const char *app_base_dir, TEN_ADDON_TYPE addon_type,
    const char *addon_name);

TEN_RUNTIME_PRIVATE_API bool
ten_addon_try_load_specific_addon_using_all_addon_loaders(
    TEN_ADDON_TYPE addon_type, const char *addon_name);
