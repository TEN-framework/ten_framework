//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#pragma once

#include "include_internal/ten_runtime/binding/cpp/detail/addon_loader.h"
#include "include_internal/ten_runtime/ten_env/metadata.h"
#include "ten_runtime/binding/cpp/detail/addon.h"
#include "ten_runtime/binding/cpp/detail/extension.h"
#include "ten_runtime/binding/cpp/detail/ten_env.h"

namespace ten {

inline bool ten_env_t::on_create_instance_done(void *instance, void *context,
                                               error_t *err) {
  void *c_instance = nullptr;

  auto *cpp_context = reinterpret_cast<addon_context_t *>(context);
  TEN_ASSERT(cpp_context, "Invalid argument.");

  switch (cpp_context->task) {
    case ADDON_TASK_CREATE_EXTENSION:
      c_instance = extension_internal_accessor_t::get_c_extension(
          static_cast<extension_t *>(instance));
      break;
    case ADDON_TASK_CREATE_ADDON_LOADER:
      c_instance = addon_loader_internal_accessor_t::get_c_addon_loader(
          static_cast<addon_loader_t *>(instance));
      break;
    default:
      TEN_ASSERT(0, "Should not happen.");
      break;
  }

  TEN_ASSERT(c_instance, "Should not happen.");

  bool rc = ten_env_on_create_instance_done(
      c_ten_env, c_instance, cpp_context->c_context,
      err != nullptr ? err->get_c_error() : nullptr);

  delete cpp_context;

  return rc;
}

inline bool ten_env_t::init_manifest_from_json(const char *json, error_t *err) {
  TEN_ASSERT(c_ten_env, "Should not happen.");

  if (json == nullptr) {
    TEN_ASSERT(0, "Invalid argument.");
    return false;
  }

  return ten_env_init_manifest_from_json(
      c_ten_env, json, err != nullptr ? err->get_c_error() : nullptr);
}

}  // namespace ten
