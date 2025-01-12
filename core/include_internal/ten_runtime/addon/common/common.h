//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#pragma once

#include "ten_runtime/ten_config.h"

#include "ten_runtime/addon/addon.h"

typedef struct ten_addon_host_t ten_addon_host_t;

TEN_RUNTIME_PRIVATE_API ten_addon_host_t *ten_addon_store_find_by_type(
    TEN_ADDON_TYPE addon_type, const char *addon_name);

TEN_RUNTIME_PRIVATE_API int ten_addon_store_lock_by_type(
    TEN_ADDON_TYPE addon_type);

TEN_RUNTIME_PRIVATE_API int ten_addon_store_unlock_by_type(
    TEN_ADDON_TYPE addon_type);

TEN_RUNTIME_PRIVATE_API int ten_addon_store_lock_all_type(void);

TEN_RUNTIME_PRIVATE_API int ten_addon_store_unlock_all_type(void);
