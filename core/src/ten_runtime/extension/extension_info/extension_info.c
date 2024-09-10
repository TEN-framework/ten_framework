//
// This file is part of the TEN Framework project.
// See https://github.com/TEN-framework/ten_framework/LICENSE for license
// information.
//
#include "include_internal/ten_runtime/extension/extension_info/extension_info.h"

#include <stdlib.h>

#include "include_internal/ten_runtime/common/constant_str.h"
#include "include_internal/ten_runtime/common/loc.h"
#include "include_internal/ten_runtime/extension/extension.h"
#include "include_internal/ten_runtime/extension/msg_dest_info/msg_dest_info.h"
#include "include_internal/ten_runtime/msg_conversion/msg_conversion/msg_conversion.h"
#include "ten_runtime/common/errno.h"
#include "ten_utils/container/list.h"
#include "ten_utils/container/list_node_ptr.h"
#include "ten_utils/container/list_node_smart_ptr.h"
#include "ten_utils/lib/alloc.h"
#include "ten_utils/lib/error.h"
#include "ten_utils/lib/smart_ptr.h"
#include "ten_utils/lib/string.h"
#include "ten_utils/macro/check.h"
#include "ten_utils/value/value.h"

ten_extension_info_t *ten_extension_info_create(void) {
  ten_extension_info_t *self =
      (ten_extension_info_t *)TEN_MALLOC(sizeof(ten_extension_info_t));
  TEN_ASSERT(self, "Failed to allocate memory.");

  ten_signature_set(&self->signature,
                    (ten_signature_t)TEN_EXTENSION_INFO_SIGNATURE);
  ten_sanitizer_thread_check_init_with_current_thread(&self->thread_check);

  ten_string_init(&self->extension_addon_name);

  ten_loc_init_empty(&self->loc);
  self->property = ten_value_create_object_with_move(NULL);

  ten_list_init(&self->msg_conversions);

  ten_all_msg_type_dest_static_info_init(&self->msg_dest_static_info);

  return self;
}

bool ten_extension_info_is_desired_extension_group(
    ten_extension_info_t *self, const char *app_uri,
    const char *extension_group_name) {
  TEN_ASSERT(self, "Invalid argument.");
  // TEN_NOLINTNEXTLINE(thread-check)
  // thread-check: The graph-related information of the extension remains
  // unchanged during the lifecycle of engine/graph, allowing safe
  // cross-thread access.
  TEN_ASSERT(ten_extension_info_check_integrity(self, false),
             "Invalid use of extension_info %p.", self);

  TEN_ASSERT(app_uri && extension_group_name, "Should not happen.");

  if (ten_string_is_equal_c_str(&self->loc.app_uri, app_uri) &&
      ten_string_is_equal_c_str(&self->loc.extension_group_name,
                                extension_group_name)) {
    return true;
  }
  return false;
}

static bool ten_extension_info_is_specified_extension(
    ten_extension_info_t *self, const char *app_uri, const char *graph_name,
    const char *extension_group_name, const char *extension_name) {
  TEN_ASSERT(self && ten_extension_info_check_integrity(self, true) &&
                 extension_group_name && extension_name,
             "Should not happen.");

  if (app_uri && !ten_string_is_equal_c_str(&self->loc.app_uri, app_uri)) {
    return false;
  }

  if (graph_name &&
      !ten_string_is_equal_c_str(&self->loc.graph_name, graph_name)) {
    return false;
  }

  if (extension_group_name &&
      !ten_string_is_equal_c_str(&self->loc.extension_group_name,
                                 extension_group_name)) {
    return false;
  }

  if (extension_name &&
      !ten_string_is_equal_c_str(&self->loc.extension_name, extension_name)) {
    return false;
  }

  return true;
}

static void ten_extension_info_destroy(ten_extension_info_t *self) {
  TEN_ASSERT(self, "Invalid argument.");
  // TEN_NOLINTNEXTLINE(thread-check)
  // thread-check: In TEN, the destroy operation should only be allowed to be
  // invoked when there are no thread safety issues present.
  TEN_ASSERT(ten_extension_info_check_integrity(self, false),
             "Invalid use of extension_info %p.", self);

  ten_sanitizer_thread_check_deinit(&self->thread_check);
  ten_signature_set(&self->signature, 0);

  ten_all_msg_type_dest_static_info_deinit(&self->msg_dest_static_info);

  ten_string_deinit(&self->extension_addon_name);

  ten_loc_deinit(&self->loc);

  if (self->property) {
    ten_value_destroy(self->property);
  }
  ten_list_clear(&self->msg_conversions);

  TEN_FREE(self);
}

