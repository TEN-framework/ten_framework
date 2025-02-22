//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#include "ten_runtime/msg/cmd_result/cmd_result.h"

#include "include_internal/ten_runtime/app/app.h"
#include "include_internal/ten_runtime/common/constant_str.h"
#include "include_internal/ten_runtime/connection/connection.h"
#include "include_internal/ten_runtime/engine/engine.h"
#include "include_internal/ten_runtime/engine/internal/close.h"
#include "include_internal/ten_runtime/engine/internal/extension_interface.h"
#include "include_internal/ten_runtime/engine/internal/remote_interface.h"
#include "include_internal/ten_runtime/engine/msg_interface/common.h"
#include "include_internal/ten_runtime/engine/msg_interface/start_graph.h"
#include "include_internal/ten_runtime/msg/cmd_base/cmd_base.h"
#include "include_internal/ten_runtime/msg/cmd_base/cmd_result/cmd.h"
#include "include_internal/ten_runtime/msg/msg.h"
#include "include_internal/ten_runtime/path/path.h"
#include "include_internal/ten_runtime/path/path_table.h"
#include "include_internal/ten_runtime/remote/remote.h"
#include "include_internal/ten_utils/log/log.h"
#include "ten_runtime/common/status_code.h"
#include "ten_runtime/msg/msg.h"
#include "ten_utils/lib/smart_ptr.h"
#include "ten_utils/lib/string.h"
#include "ten_utils/macro/check.h"
#include "ten_utils/value/value_get.h"
#include "ten_utils/value/value_is.h"

static bool ten_engine_close_duplicated_remote_or_upgrade_it_to_normal(
    ten_engine_t *self, ten_shared_ptr_t *cmd_result, ten_error_t *err) {
  TEN_ASSERT(self && ten_engine_check_integrity(self, true) && cmd_result &&
                 ten_cmd_base_check_integrity(cmd_result),
             "Should not happen.");

  ten_remote_t *weak_remote =
      ten_engine_find_weak_remote(self, ten_msg_get_src_app_uri(cmd_result));
  if (weak_remote == NULL) {
    // Only if the 'start_graph' flow involves a connection, we need to handle
    // situations relevant to that connection.
    return true;
  }

  TEN_ASSERT(ten_remote_check_integrity(weak_remote, true),
             "Invalid use of remote %p.", weak_remote);

  ten_string_t detail_str;
  ten_string_init(&detail_str);

  ten_value_t *detail_value = ten_msg_peek_property(cmd_result, "detail", NULL);
  if (!detail_value || !ten_value_is_string(detail_value)) {
    TEN_ASSERT(0, "Should not happen.");
  }

  bool rc = ten_value_to_string(detail_value, &detail_str, err);
  TEN_ASSERT(rc, "Should not happen.");

  if (ten_string_is_equal_c_str(&detail_str, TEN_STR_DUPLICATE)) {
    TEN_LOGW("Receives a 'duplicate' result from %s",
             ten_string_get_raw_str(&weak_remote->uri));

    // This is a duplicated channel, closing it now.
    ten_connection_t *connection = weak_remote->connection;
    TEN_ASSERT(connection && ten_connection_check_integrity(connection, true),
               "Should not happen.");

    connection->duplicate = true;

    ten_connection_close(connection);
  } else {
    // The 'start_graph' is done, change this remote from weak-type to
    // normal-type.
    ten_engine_upgrade_weak_remote_to_normal_remote(self, weak_remote);
  }

  ten_string_deinit(&detail_str);

  return true;
}

static ten_shared_ptr_t *ten_engine_process_out_path(
    ten_engine_t *self, ten_shared_ptr_t *cmd_result, ten_error_t *err) {
  TEN_ASSERT(self && ten_engine_check_integrity(self, true),
             "Should not happen.");
  TEN_ASSERT(cmd_result &&
                 ten_msg_get_type(cmd_result) == TEN_MSG_TYPE_CMD_RESULT &&
                 ten_msg_get_dest_cnt(cmd_result) == 1,
             "Should not happen.");

  ten_path_t *out_path =
      ten_path_table_set_result(self->path_table, TEN_PATH_OUT, cmd_result);
  if (!out_path) {
    TEN_LOGD("[%s] IN path is missing, discard cmd result.",
             ten_engine_get_id(self, true));
    return NULL;
  }

  TEN_ASSERT(ten_path_check_integrity(out_path, true), "Should not happen.");

  bool is_final_result = ten_cmd_result_is_final(cmd_result, err);
  // Currently, all `cmd_results` processed by the engine will _not_ be
  // streaming `cmd_results`.
  TEN_ASSERT(is_final_result, "Should not happen.");

  // Check whether _all_ cmd_results related to this command have been received
  // to determine whether to proceed with the next steps of the cmd flow.
  cmd_result = ten_path_table_determine_actual_cmd_result(
      self->path_table, TEN_PATH_OUT, out_path, is_final_result);
  if (!cmd_result) {
    return NULL;
  }

  return cmd_result;
}

