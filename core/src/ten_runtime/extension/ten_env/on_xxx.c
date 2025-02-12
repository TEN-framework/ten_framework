//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#include "include_internal/ten_runtime/extension/on_xxx.h"

#include "include_internal/ten_runtime/common/loc.h"
#include "include_internal/ten_runtime/extension/base_dir.h"
#include "include_internal/ten_runtime/extension/close.h"
#include "include_internal/ten_runtime/extension/extension.h"
#include "include_internal/ten_runtime/extension/metadata.h"
#include "include_internal/ten_runtime/extension/msg_handling.h"
#include "include_internal/ten_runtime/extension/path_timer.h"
#include "include_internal/ten_runtime/extension_group/extension_group.h"
#include "include_internal/ten_runtime/extension_store/extension_store.h"
#include "include_internal/ten_runtime/extension_thread/extension_thread.h"
#include "include_internal/ten_runtime/extension_thread/msg_interface/common.h"
#include "include_internal/ten_runtime/extension_thread/on_xxx.h"
#include "include_internal/ten_runtime/metadata/metadata_info.h"
#include "include_internal/ten_runtime/msg/msg.h"
#include "include_internal/ten_runtime/ten_env/ten_env.h"
#include "include_internal/ten_runtime/timer/timer.h"
#include "ten_utils/log/log.h"
#include "ten_utils/macro/check.h"

static bool ten_extension_parse_interface_schema(ten_extension_t *self,
                                                 ten_value_t *api_definition,
                                                 ten_error_t *err) {
  TEN_ASSERT(self && ten_extension_check_integrity(self, true),
             "Invalid argument.");
  TEN_ASSERT(api_definition && ten_value_check_integrity(api_definition),
             "Invalid argument.");

  bool result = ten_schema_store_set_interface_schema_definition(
      &self->schema_store, api_definition, ten_extension_get_base_dir(self),
      err);
  if (!result) {
    TEN_LOGW("[%s] Failed to set interface schema definition: %s.",
             ten_extension_get_name(self, true), ten_error_message(err));
  }

  return result;
}

static void ten_extension_adjust_and_validate_property_on_configure_done(
    ten_extension_t *self) {
  TEN_ASSERT(self && ten_extension_check_integrity(self, true),
             "Should not happen.");

  ten_error_t err;
  TEN_ERROR_INIT(err);

  bool success = ten_schema_store_adjust_properties(&self->schema_store,
                                                    &self->property, &err);
  if (!success) {
    TEN_LOGW("[%s] Failed to adjust property type: %s.",
             ten_extension_get_name(self, true), ten_error_message(&err));
    goto done;
  }

  success = ten_schema_store_validate_properties(&self->schema_store,
                                                 &self->property, &err);
  if (!success) {
    TEN_LOGW("[%s] Invalid property: %s.", ten_extension_get_name(self, true),
             ten_error_message(&err));
    goto done;
  }

done:
  ten_error_deinit(&err);
  if (!success) {
    TEN_ASSERT(0, "Invalid property.");
  }
}

