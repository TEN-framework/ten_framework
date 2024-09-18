//
// Copyright © 2024 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#pragma once

#include "ten_runtime/ten_config.h"

#include "ten_utils/lib/smart_ptr.h"

typedef struct ten_msg_t ten_msg_t;
typedef struct ten_cmd_start_graph_t ten_cmd_start_graph_t;

TEN_RUNTIME_API ten_shared_ptr_t *ten_cmd_start_graph_create(void);
