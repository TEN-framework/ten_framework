//
// Copyright © 2024 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#pragma once

#include "ten_runtime/test/extension_tester.h"
#include "ten_utils/macro/check.h"

namespace ten {

class extension_tester_t {
 public:
  virtual ~extension_tester_t() {
    TEN_ASSERT(c_extension_tester, "Should not happen.");
    ten_extension_tester_destroy(c_extension_tester);
  }

  // @{
  extension_tester_t(const extension_tester_t &) = delete;
  extension_tester_t(extension_tester_t &&) = delete;
  extension_tester_t &operator=(const extension_tester_t &) = delete;
  extension_tester_t &operator=(const extension_tester_t &&) = delete;
  // @}

 protected:
  explicit extension_tester_t()
      : c_extension_tester(::ten_extension_tester_create(
            reinterpret_cast<ten_extension_tester_on_start_func_t>(
                &proxy_on_start),
            reinterpret_cast<ten_extension_tester_on_cmd_func_t>(
                &proxy_on_cmd))) {
    TEN_ASSERT(c_extension_tester, "Should not happen.");

    ten_binding_handle_set_me_in_target_lang(
        reinterpret_cast<ten_binding_handle_t *>(c_extension_tester),
        static_cast<void *>(this));
  }

  virtual void on_start(ten_env_tester_t &cpp_ten_env_tester) {
    cpp_ten_env_tester.on_start_done();
  }

  virtual void on_cmd(ten_env_tester_t &cpp_ten_env_tester,
                      std::unique_ptr<cmd_t> cmd) {}

 private:
  void invoke_cpp_extension_tester_on_start(
      ten_env_tester_t &cpp_ten_env_tester) {
    on_start(cpp_ten_env_tester);
  }

  static void proxy_on_start(ten_extension_tester_t *tester,
                             ::ten_env_tester_t *cpp_ten_env_tester) {
    TEN_ASSERT(tester && cpp_ten_env_tester, "Should not happen.");

    auto *cpp_extension_tester = static_cast<extension_tester_t *>(
        ten_binding_handle_get_me_in_target_lang(
            reinterpret_cast<ten_binding_handle_t *>(tester)));
    auto *cpp_ten_env_tester = static_cast<ten_env_tester_t *>(
        ten_binding_handle_get_me_in_target_lang(
            reinterpret_cast<ten_binding_handle_t *>(cpp_ten_env_tester)));

    cpp_extension_tester->invoke_cpp_extension_tester_on_start(
        *cpp_ten_env_tester);
  }

  void invoke_cpp_extension_on_cmd(ten_env_tester_t &cpp_ten_env_tester,
                                   std::unique_ptr<cmd_t> cmd) {
    on_cmd(cpp_ten_env_tester, std::move(cmd));
  }

  static void proxy_on_cmd(ten_extension_tester_t *tester,
                           ::ten_env_tester_t *ten_env, ten_shared_ptr_t *cmd) {
    TEN_ASSERT(tester && ten_env && cmd, "Should not happen.");

    auto *cpp_extension_tester = static_cast<extension_tester_t *>(
        ten_binding_handle_get_me_in_target_lang(
            reinterpret_cast<ten_binding_handle_t *>(tester)));
    auto *cpp_ten_env_tester = static_cast<ten_env_tester_t *>(
        ten_binding_handle_get_me_in_target_lang(
            reinterpret_cast<ten_binding_handle_t *>(ten_env)));

    // Clone a C shared_ptr to be owned by the C++ instance.
    cmd = ten_shared_ptr_clone(cmd);

    cmd_t *cpp_cmd_ptr = new cmd_t(cmd);
    auto cpp_cmd_unique_ptr = std::unique_ptr<cmd_t>(cpp_cmd_ptr);

    cpp_extension_tester->invoke_cpp_extension_on_cmd(
        *cpp_ten_env_tester, std::move(cpp_cmd_unique_ptr));
  }

  ::ten_extension_tester_t *c_extension_tester;
};

}  // namespace ten