bool ten_extension_on_configure_done(ten_env_t *self) {
  TEN_ASSERT(self, "Invalid argument.");
  TEN_ASSERT(ten_env_check_integrity(self, true), "Invalid use of ten_env %p.",
             self);

  ten_extension_t *extension = ten_env_get_attached_extension(self);
  TEN_ASSERT(extension, "Invalid argument.");
  TEN_ASSERT(ten_extension_check_integrity(extension, true),
             "Invalid use of extension %p.", extension);

  if (extension->state != TEN_EXTENSION_STATE_ON_CONFIGURE) {
    TEN_LOGI(
        "[%s] Failed to on_configure_done() because of incorrect timing: %d",
        ten_extension_get_name(extension, true), extension->state);
    return false;
  }

  TEN_LOGD("[%s] on_configure() done.",
           ten_extension_get_name(extension, true));

  extension->state = TEN_EXTENSION_STATE_ON_CONFIGURE_DONE;

  ten_extension_thread_t *extension_thread = extension->extension_thread;
  TEN_ASSERT(extension_thread &&
                 ten_extension_thread_check_integrity(extension_thread, true),
             "Should not happen.");

  if (extension_thread->is_close_triggered) {
    // Do not proceed with the subsequent init/start flow, as the extension
    // thread is about to shut down.
    TEN_LOGD(
        "[%s] Since the close process has already been triggered, no further "
        "steps will be carried out after `on_configure_done`. Enter `on_stop`"
        "immediately.",
        ten_extension_get_name(extension, true));

    ten_extension_on_stop(extension);
    return true;
  }

  ten_error_t err;
  TEN_ERROR_INIT(err);

  bool rc = ten_handle_manifest_info_when_on_configure_done(
      &extension->manifest_info, ten_extension_get_base_dir(extension),
      &extension->manifest, &err);
  if (!rc) {
    TEN_LOGW("Failed to load extension manifest data, FATAL ERROR.");
    // NOLINTNEXTLINE(concurrency-mt-unsafe)
    exit(EXIT_FAILURE);
  }

  rc = ten_handle_property_info_when_on_configure_done(
      &extension->property_info, ten_extension_get_base_dir(extension),
      &extension->property, &err);
  if (!rc) {
    TEN_LOGW("Failed to load extension property data, FATAL ERROR.");
    // NOLINTNEXTLINE(concurrency-mt-unsafe)
    exit(EXIT_FAILURE);
  }

  rc = ten_extension_resolve_properties_in_graph(extension, &err);
  TEN_ASSERT(rc, "Failed to resolve properties in graph.");

  ten_extension_merge_properties_from_graph(extension);

  rc = ten_extension_handle_ten_namespace_properties(
      extension, extension->extension_context);
  TEN_ASSERT(rc, "[%s] Failed to handle '_ten' properties.",
             ten_string_get_raw_str(&extension->name));

  ten_value_t *api_definition = ten_metadata_init_schema_store(
      &extension->manifest, &extension->schema_store);
  if (api_definition) {
    bool success =
        ten_extension_parse_interface_schema(extension, api_definition, &err);
    TEN_ASSERT(success, "Failed to parse interface schema.");
  }

  ten_extension_adjust_and_validate_property_on_configure_done(extension);

  // Create timers for automatically cleaning expired IN_PATHs and OUT_PATHs.
  ten_timer_t *in_path_timer =
      ten_extension_create_timer_for_in_path(extension);
  ten_list_push_ptr_back(&extension->path_timers, in_path_timer, NULL);
  ten_timer_enable(in_path_timer);

  ten_timer_t *out_path_timer =
      ten_extension_create_timer_for_out_path(extension);
  ten_list_push_ptr_back(&extension->path_timers, out_path_timer, NULL);
  ten_timer_enable(out_path_timer);

  // The interface info has been resolved, and extensions might send msg out
  // during `on_start()`, so it's the best time to merge the interface info to
  // the extension_info.
  rc =
      ten_extension_determine_and_merge_all_interface_dest_extension(extension);
  TEN_ASSERT(rc, "Should not happen.");

  // Trigger the extension on_init flow.
  ten_extension_on_init(extension->ten_env);

  ten_error_deinit(&err);

  return true;
}

