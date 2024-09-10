//
// This file is part of the TEN Framework project.
// See https://github.com/TEN-framework/ten_framework/LICENSE for license
// information.
//
#pragma once

#include "ten_runtime/ten_config.h"

#include "ten_utils/lib/smart_ptr.h"

typedef struct ten_msg_t ten_msg_t;
typedef struct ten_cmd_start_graph_t ten_cmd_start_graph_t;

TEN_RUNTIME_API ten_shared_ptr_t *ten_cmd_start_graph_create(void);
