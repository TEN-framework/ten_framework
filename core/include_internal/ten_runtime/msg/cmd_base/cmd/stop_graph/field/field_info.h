//
// This file is part of the TEN Framework project.
// See https://github.com/TEN-framework/ten_framework/LICENSE for license
// information.
//
#pragma once

#include "ten_runtime/ten_config.h"

#include <stddef.h>

#include "include_internal/ten_runtime/common/constant_str.h"
#include "include_internal/ten_runtime/msg/cmd_base/cmd/cmd.h"
#include "include_internal/ten_runtime/msg/cmd_base/cmd/stop_graph/field/field.h"
#include "include_internal/ten_runtime/msg/cmd_base/cmd/stop_graph/field/graph_name.h"
#include "include_internal/ten_runtime/msg/field/field_info.h"

#ifdef __cplusplus
  #error \
      "This file contains C99 array designated initializer, and Visual Studio C++ compiler can only support up to C89 by default, so we enable this checking to prevent any wrong inclusion of this file."
#endif

static const ten_msg_field_info_t ten_cmd_stop_graph_fields_info[] = {
    [TEN_CMD_STOP_GRAPH_FIELD_CMDHDR] =
        {
            .field_name = NULL,
            .put_field_to_json = ten_raw_cmd_put_field_to_json,
            .get_field_from_json = ten_raw_cmd_get_field_from_json,
            .copy_field = NULL,
        },
    [TEN_CMD_STOP_GRAPH_FIELD_GRAPH_NAME] =
        {
            .field_name = TEN_STR_GRAPH_NAME,
            .put_field_to_json = ten_cmd_stop_graph_put_graph_name_to_json,
            .get_field_from_json = ten_cmd_stop_graph_get_graph_name_from_json,
            .copy_field = ten_cmd_stop_graph_copy_graph_name,
        },
    [TEN_CMD_STOP_GRAPH_FIELD_LAST] = {0},
};

static const size_t ten_cmd_stop_graph_fields_info_size =
    sizeof(ten_cmd_stop_graph_fields_info) /
    sizeof(ten_cmd_stop_graph_fields_info[0]);