bool ten_extension_on_init_done(ten_env_t *self) {
  TEN_ASSERT(self, "Invalid argument.");
  TEN_ASSERT(ten_env_check_integrity(self, true), "Invalid use of ten_env %p.",
             self);

  ten_extension_t *extension = ten_env_get_attached_extension(self);
  TEN_ASSERT(extension, "Invalid argument.");
  TEN_ASSERT(ten_extension_check_integrity(extension, true),
             "Invalid use of extension %p.", extension);

  if (extension->state != TEN_EXTENSION_STATE_ON_INIT) {
    // `on_init_done` can only be called at specific times.
    TEN_LOGI("[%s] Failed to on_init_done() because of incorrect timing: %d",
             ten_extension_get_name(extension, true), extension->state);
    return false;
  }

  TEN_LOGD("[%s] on_init() done.", ten_extension_get_name(extension, true));

  extension->state = TEN_EXTENSION_STATE_ON_INIT_DONE;

  ten_extension_thread_t *extension_thread = extension->extension_thread;
  TEN_ASSERT(extension_thread &&
                 ten_extension_thread_check_integrity(extension_thread, true),
             "Should not happen.");

  if (extension_thread->is_close_triggered) {
    // Do not proceed with the subsequent start flow, as the extension thread is
    // about to shut down.
    TEN_LOGD(
        "[%s] Since the close process has already been triggered, no further "
        "steps will be carried out after `on_init_done`. Enter `on_stop`"
        "immediately.",
        ten_extension_get_name(extension, true));

    ten_extension_on_stop(extension);
    return true;
  }

  // Trigger on_start of extension.
  ten_extension_on_start(extension);

  return true;
}

static void ten_extension_flush_all_pending_msgs_received_in_init_stage(
    ten_extension_t *self) {
  TEN_ASSERT(self, "Invalid argument.");
  TEN_ASSERT(ten_extension_check_integrity(self, true),
             "Invalid use of extension %p.", self);

  // Flush the previously got messages, which are received before
  // on_init_done(), into the extension.
  ten_extension_thread_t *extension_thread = self->extension_thread;
  ten_list_foreach (&extension_thread->pending_msgs_received_in_init_stage,
                    iter) {
    ten_shared_ptr_t *msg = ten_smart_ptr_listnode_get(iter.node);
    TEN_ASSERT(msg, "Should not happen.");

    ten_loc_t *dest_loc = ten_msg_get_first_dest_loc(msg);
    TEN_ASSERT(dest_loc, "Should not happen.");

    if (ten_string_is_equal(&dest_loc->extension_name, &self->name)) {
      ten_extension_handle_in_msg(self, msg);
      ten_list_remove_node(
          &extension_thread->pending_msgs_received_in_init_stage, iter.node);
    }
  }

  // Flush the previously got messages, which are received before
  // on_init_done(), into the extension.
  ten_list_foreach (&self->pending_msgs_received_before_on_start_done, iter) {
    ten_shared_ptr_t *msg = ten_smart_ptr_listnode_get(iter.node);
    TEN_ASSERT(msg, "Should not happen.");

    ten_extension_handle_in_msg(self, msg);
  }
  ten_list_clear(&self->pending_msgs_received_before_on_start_done);
}

bool ten_extension_on_start_done(ten_env_t *self) {
  TEN_ASSERT(self, "Invalid argument.");
  TEN_ASSERT(ten_env_check_integrity(self, true), "Invalid use of ten_env %p.",
             self);

  ten_extension_t *extension = ten_env_get_attached_extension(self);
  TEN_ASSERT(extension, "Invalid argument.");
  TEN_ASSERT(ten_extension_check_integrity(extension, true),
             "Invalid use of extension %p.", extension);

  if (extension->state != TEN_EXTENSION_STATE_ON_START) {
    TEN_LOGI("[%s] Failed to on_start_done() because of incorrect timing: %d",
             ten_extension_get_name(extension, true), extension->state);
    return false;
  }

  TEN_LOGI("[%s] on_start() done.", ten_extension_get_name(extension, true));

  extension->state = TEN_EXTENSION_STATE_ON_START_DONE;

  ten_extension_thread_t *extension_thread = extension->extension_thread;
  TEN_ASSERT(extension_thread &&
                 ten_extension_thread_check_integrity(extension_thread, true),
             "Should not happen.");

  if (extension_thread->is_close_triggered) {
    TEN_LOGD(
        "[%s] Since the close process has already been triggered, no further "
        "steps will be carried out after `on_start_done`. Enter `on_stop`"
        "immediately.",
        ten_extension_get_name(extension, true));

    ten_extension_on_stop(extension);
    return true;
  }

  ten_extension_flush_all_pending_msgs_received_in_init_stage(extension);

  return true;
}