// 1. All extension_info will be stored in the `extensions_info`, including
// those defined in the `nodes` section, and those defined in each `dest`
// section in `connections`.
//
// 2. All extension_info in the `extensions_info` are unique, which is
// identified by the `loc` field.
//
// 3. Each extension_info in the `extensions_info` is a shared_ptr, and if one
// is used in the `dest` section, a weak_ptr will be created to reference it to
// avoid there is a cycle in the graph.
ten_shared_ptr_t *get_extension_info_in_extensions_info(
    ten_list_t *extensions_info, const char *app_uri, const char *graph_name,
    const char *extension_group_name, const char *extension_addon_name,
    const char *extension_instance_name, bool *new_one_created,
    ten_error_t *err) {
  TEN_ASSERT(extensions_info && extension_group_name && extension_instance_name,
             "Should not happen.");

  ten_extension_info_t *extension_info = NULL;

  // Find the corresponding extension_info according to the instance name of
  // extension_group and extension.
  ten_listnode_t *extension_info_node = ten_list_find_shared_ptr_custom_4(
      extensions_info, app_uri, graph_name, extension_group_name,
      extension_instance_name, ten_extension_info_is_specified_extension);
  if (extension_info_node) {
    extension_info = ten_shared_ptr_get_data(
        ten_smart_ptr_listnode_get(extension_info_node));
    TEN_ASSERT(extension_info &&
                   ten_extension_info_check_integrity(extension_info, true),
               "Should not happen.");

    if (new_one_created) {
      *new_one_created = false;
    }

    // The extension addon name should be equal if both specified.
    if (extension_addon_name && strlen(extension_addon_name) &&
        !ten_string_is_empty(&extension_info->extension_addon_name) &&
        !ten_string_is_equal_c_str(&extension_info->extension_addon_name,
                                   extension_addon_name)) {
      if (err) {
        ten_error_set(
            err, TEN_ERRNO_INVALID_GRAPH,
            "extension '%s' is associated with different addon '%s', "
            "'%s'",
            extension_instance_name, extension_addon_name,
            ten_string_get_raw_str(&extension_info->extension_addon_name));
      }

      TEN_ASSERT(0,
                 "extension '%s' is associated with different addon '%s', "
                 "'%s'",
                 extension_instance_name, extension_addon_name,
                 ten_string_get_raw_str(&extension_info->extension_addon_name));

      return NULL;
    }

    // If we know the extension addon name now, add this data to the
    // extension_info.
    if (extension_addon_name && strlen(extension_addon_name) &&
        ten_string_is_empty(&extension_info->extension_addon_name)) {
      ten_string_set_formatted(&extension_info->extension_addon_name, "%s",
                               extension_addon_name);
    }

    return ten_smart_ptr_listnode_get(extension_info_node);
  }

  TEN_ASSERT(!extension_info, "Should not happen.");

  ten_extension_info_t *self = ten_extension_info_create();

  ten_loc_set(&self->loc, app_uri, graph_name, extension_group_name,
              extension_instance_name, NULL);

  // Add the extension addon name if we know it now.
  if (extension_addon_name && strlen(extension_addon_name)) {
    ten_string_set_formatted(&self->extension_addon_name, "%s",
                             extension_addon_name);
  }

  ten_shared_ptr_t *shared_self =
      ten_shared_ptr_create(self, ten_extension_info_destroy);
  ten_shared_ptr_t *shared_self_ =
      ten_list_push_smart_ptr_back(extensions_info, shared_self);
  ten_shared_ptr_destroy(shared_self);

  if (new_one_created) {
    *new_one_created = true;
  }
  return shared_self_;
}

static bool copy_msg_dest(ten_list_t *to_static_info,
                          ten_list_t *from_static_info,
                          ten_list_t *extensions_info, ten_error_t *err) {
  TEN_ASSERT(to_static_info && extensions_info, "Should not happen.");

  ten_list_foreach (from_static_info, iter) {
    ten_shared_ptr_t *msg_dest_static_info =
        ten_smart_ptr_listnode_get(iter.node);

    ten_shared_ptr_t *new_msg_dest_static_info = ten_msg_dest_static_info_clone(
        msg_dest_static_info, extensions_info, err);
    if (!new_msg_dest_static_info) {
      return false;
    }

    ten_list_push_smart_ptr_back(to_static_info, new_msg_dest_static_info);

    ten_shared_ptr_destroy(new_msg_dest_static_info);
  }

  return true;
}

