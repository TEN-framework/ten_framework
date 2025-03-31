//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#include "include_internal/ten_runtime/addon/addon_loader/ten_env/on_xxx.h"

#include "include_internal/ten_runtime/addon_loader/addon_loader.h"
#include "include_internal/ten_runtime/ten_env/ten_env.h"

void ten_addon_loader_on_init_done(ten_env_t *self) {
  TEN_ASSERT(self, "Invalid argument.");
  TEN_ASSERT(ten_env_check_integrity(self, true), "Invalid use of ten_env %p.",
             self);

  ten_addon_loader_t *addon_loader = ten_env_get_attached_addon_loader(self);
  TEN_ASSERT(addon_loader, "Should not happen.");
  TEN_ASSERT(ten_addon_loader_check_integrity(addon_loader),
             "Should not happen.");

  if (addon_loader->on_init_done_cb) {
    addon_loader->on_init_done_cb(self, addon_loader->on_init_done_cb_data);
  }
}

bool ten_addon_loader_on_deinit_done(ten_env_t *self) {
  TEN_ASSERT(self, "Invalid argument.");
  TEN_ASSERT(
      ten_env_check_integrity(self,
                              // TODO(xilin): Remove this after move the deinit
                              // of the addon loader to the addon thread.
                              false),
      "Invalid use of ten_env %p.", self);

  ten_addon_loader_t *addon_loader = ten_env_get_attached_addon_loader(self);
  TEN_ASSERT(addon_loader, "Should not happen.");
  TEN_ASSERT(ten_addon_loader_check_integrity(addon_loader),
             "Should not happen.");

  if (addon_loader->on_deinit_done_cb) {
    addon_loader->on_deinit_done_cb(self, addon_loader->on_deinit_done_cb_data);
  }

  return true;
}