bool ten_extension_on_stop_done(ten_env_t *self) {
  TEN_ASSERT(self, "Invalid argument.");
  TEN_ASSERT(ten_env_check_integrity(self, true), "Invalid use of ten_env %p.",
             self);

  ten_extension_t *extension = ten_env_get_attached_extension(self);
  TEN_ASSERT(extension, "Invalid argument.");
  TEN_ASSERT(ten_extension_check_integrity(extension, true),
             "Invalid use of extension %p.", extension);

  TEN_LOGI("[%s] on_stop() done.", ten_extension_get_name(extension, true));

  if (extension->state != TEN_EXTENSION_STATE_ON_STOP) {
    TEN_LOGI("[%s] Failed to on_stop_done() because of incorrect timing: %d",
             ten_extension_get_name(extension, true), extension->state);
    return false;
  }

  extension->state = TEN_EXTENSION_STATE_ON_STOP_DONE;

  ten_extension_do_pre_close_action(extension);

  return true;
}

static void ten_extension_thread_del_extension(void *self_, void *extension_) {
  ten_extension_thread_t *self = self_;
  ten_extension_t *extension = extension_;

  TEN_ASSERT(self, "Invalid argument.");
  TEN_ASSERT(ten_extension_thread_check_integrity(self, true),
             "Invalid use of extension_thread %p.", self);
  TEN_ASSERT(extension, "Invalid argument.");

  ten_extension_inherit_thread_ownership(extension, self);
  TEN_ASSERT(ten_extension_check_integrity(extension, true),
             "Invalid use of extension %p.", extension);

  TEN_LOGD("[%s] Deleted from extension thread (%s).",
           ten_extension_get_name(extension, true),
           ten_string_get_raw_str(&self->extension_group->name));

  // Delete the extension from the extension store of the extension thread, so
  // that no more messages could be routed to this extension in the future.
  ten_extension_store_del_extension(self->extension_store, extension);

  self->extensions_cnt_of_deleted++;
  if (self->extensions_cnt_of_deleted == ten_list_size(&self->extensions)) {
    ten_extension_group_destroy_extensions(self->extension_group,
                                           self->extensions);
  }
}

static void ten_extension_thread_on_extension_on_deinit_done(
    ten_extension_thread_t *self, ten_extension_t *deinit_extension) {
  TEN_ASSERT(self, "Invalid argument.");
  TEN_ASSERT(ten_extension_thread_check_integrity(self, true),
             "Invalid use of extension_thread %p.", self);
  TEN_ASSERT(
      deinit_extension && ten_extension_check_integrity(deinit_extension, true),
      "Should not happen.");
  TEN_ASSERT(deinit_extension->extension_thread == self, "Should not happen.");

  // Notify the 'ten' object of this extension that we are closing.
  TEN_ASSERT(deinit_extension->ten_env &&
                 ten_env_check_integrity(deinit_extension->ten_env, true),
             "Should not happen.");

  // The extensions cannot be deleted immediately at this point. Instead, the
  // deletion action needs to be turned into an asynchronous task and placed at
  // the end of the task queue. The reason for this is that at this moment, the
  // extension thread's runloop may still contain some tasks, and the arguments
  // of those tasks may reference the `deinit_extension` specified by this
  // function. If `deinit_extension` is deleted immediately here, those tasks,
  // which are scheduled to execute in the future, may attempt to access a
  // dangling pointer to `deinit_extension`. By making the deletion of the
  // extension asynchronous and placing it at the tail of the task queue, the
  // situation of accessing a dangling pointer can be avoided. Furthermore,
  // since `ten_env` is already closed (via `ten_env_close()`) after
  // `on_deinit_done()`, all `ten_env` API calls made after `on_deinit_done`
  // will synchronously return failure. This ensures that no new tasks will be
  // added to the extension thread's runloop. As a result, once the asynchronous
  // task to destroy the extension is completed, no further tasks will be
  // executed. Therefore, placing the task to delete the extension at the end of
  // the queue ensures that no tasks executed afterward will access the raw
  // pointer to `deinit_extension`.
  ten_runloop_post_task_tail(self->runloop, ten_extension_thread_del_extension,
                             self, deinit_extension);
}