ten_shared_ptr_t *ten_extension_info_clone(ten_extension_info_t *self,
                                           ten_list_t *extensions_info,
                                           ten_error_t *err) {
  TEN_ASSERT(extensions_info, "Should not happen.");

  TEN_ASSERT(self, "Invalid argument.");
  // TEN_NOLINTNEXTLINE(thread-check)
  // thread-check: The graph-related information of the extension remains
  // unchanged during the lifecycle of engine/graph, allowing safe
  // cross-thread access.
  TEN_ASSERT(ten_extension_info_check_integrity(self, false),
             "Invalid use of extension_info %p.", self);

  bool new_extension_info_created = false;
  ten_shared_ptr_t *new_dest = get_extension_info_in_extensions_info(
      extensions_info, ten_string_get_raw_str(&self->loc.app_uri),
      ten_string_get_raw_str(&self->loc.graph_name),
      ten_string_get_raw_str(&self->loc.extension_group_name),
      ten_string_get_raw_str(&self->extension_addon_name),
      ten_string_get_raw_str(&self->loc.extension_name),
      &new_extension_info_created, err);

  if (new_extension_info_created) {
    ten_extension_info_t *new_extension_info =
        ten_shared_ptr_get_data(new_dest);
    TEN_ASSERT(new_extension_info &&
                   ten_extension_info_check_integrity(new_extension_info, true),
               "Should not happen.");

    ten_value_object_merge_with_clone(new_extension_info->property,
                                      self->property);

    ten_list_foreach (&self->msg_conversions, iter) {
      ten_msg_conversion_t *msg_conversion = ten_ptr_listnode_get(iter.node);
      TEN_ASSERT(
          msg_conversion && ten_msg_conversion_check_integrity(msg_conversion),
          "Should not happen.");

      bool rc = ten_msg_conversion_merge(&new_extension_info->msg_conversions,
                                         msg_conversion, err);
      TEN_ASSERT(rc, "Should not happen.");
    }

    if (!copy_msg_dest(&new_extension_info->msg_dest_static_info.cmd,
                       &self->msg_dest_static_info.cmd, extensions_info, err)) {
      return NULL;
    }

    if (!copy_msg_dest(&new_extension_info->msg_dest_static_info.data,
                       &self->msg_dest_static_info.data, extensions_info,
                       err)) {
      return NULL;
    }

    if (!copy_msg_dest(&new_extension_info->msg_dest_static_info.audio_frame,
                       &self->msg_dest_static_info.audio_frame, extensions_info,
                       err)) {
      return NULL;
    }

    if (!copy_msg_dest(&new_extension_info->msg_dest_static_info.video_frame,
                       &self->msg_dest_static_info.video_frame, extensions_info,
                       err)) {
      return NULL;
    }

    if (!copy_msg_dest(&new_extension_info->msg_dest_static_info.interface,
                       &self->msg_dest_static_info.interface, extensions_info,
                       err)) {
      return NULL;
    }
  }

  return new_dest;
}

bool ten_extension_info_check_integrity(ten_extension_info_t *self,
                                        bool check_thread) {
  TEN_ASSERT(self, "Should not happen.");

  if (ten_signature_get(&self->signature) !=
      (ten_signature_t)TEN_EXTENSION_INFO_SIGNATURE) {
    return false;
  }

  if (check_thread &&
      !ten_sanitizer_thread_check_do_check(&self->thread_check)) {
    return false;
  }

  return true;
}

void ten_extension_info_translate_localhost_to_app_uri(
    ten_extension_info_t *self, const char *uri) {
  TEN_ASSERT(self && ten_extension_info_check_integrity(self, true) && uri,
             "Should not happen.");

  if (ten_string_is_equal_c_str(&self->loc.app_uri, TEN_STR_LOCALHOST) ||
      ten_string_is_empty(&self->loc.app_uri)) {
    ten_string_copy_c_str(&self->loc.app_uri, uri, strlen(uri));
  }
}

ten_extension_info_t *ten_extension_info_from_smart_ptr(
    ten_smart_ptr_t *extension_info_smart_ptr) {
  TEN_ASSERT(extension_info_smart_ptr, "Invalid argument.");
  return ten_smart_ptr_get_data(extension_info_smart_ptr);
}

static void ten_extension_info_fill_app_uri(ten_extension_info_t *self,
                                            const char *app_uri) {
  TEN_ASSERT(self && ten_extension_info_check_integrity(self, true),
             "Invalid argument.");
  TEN_ASSERT(app_uri, "Should not happen.");
  TEN_ASSERT(!ten_loc_is_empty(&self->loc), "Should not happen.");

  // Fill the app uri of the extension_info if it is empty.
  if (ten_string_is_empty(&self->loc.app_uri)) {
    ten_string_set_formatted(&self->loc.app_uri, "%s", app_uri);
  }

  // Fill the app uri of each item in the msg_conversions_list if it is empty.
  ten_list_foreach (&self->msg_conversions, iter) {
    ten_msg_conversion_t *conversion_iter = ten_ptr_listnode_get(iter.node);
    TEN_ASSERT(
        conversion_iter && ten_msg_conversion_check_integrity(conversion_iter),
        "Should not happen.");

    if (ten_string_is_empty(&conversion_iter->src_loc.app_uri)) {
      ten_string_set_formatted(&conversion_iter->src_loc.app_uri, "%s",
                               app_uri);
    }
  }
}

