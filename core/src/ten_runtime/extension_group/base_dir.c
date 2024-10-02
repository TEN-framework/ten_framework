//
// Copyright © 2024 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#include "include_internal/ten_runtime/common/base_dir.h"

#include "include_internal/ten_runtime/common/constant_str.h"
#include "include_internal/ten_runtime/extension_group/extension_group.h"
#include "ten_utils/lib/path.h"
#include "ten_utils/lib/string.h"

static ten_string_t *ten_extension_group_find_base_dir(const char *name) {
  ten_string_t *extension_group_base_dir = NULL;

  ten_string_t *path =
      ten_path_get_module_path((void *)ten_extension_group_find_base_dir);
  if (!path) {
    TEN_LOGW("Could not get extension_group base dir from module path.");
    return NULL;
  }

  ten_find_base_dir(path, TEN_STR_EXTENSION_GROUP, name,
                    &extension_group_base_dir);
  ten_string_destroy(path);

  if (!extension_group_base_dir) {
    TEN_LOGW("Could not get extension_group base dir from module path.");
    return NULL;
  }

  ten_path_to_system_flavor(extension_group_base_dir);
  return extension_group_base_dir;
}

ten_string_t *ten_extension_group_get_base_dir(ten_extension_group_t *self) {
  TEN_ASSERT(self && ten_extension_group_check_integrity(self, true),
             "Invalid argument.");
  return &self->base_dir;
}

void ten_extension_group_find_and_set_base_dir(ten_extension_group_t *self) {
  TEN_ASSERT(self && ten_extension_group_check_integrity(self, true),
             "Should not happen.");

  ten_string_t *extension_group_base_dir =
      ten_extension_group_find_base_dir(ten_string_get_raw_str(&self->name));
  if (!extension_group_base_dir) {
    TEN_LOGW("Failed to determine extension_group base directory.");
    return;
  }

  ten_string_copy(&self->base_dir, extension_group_base_dir);
  ten_string_destroy(extension_group_base_dir);
}