bool ten_extension_on_deinit_done(ten_env_t *self) {
  TEN_ASSERT(self, "Invalid argument.");
  TEN_ASSERT(ten_env_check_integrity(self, true), "Invalid use of ten_env %p.",
             self);

  ten_extension_t *extension = ten_env_get_attached_extension(self);
  TEN_ASSERT(extension, "Invalid argument.");
  TEN_ASSERT(ten_extension_check_integrity(extension, true),
             "Invalid use of extension %p.", extension);

  if (extension->state != TEN_EXTENSION_STATE_ON_DEINIT) {
    TEN_LOGI("[%s] Failed to on_deinit_done() because of incorrect timing: %d",
             ten_extension_get_name(extension, true), extension->state);
    return false;
  }

  extension->state = TEN_EXTENSION_STATE_ON_DEINIT_DONE;

  TEN_LOGD("[%s] on_deinit() done.", ten_extension_get_name(extension, true));

  // Close the ten_env so that any apis called on the ten_env will return
  // TEN_ERROR_ENV_CLOSED.
  ten_env_close(self);

  // Important: All the registered result handlers have to be called.
  //
  // Ex: If there are still some _IN_ or _OUT_ paths remaining in the path table
  // of extensions, in order to prevent memory leaks such as the result handler
  // itself in C++ binding, we need to create the corresponding cmd results
  // and send them into the original source extension.
  //
  // This means that once users call ten_env_on_deinit_done, then no further
  // messages will be received, and the originally registered cmd_result_handler
  // will be called back with an error. So, if developers/extensions truly care
  // about the result, they should perform `on_deinit_done` only after receiving
  // the result.
  ten_extension_flush_remaining_paths(extension);

  if (!ten_list_is_empty(&self->ten_proxy_list)) {
    // There is still the presence of ten_env_proxy, so the closing process
    // cannot continue.
    TEN_LOGI(
        "[%s] Waiting for ten_env_proxy to be released, remaining %d "
        "ten_env_proxy(s).",
        ten_extension_get_name(extension, true),
        ten_list_size(&self->ten_proxy_list));
    return true;
  }

  ten_extension_thread_on_extension_on_deinit_done(extension->extension_thread,
                                                   extension);

  return true;
}

bool ten_extension_on_ten_env_proxy_released(ten_env_t *self) {
  TEN_ASSERT(self, "Invalid argument.");
  TEN_ASSERT(ten_env_check_integrity(self, true), "Invalid use of ten_env %p.",
             self);

  ten_extension_t *extension = ten_env_get_attached_extension(self);
  TEN_ASSERT(extension, "Invalid argument.");
  TEN_ASSERT(ten_extension_check_integrity(extension, true),
             "Invalid use of extension %p.", extension);

  if (!ten_list_is_empty(&self->ten_proxy_list)) {
    // There is still the presence of ten_env_proxy, so the closing process
    // cannot continue.
    TEN_LOGI(
        "[%s] Waiting for ten_env_proxy to be released, remaining %d "
        "ten_env_proxy(s).",
        ten_extension_get_name(extension, true),
        ten_list_size(&self->ten_proxy_list));
    return true;
  }

  ten_extension_thread_on_extension_on_deinit_done(extension->extension_thread,
                                                   extension);

  return true;
}
