//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#pragma once

#include "ten_runtime/binding/cpp/detail/addon.h"

namespace ten {

class addon_loader_addon_t : public addon_t {
 private:
  void invoke_cpp_addon_on_create_instance(ten_env_t &ten_env, const char *name,
                                           void *context) override {
    try {
      auto *cpp_context = new addon_context_t();
      cpp_context->task = ADDON_TASK_CREATE_ADDON_LOADER;
      cpp_context->c_context = context;

      on_create_instance(ten_env, name, cpp_context);
    } catch (...) {
      TEN_LOGW("Caught a exception '%s' in addon on_create_instance(%s).",
               curr_exception_type_name().c_str(), name);
    }
  }
};

}  // namespace ten