static bool ten_engine_handle_cmd_result_for_cmd_start_graph(
    ten_engine_t *self, ten_shared_ptr_t *cmd_result, ten_error_t *err) {
  TEN_ASSERT(self && ten_engine_check_integrity(self, true),
             "Invalid argument.");
  TEN_ASSERT(cmd_result &&
                 ten_msg_get_type(cmd_result) == TEN_MSG_TYPE_CMD_RESULT &&
                 ten_msg_get_dest_cnt(cmd_result) == 1,
             "Should not happen.");
  TEN_ASSERT(ten_c_string_is_equal(
                 ten_app_get_uri(self->app),
                 ten_string_get_raw_str(
                     &ten_msg_get_first_dest_loc(cmd_result)->app_uri)),
             "Should not happen.");

  if (ten_cmd_result_get_status_code(cmd_result) == TEN_STATUS_CODE_OK) {
    bool rc = ten_engine_close_duplicated_remote_or_upgrade_it_to_normal(
        self, cmd_result, err);
    TEN_ASSERT(rc, "Should not happen.");
  }

  cmd_result = ten_engine_process_out_path(self, cmd_result, err);
  if (!cmd_result) {
    TEN_LOGD(
        "The 'start_graph' flow is not completed, skip the cmd_result now.");
    return true;
  }

  // The processing of the 'start_graph' flows are completed.

  // If a cmd_result is received during the start_graph flow, it indicates a
  // multiple-app start_graph scenario. Before starting to connect to more apps
  // in the whole start_graph process,
  // `original_start_graph_cmd_of_enabling_engine` must be set. Otherwise, after
  // the entire process is completed, there will be no way to determine where to
  // send the `cmd_result` of the `start_graph` command.
  ten_shared_ptr_t *original_start_graph_cmd =
      self->original_start_graph_cmd_of_enabling_engine;
  TEN_ASSERT(
      original_start_graph_cmd &&
          ten_cmd_base_check_integrity(original_start_graph_cmd),
      "The engine should be started because of receiving a 'start_graph' "
      "command.");

  if (ten_cmd_result_get_status_code(cmd_result) == TEN_STATUS_CODE_OK) {
    // All the later connection stages are completed, enable the extension
    // system now.
    ten_engine_enable_extension_system(self, err);
  } else if (ten_cmd_result_get_status_code(cmd_result) ==
             TEN_STATUS_CODE_ERROR) {
    ten_value_t *err_msg_value =
        ten_msg_peek_property(cmd_result, TEN_STR_DETAIL, NULL);
    if (err_msg_value) {
      TEN_ASSERT(ten_value_is_string(err_msg_value), "Should not happen.");

      ten_engine_return_error_for_cmd_start_graph(
          self, original_start_graph_cmd,
          ten_value_peek_raw_str(err_msg_value, err));
    } else {
      ten_engine_return_error_for_cmd_start_graph(
          self, original_start_graph_cmd, "Failed to start engine in app [%s].",
          ten_msg_get_src_app_uri(cmd_result));
    }

    ten_shared_ptr_destroy(original_start_graph_cmd);
    self->original_start_graph_cmd_of_enabling_engine = NULL;
  } else {
    TEN_ASSERT(0, "Should not happen.");
  }

  ten_shared_ptr_destroy(cmd_result);

  return true;
}

void ten_engine_handle_cmd_result(ten_engine_t *self,
                                  ten_shared_ptr_t *cmd_result,
                                  ten_error_t *err) {
  TEN_ASSERT(self && ten_engine_check_integrity(self, true),
             "Should not happen.");
  TEN_ASSERT(cmd_result &&
                 ten_msg_get_type(cmd_result) == TEN_MSG_TYPE_CMD_RESULT &&
                 ten_msg_get_dest_cnt(cmd_result) == 1,
             "Should not happen.");

  switch (ten_cmd_result_get_original_cmd_type(cmd_result)) {
    case TEN_MSG_TYPE_CMD_START_GRAPH: {
      bool rc = ten_engine_handle_cmd_result_for_cmd_start_graph(
          self, cmd_result, err);
      TEN_ASSERT(rc, "Should not happen.");
      break;
    }

    case TEN_MSG_TYPE_INVALID:
      TEN_ASSERT(0, "Should not happen.");
      break;

    default:
      TEN_ASSERT(0, "Handle more original command type.");
      break;
  }
}
