//
// This file is part of the TEN Framework project.
// See https://github.com/TEN-framework/ten_framework/LICENSE for license
// information.
//
#include "include_internal/ten_runtime/extension/msg_dest_info/json.h"

#include "include_internal/ten_runtime/common/constant_str.h"
#include "include_internal/ten_runtime/common/loc.h"
#include "include_internal/ten_runtime/extension/extension.h"
#include "include_internal/ten_runtime/extension/extension_info/extension_info.h"
#include "include_internal/ten_runtime/extension/extension_info/json.h"
#include "include_internal/ten_runtime/extension/msg_dest_info/msg_dest_info.h"
#include "include_internal/ten_runtime/msg_conversion/msg_conversion/msg_and_result_conversion_operation.h"
#include "include_internal/ten_runtime/msg_conversion/msg_conversion/msg_conversion.h"
#include "ten_utils/macro/check.h"
#include "ten_utils/container/list.h"
#include "ten_utils/container/list_node.h"
#include "ten_utils/lib/error.h"
#include "ten_utils/lib/json.h"
#include "ten_utils/lib/smart_ptr.h"
#include "ten_utils/lib/string.h"

ten_json_t *ten_msg_dest_static_info_to_json(
    ten_msg_dest_static_info_t *self, ten_extension_info_t *src_extension_info,
    ten_error_t *err) {
  TEN_ASSERT(self && ten_msg_dest_static_info_check_integrity(self),
             "Should not happen.");

  ten_json_t *json = ten_json_create_object();
  TEN_ASSERT(json, "Should not happen.");
  ten_json_object_set_new(
      json, TEN_STR_NAME,
      ten_json_create_string(ten_string_get_raw_str(&self->msg_name)));

  ten_json_t *dests_json = ten_json_create_array();
  TEN_ASSERT(dests_json, "Should not happen.");
  ten_json_object_set_new(json, TEN_STR_DEST, dests_json);

  ten_list_foreach (&self->dest, iter) {
    ten_weak_ptr_t *dest = ten_smart_ptr_listnode_get(iter.node);
    TEN_ASSERT(dest, "Invalid argument.");

    ten_extension_info_t *extension_info = ten_smart_ptr_get_data(dest);

    ten_json_t *dest_json = ten_json_create_object();
    TEN_ASSERT(dest_json, "Should not happen.");

    ten_json_object_set_new(dest_json, TEN_STR_APP,
                            ten_json_create_string(ten_string_get_raw_str(
                                &extension_info->loc.app_uri)));

    ten_json_object_set_new(dest_json, TEN_STR_GRAPH,
                            ten_json_create_string(ten_string_get_raw_str(
                                &extension_info->loc.graph_name)));

    ten_json_t *extension_group_json = ten_json_create_string(
        ten_string_get_raw_str(&extension_info->loc.extension_group_name));
    TEN_ASSERT(extension_group_json, "Should not happen.");
    ten_json_object_set_new(dest_json, TEN_STR_EXTENSION_GROUP,
                            extension_group_json);

    ten_json_t *extension_json = ten_json_create_string(
        ten_string_get_raw_str(&extension_info->loc.extension_name));
    TEN_ASSERT(extension_json, "Should not happen.");
    ten_json_object_set_new(dest_json, TEN_STR_EXTENSION, extension_json);

    ten_list_foreach (&extension_info->msg_conversions, msg_conversion_iter) {
      ten_msg_conversion_t *msg_conversion =
          ten_ptr_listnode_get(msg_conversion_iter.node);
      TEN_ASSERT(
          msg_conversion && ten_msg_conversion_check_integrity(msg_conversion),
          "Should not happen.");

      if (ten_loc_is_equal(&src_extension_info->loc,
                           &msg_conversion->src_loc) &&
          ten_string_is_equal(&msg_conversion->msg_name, &self->msg_name)) {
        ten_json_t *msg_and_result_json =
            ten_msg_and_result_conversion_operation_to_json(
                msg_conversion->msg_and_result_conversion_operation, err);
        if (!msg_and_result_json) {
          ten_json_destroy(json);
          return NULL;
        }

        ten_json_object_set_new(dest_json, TEN_STR_MSG_CONVERSION,
                                msg_and_result_json);
      }
    }

    ten_json_array_append_new(dests_json, dest_json);
  }

  return json;
}

// NOLINTNEXTLINE(misc-no-recursion)
ten_shared_ptr_t *ten_msg_dest_static_info_from_json(
    ten_json_t *json, ten_list_t *extensions_info,
    ten_extension_info_t *src_extension_info) {
  TEN_ASSERT(json && extensions_info, "Should not happen.");

  const char *msg_name = ten_json_object_peek_string(json, TEN_STR_NAME);
  if (!msg_name) {
    msg_name = "";
  }

  ten_msg_dest_static_info_t *self = ten_msg_dest_static_info_create(msg_name);

  ten_json_t *dests_json = ten_json_object_peek(json, TEN_STR_DEST);
  TEN_ASSERT(ten_json_is_array(dests_json), "Should not happen.");

  if (dests_json) {
    size_t i = 0;
    ten_json_t *dest_json = NULL;
    ten_json_array_foreach(dests_json, i, dest_json) {
      TEN_ASSERT(ten_json_is_object(dest_json), "Should not happen.");

      ten_shared_ptr_t *dest =
          ten_extension_info_parse_connection_dest_part_from_json(
              dest_json, extensions_info, src_extension_info, msg_name, NULL);
      if (!dest) {
        ten_msg_dest_static_info_destroy(self);
        return NULL;
      }

      // We need to use weak_ptr here to prevent the circular shared_ptr problem
      // in the case of loop graph.
      ten_weak_ptr_t *weak_dest = ten_weak_ptr_create(dest);
      ten_list_push_smart_ptr_back(&self->dest, weak_dest);
      ten_weak_ptr_destroy(weak_dest);
    }
  }

  return ten_shared_ptr_create(self, ten_msg_dest_static_info_destroy);
}
