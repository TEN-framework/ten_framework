//
// Copyright © 2024 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#pragma once

#include "ten_runtime/test/ten_env_tester.h"

namespace ten {

class ten_env_tester_t {
 public:
  // @{
  ten_env_tester_t(const ten_env_tester_t &) = delete;
  ten_env_tester_t(ten_env_tester_t &&) = delete;
  ten_env_tester_t &operator=(const ten_env_tester_t &) = delete;
  ten_env_tester_t &operator=(const ten_env_tester_t &&) = delete;
  // @}};

  virtual bool on_start_done(error_t *err) {
    TEN_ASSERT(c_ten_env_tester, "Should not happen.");
    return ten_env_tester_on_start_done(
        c_ten_env_tester,
        err != nullptr ? err->get_internal_representation() : nullptr);
  }

  bool on_start_done() { return on_start_done(nullptr); }

 private:
  ::ten_env_tester_t *c_ten_env_tester;

}  // namespace ten