// Fill the app uri of each extension_info in the extensions_info.
void ten_extensions_info_fill_app_uri(ten_list_t *extensions_info,
                                      const char *app_uri) {
  ten_list_foreach (extensions_info, iter) {
    ten_extension_info_t *extension_info =
        ten_shared_ptr_get_data(ten_smart_ptr_listnode_get(iter.node));
    TEN_ASSERT(extension_info &&
                   ten_extension_info_check_integrity(extension_info, true),
               "Invalid argument.");

    ten_extension_info_fill_app_uri(extension_info, app_uri);
  }
}

static void ten_extension_info_fill_loc_info(ten_extension_info_t *self,
                                             const char *app_uri,
                                             const char *graph_name) {
  TEN_ASSERT(self, "Invalid argument.");
  // TEN_NOLINTNEXTLINE(thread-check)
  // thread-check: The graph-related information of the extension remains
  // unchanged during the lifecycle of engine/graph, allowing safe
  // cross-thread access.
  TEN_ASSERT(ten_extension_info_check_integrity(self, false),
             "Invalid use of extension_info %p.", self);

  if (ten_string_is_empty(&self->loc.graph_name)) {
    ten_string_set_formatted(&self->loc.graph_name, "%s", graph_name);
  }

  if (ten_string_is_empty(&self->loc.app_uri) ||
      ten_string_is_equal_c_str(&self->loc.app_uri, TEN_STR_LOCALHOST)) {
    ten_string_set_formatted(&self->loc.app_uri, app_uri);
  }

  // Fill the app uri of each item in the msg_conversions_list if it is empty.
  ten_list_foreach (&self->msg_conversions, iter) {
    ten_msg_conversion_t *conversion_iter = ten_ptr_listnode_get(iter.node);
    TEN_ASSERT(
        conversion_iter && ten_msg_conversion_check_integrity(conversion_iter),
        "Should not happen.");

    if (ten_string_is_empty(&conversion_iter->src_loc.graph_name)) {
      ten_string_set_formatted(&conversion_iter->src_loc.graph_name, "%s",
                               graph_name);
    }

    if (ten_string_is_empty(&self->loc.app_uri) ||
        ten_string_is_equal_c_str(&self->loc.app_uri, TEN_STR_LOCALHOST)) {
      ten_string_set_formatted(&self->loc.app_uri, app_uri);
    }
  }
}

void ten_extensions_info_fill_loc_info(ten_list_t *extensions_info,
                                       const char *app_uri,
                                       const char *graph_name) {
  ten_list_foreach (extensions_info, iter) {
    ten_extension_info_t *extension_info =
        ten_shared_ptr_get_data(ten_smart_ptr_listnode_get(iter.node));
    // TEN_NOLINTNEXTLINE(thread-check)
    // thread-check: The graph-related information of the extension remains
    // unchanged during the lifecycle of engine/graph, allowing safe
    // cross-thread access.
    TEN_ASSERT(extension_info &&
                   ten_extension_info_check_integrity(extension_info, false),
               "Invalid argument.");

    ten_extension_info_fill_loc_info(extension_info, app_uri, graph_name);
  }

  // Check if the extension_info in the `dest` section is correct.
  ten_list_foreach (extensions_info, iter) {
    ten_extension_info_t *extension_info =
        ten_shared_ptr_get_data(ten_smart_ptr_listnode_get(iter.node));

    ten_list_foreach (&extension_info->msg_dest_static_info.cmd, iter_cmd) {
      ten_msg_dest_static_info_t *dest_info =
          ten_shared_ptr_get_data(ten_smart_ptr_listnode_get(iter_cmd.node));
      ten_list_foreach (&dest_info->dest, dest_iter) {
        ten_extension_info_t *dest_extension_info =
            ten_smart_ptr_get_data(ten_smart_ptr_listnode_get(dest_iter.node));
        if (ten_string_is_empty(&dest_extension_info->loc.app_uri)) {
          TEN_ASSERT(0, "extension_info->loc.app_uri should not be empty.");
          return;
        }

        if (ten_string_is_equal_c_str(&dest_extension_info->loc.app_uri,
                                      TEN_STR_LOCALHOST)) {
          if (!ten_string_is_equal_c_str(&dest_extension_info->loc.app_uri,
                                         app_uri)) {
            TEN_ASSERT(0,
                       "extension_info->loc.app_uri should not be localhost.");
            return;
          }
        }

        if (ten_string_is_empty(&dest_extension_info->loc.graph_name)) {
          TEN_ASSERT(0, "extension_info->loc.graph_name should not be empty.");
          return;
        }
      }
    }
  }
}
