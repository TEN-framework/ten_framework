//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#include "include_internal/ten_runtime/extension_group/extension_group_info/json.h"

#include "include_internal/ten_runtime/common/constant_str.h"
#include "include_internal/ten_runtime/extension_group/extension_group_info/extension_group_info.h"
#include "ten_utils/container/list.h"
#include "ten_utils/lib/json.h"
#include "ten_utils/lib/string.h"
#include "ten_utils/macro/check.h"

// NOLINTNEXTLINE(misc-no-recursion)
ten_json_t *ten_extension_group_info_to_json(ten_extension_group_info_t *self) {
  TEN_ASSERT(self, "Invalid argument.");
  TEN_ASSERT(ten_extension_group_info_check_integrity(self),
             "Should not happen.");

  ten_json_t *info = ten_json_create_object();
  TEN_ASSERT(info, "Should not happen.");

  ten_json_t *type = ten_json_create_string(TEN_STR_EXTENSION_GROUP);
  TEN_ASSERT(type, "Should not happen.");
  ten_json_object_set_new(info, TEN_STR_TYPE, type);

  ten_json_t *name = ten_json_create_string(
      ten_string_get_raw_str(&self->loc.extension_group_name));
  TEN_ASSERT(name, "Should not happen.");
  ten_json_object_set_new(info, TEN_STR_NAME, name);

  ten_json_t *addon = ten_json_create_string(
      ten_string_get_raw_str(&self->extension_group_addon_name));
  TEN_ASSERT(addon, "Should not happen.");
  ten_json_object_set_new(info, TEN_STR_ADDON, addon);

  ten_json_t *app_uri =
      ten_json_create_string(ten_string_get_raw_str(&self->loc.app_uri));
  TEN_ASSERT(app_uri, "Should not happen.");
  ten_json_object_set_new(info, TEN_STR_APP, app_uri);

  if (self->property) {
    ten_json_object_set_new(info, TEN_STR_PROPERTY,
                            ten_value_to_json(self->property));
  }

  return info;
}

ten_shared_ptr_t *ten_extension_group_info_from_json(
    ten_json_t *json, ten_list_t *extension_groups_info, ten_error_t *err) {
  TEN_ASSERT(json && ten_json_check_integrity(json), "Should not happen.");
  TEN_ASSERT(
      extension_groups_info && ten_list_check_integrity(extension_groups_info),
      "Should not happen.");

  const char *type = ten_json_object_peek_string(json, TEN_STR_TYPE);
  if (!type || !strlen(type) || strcmp(type, TEN_STR_EXTENSION_GROUP) != 0) {
    TEN_ASSERT(0, "Invalid extension group info.");
    return NULL;
  }

  const char *app_uri = ten_json_object_peek_string(json, TEN_STR_APP);
  const char *graph_id = ten_json_object_peek_string(json, TEN_STR_GRAPH);
  const char *addon_name = ten_json_object_peek_string(json, TEN_STR_ADDON);
  const char *instance_name = ten_json_object_peek_string(json, TEN_STR_NAME);

  ten_shared_ptr_t *self = get_extension_group_info_in_extension_groups_info(
      extension_groups_info, app_uri, graph_id, addon_name, instance_name, NULL,
      err);
  if (!self) {
    return NULL;
  }

  ten_extension_group_info_t *extension_group_info =
      ten_shared_ptr_get_data(self);
  TEN_ASSERT(ten_extension_group_info_check_integrity(extension_group_info),
             "Should not happen.");

  // Parse 'prop'
  ten_json_t *props_json = ten_json_object_peek(json, TEN_STR_PROPERTY);
  if (props_json) {
    if (!ten_json_is_object(props_json)) {
      // Indicates an error.
      TEN_ASSERT(0,
                 "Failed to parse 'prop' in 'start_graph' command, it's not an "
                 "object.");
      return NULL;
    }

    ten_value_object_merge_with_json(extension_group_info->property,
                                     props_json);
  }

  return self;
}
