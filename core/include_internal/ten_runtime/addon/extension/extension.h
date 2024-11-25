//
// Copyright © 2024 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#pragma once

#include "ten_runtime/ten_config.h"

typedef struct ten_addon_store_t ten_addon_store_t;
typedef struct ten_addon_t ten_addon_t;

TEN_RUNTIME_PRIVATE_API ten_addon_store_t *ten_extension_get_global_store(void);

TEN_RUNTIME_API void ten_addon_register_extension_v2(const char *name,
                                                     const char *base_dir,
                                                     void *register_ctx,
                                                     ten_addon_t *addon);
