
//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#include "include_internal/ten_runtime/app/msg_interface/start_graph.h"

#include <time.h>

#include "include_internal/ten_runtime/app/app.h"
#include "include_internal/ten_runtime/app/close.h"
#include "include_internal/ten_runtime/app/engine_interface.h"
#include "include_internal/ten_runtime/app/graph.h"
#include "include_internal/ten_runtime/app/metadata.h"
#include "include_internal/ten_runtime/app/msg_interface/common.h"
#include "include_internal/ten_runtime/app/predefined_graph.h"
#include "include_internal/ten_runtime/common/constant_str.h"
#include "include_internal/ten_runtime/connection/connection.h"
#include "include_internal/ten_runtime/connection/migration.h"
#include "include_internal/ten_runtime/engine/engine.h"
#include "include_internal/ten_runtime/engine/internal/migration.h"
#include "include_internal/ten_runtime/engine/msg_interface/common.h"
#include "include_internal/ten_runtime/msg/cmd_base/cmd/start_graph/cmd.h"
#include "include_internal/ten_runtime/msg/msg.h"
#include "include_internal/ten_runtime/protocol/protocol.h"
#include "ten_runtime/app/app.h"
#include "ten_runtime/common/status_code.h"
#include "ten_runtime/msg/cmd_result/cmd_result.h"
#include "ten_runtime/msg/msg.h"
#include "ten_utils/lib/smart_ptr.h"
#include "ten_utils/log/log.h"
#include "ten_utils/macro/check.h"

static bool ten_app_fill_start_graph_cmd_extensions_info_from_predefined_graph(
    ten_app_t *self, ten_shared_ptr_t *cmd, ten_error_t *err) {
  TEN_ASSERT(self && ten_app_check_integrity(self, true), "Should not happen.");
  TEN_ASSERT(cmd && ten_cmd_base_check_integrity(cmd), "Should not happen.");

  ten_string_t *predefined_graph_name =
      ten_cmd_start_graph_get_predefined_graph_name(cmd);
  if (ten_string_is_empty(predefined_graph_name)) {
    return true;
  }

  ten_list_t *extensions_info = ten_cmd_start_graph_get_extensions_info(cmd);
  ten_list_t *extension_groups_info =
      ten_cmd_start_graph_get_extension_groups_info(cmd);

  bool res = ten_app_get_predefined_graph_extensions_and_groups_info_by_name(
      self, ten_string_get_raw_str(predefined_graph_name), extensions_info,
      extension_groups_info, err);
  TEN_ASSERT(res, "should not happen.");
  if (!res) {
    return false;
  }

  return true;
}

static bool ten_app_check_start_graph_cmd_from_connection(
    ten_app_t *self, ten_connection_t *connection, ten_shared_ptr_t *cmd,
    ten_error_t *err) {
  TEN_ASSERT(self && ten_app_check_integrity(self, true), "Invalid argument.");
  TEN_ASSERT(cmd && ten_cmd_base_check_integrity(cmd), "Invalid argument.");
  TEN_ASSERT(ten_msg_get_type(cmd) == TEN_MSG_TYPE_CMD_START_GRAPH,
             "Invalid argument.");
  TEN_ASSERT(err && ten_error_check_integrity(err), "Invalid argument.");

  bool rc = ten_app_check_start_graph_cmd(self, cmd, err);
  if (!rc && connection) {
    ten_shared_ptr_t *ret_cmd =
        ten_cmd_result_create_from_cmd(TEN_STATUS_CODE_ERROR, cmd);
    ten_msg_set_property(ret_cmd, TEN_STR_DETAIL,
                         ten_value_create_string(ten_error_errmsg(err)), NULL);
    ten_msg_clear_and_set_dest_from_msg_src(ret_cmd, cmd);
    ten_connection_send_msg(connection, ret_cmd);
    ten_shared_ptr_destroy(ret_cmd);
  }

  return rc;
}

bool ten_app_handle_start_graph_cmd(ten_app_t *self,
                                    ten_connection_t *connection,
                                    ten_shared_ptr_t *cmd, ten_error_t *err) {
  TEN_ASSERT(self && ten_app_check_integrity(self, true), "Invalid argument.");
  TEN_ASSERT(cmd && ten_cmd_base_check_integrity(cmd), "Invalid argument.");
  TEN_ASSERT(ten_msg_get_type(cmd) == TEN_MSG_TYPE_CMD_START_GRAPH,
             "Invalid argument.");
  TEN_ASSERT(ten_msg_get_dest_cnt(cmd) == 1, "Invalid argument.");
  TEN_ASSERT(
      connection ? ten_app_has_orphan_connection(self, connection) : true,
      "Invalid argument.");

  // If the start_graph command is aimed at initting from a predefined graph, we
  // should append the extension info list of the predefined graph to the cmd.
  if (!ten_app_fill_start_graph_cmd_extensions_info_from_predefined_graph(
          self, cmd, err)) {
    return false;
  }

  ten_engine_t *engine =
      ten_app_get_engine_based_on_dest_graph_id_from_msg(self, cmd);
  if (engine == NULL) {
    // The graph should be only checked once.
    if (!ten_app_check_start_graph_cmd_from_connection(self, connection, cmd,
                                                       err)) {
      TEN_LOGE("[%s] Failed to check start_graph cmd, %s",
               ten_app_get_uri(self), ten_error_errmsg(err));
      return false;
    }

    // The engine does not exist, create one, and send 'cmd' to the newly
    // created engine.
    engine = ten_app_create_engine(self, cmd);
  } else {
    // The engine of the graph has already been created, this condition would be
    // hit in polygon graph.
  }

  // No matter the situation, it is up to the engine to handle the connect
  // command and return the corresponding cmd result.
  ten_app_do_connection_migration_or_push_to_engine_queue(connection, engine,
                                                          cmd);

  return true;
}
