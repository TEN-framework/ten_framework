//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#pragma once

#include "ten_runtime/ten_config.h"

#include <stdbool.h>

#include "ten_utils/lib/smart_ptr.h"

typedef struct ten_engine_t ten_engine_t;

TEN_RUNTIME_PRIVATE_API void ten_engine_append_to_in_msgs_queue(
    ten_engine_t *self, ten_shared_ptr_t *cmd);

TEN_RUNTIME_PRIVATE_API void ten_engine_handle_in_msgs_async(
    ten_engine_t *self);

TEN_RUNTIME_PRIVATE_API bool ten_engine_dispatch_msg(ten_engine_t *self,
                                                     ten_shared_ptr_t *msg);
