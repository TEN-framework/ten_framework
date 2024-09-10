//
// This file is part of the TEN Framework project.
// See https://github.com/TEN-framework/ten_framework/LICENSE for license
// information.
//
#pragma once

#include <cassert>
#include <cstddef>
#include <cstdlib>
#include <memory>
#include <string>
#include <utility>

namespace ten {

inline void extension_t::proxy_on_cmd_internal(
    ten_extension_t *extension, ::ten_env_t *ten_env, ten_shared_ptr_t *cmd,
    cpp_extension_on_cmd_func_t on_cmd_func) {
  TEN_ASSERT(extension && ten_env, "Should not happen.");

  auto *cpp_extension =
      static_cast<extension_t *>(ten_binding_handle_get_me_in_target_lang(
          reinterpret_cast<ten_binding_handle_t *>(extension)));
  auto *cpp_ten_env =
      static_cast<ten_env_t *>(ten_binding_handle_get_me_in_target_lang(
          reinterpret_cast<ten_binding_handle_t *>(ten_env)));

  // Clone a C shared_ptr to be owned by the C++ instance.
  cmd = ten_shared_ptr_clone(cmd);

  cmd_t *cpp_cmd_ptr = nullptr;
  switch (ten_msg_get_type(cmd)) {
    case TEN_MSG_TYPE_CMD_START_GRAPH:
      cpp_cmd_ptr = new cmd_start_graph_t(cmd);
      break;

    case TEN_MSG_TYPE_CMD_TIMER:
      cpp_cmd_ptr = new cmd_timer_t(cmd);
      break;

    case TEN_MSG_TYPE_CMD_TIMEOUT:
      cpp_cmd_ptr = new cmd_timeout_t(cmd);
      break;

    case TEN_MSG_TYPE_CMD_STOP_GRAPH:
      cpp_cmd_ptr = new cmd_stop_graph_t(cmd);
      break;

    case TEN_MSG_TYPE_CMD_CLOSE_APP:
      cpp_cmd_ptr = new cmd_close_app_t(cmd);
      break;

    case TEN_MSG_TYPE_CMD:
      cpp_cmd_ptr = new cmd_t(cmd);
      break;

    default:
      TEN_ASSERT(0, "Should not happen.");
      break;
  }

  auto cpp_cmd_unique_ptr = std::unique_ptr<cmd_t>(cpp_cmd_ptr);

  cpp_extension->invoke_cpp_extension_on_cmd(
      *cpp_ten_env, std::move(cpp_cmd_unique_ptr), on_cmd_func);
}

}  // namespace ten
