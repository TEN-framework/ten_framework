//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#pragma once

#include "ten_runtime/ten_config.h"

#include "ten_runtime/ten_env/ten_env.h"

TEN_RUNTIME_PRIVATE_API void ten_addon_on_init_done(ten_env_t *self);

TEN_RUNTIME_PRIVATE_API void ten_addon_on_deinit_done(ten_env_t *self);

TEN_RUNTIME_PRIVATE_API void ten_addon_on_create_instance_done(ten_env_t *self,
                                                               void *instance,
                                                               void *context);

TEN_RUNTIME_PRIVATE_API void ten_addon_on_destroy_instance_done(ten_env_t *self,
                                                                void *context);
