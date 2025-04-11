//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#include "ten_runtime/addon/addon.h"

#include "include_internal/ten_runtime/addon/addon.h"
#include "include_internal/ten_runtime/addon/addon_autoload.h"
#include "include_internal/ten_runtime/addon/addon_host.h"
#include "include_internal/ten_runtime/addon/addon_loader/addon_loader.h"
#include "include_internal/ten_runtime/addon/addon_manager.h"
#include "include_internal/ten_runtime/addon/common/common.h"
#include "include_internal/ten_runtime/addon/common/store.h"
#include "include_internal/ten_runtime/addon/extension/extension.h"
#include "include_internal/ten_runtime/addon/extension_group/extension_group.h"
#include "include_internal/ten_runtime/addon/protocol/protocol.h"
#include "include_internal/ten_runtime/app/app.h"
#include "include_internal/ten_runtime/app/base_dir.h"
#include "include_internal/ten_runtime/common/constant_str.h"
#include "include_internal/ten_runtime/extension/extension.h"
#include "include_internal/ten_runtime/extension_group/extension_group.h"
#include "include_internal/ten_runtime/metadata/metadata_info.h"
#include "include_internal/ten_runtime/ten_env/ten_env.h"
#include "ten_runtime/app/app.h"
#include "ten_runtime/binding/common.h"
#include "ten_runtime/ten_env/ten_env.h"
#include "ten_utils/lib/alloc.h"
#include "ten_utils/lib/error.h"
#include "ten_utils/lib/signature.h"
#include "ten_utils/lib/string.h"
#include "ten_utils/macro/check.h"

typedef struct ten_on_all_addons_unregistered_ctx_t {
  ten_env_on_all_addons_unregistered_cb_t cb;
  void *cb_data;
} ten_on_all_addons_unregistered_ctx_t;

bool ten_addon_check_integrity(ten_addon_t *self) {
  TEN_ASSERT(self, "Should not happen.");
  if (ten_signature_get(&self->signature) != TEN_ADDON_SIGNATURE) {
    return false;
  }

  return true;
}

void ten_addon_init(ten_addon_t *self, ten_addon_on_init_func_t on_init,
                    ten_addon_on_deinit_func_t on_deinit,
                    ten_addon_on_create_instance_func_t on_create_instance,
                    ten_addon_on_destroy_instance_func_t on_destroy_instance,
                    ten_addon_on_destroy_func_t on_destroy) {
  ten_binding_handle_set_me_in_target_lang((ten_binding_handle_t *)self, NULL);
  ten_signature_set(&self->signature, TEN_ADDON_SIGNATURE);

  self->on_init = on_init;
  self->on_deinit = on_deinit;
  self->on_create_instance = on_create_instance;
  self->on_destroy_instance = on_destroy_instance;
  self->on_destroy = on_destroy;

  self->user_data = NULL;
}

ten_addon_t *ten_addon_create(
    ten_addon_on_init_func_t on_init, ten_addon_on_deinit_func_t on_deinit,
    ten_addon_on_create_instance_func_t on_create_instance,
    ten_addon_on_destroy_instance_func_t on_destroy_instance,
    ten_addon_on_destroy_func_t on_destroy) {
  ten_addon_t *self = TEN_MALLOC(sizeof(ten_addon_t));
  TEN_ASSERT(self, "Failed to allocate memory.");

  ten_addon_init(self, on_init, on_deinit, on_create_instance,
                 on_destroy_instance, on_destroy);

  return self;
}

void ten_addon_destroy(ten_addon_t *self) {
  TEN_ASSERT(self && ten_addon_check_integrity(self), "Invalid argument.");
  TEN_FREE(self);
}

TEN_ADDON_TYPE ten_addon_type_from_string(const char *addon_type_str) {
  TEN_ASSERT(addon_type_str, "Invalid argument.");

  if (ten_c_string_is_equal(addon_type_str, TEN_STR_EXTENSION)) {
    return TEN_ADDON_TYPE_EXTENSION;
  } else if (ten_c_string_is_equal(addon_type_str, TEN_STR_EXTENSION_GROUP)) {
    return TEN_ADDON_TYPE_EXTENSION_GROUP;
  } else if (ten_c_string_is_equal(addon_type_str, TEN_STR_PROTOCOL)) {
    return TEN_ADDON_TYPE_PROTOCOL;
  } else if (ten_c_string_is_equal(addon_type_str, TEN_STR_ADDON_LOADER)) {
    return TEN_ADDON_TYPE_ADDON_LOADER;
  } else {
    return TEN_ADDON_TYPE_INVALID;
  }
}

const char *ten_addon_type_to_string(TEN_ADDON_TYPE type) {
  switch (type) {
  case TEN_ADDON_TYPE_EXTENSION:
    return TEN_STR_EXTENSION;
  case TEN_ADDON_TYPE_EXTENSION_GROUP:
    return TEN_STR_EXTENSION_GROUP;
  case TEN_ADDON_TYPE_PROTOCOL:
    return TEN_STR_PROTOCOL;
  case TEN_ADDON_TYPE_ADDON_LOADER:
    return TEN_STR_ADDON_LOADER;
  default:
    TEN_ASSERT(0, "Should not happen.");
    return NULL;
  }
}

static ten_string_t *ten_addon_find_base_dir_from_app(const char *addon_type,
                                                      const char *addon_name) {
  TEN_ASSERT(
      addon_type && strlen(addon_type) && addon_name && strlen(addon_name),
      "Invalid argument.");

  ten_string_t *base_dir = ten_find_app_base_dir();
  if (!base_dir || ten_string_is_empty(base_dir)) {
    TEN_ASSERT(0, "Failed to find base_dir of the current TEN app.");
    return NULL;
  }

  ten_string_t *addon_base_dir = ten_string_create_formatted(
      "%s/%s/%s/%s", ten_string_get_raw_str(base_dir), TEN_STR_TEN_PACKAGES,
      addon_type, addon_name);

  ten_string_destroy(base_dir);

  ten_path_to_system_flavor(addon_base_dir);

  if (!ten_path_exists(ten_string_get_raw_str(addon_base_dir))) {
    // Clear the `base_dir` to avoid users to use an invalid path.
    ten_string_destroy(addon_base_dir);
    addon_base_dir = NULL;
  }

  return addon_base_dir;
}

/**
 * @brief The registration flow of an addon is as follows.
 *
 *   register -> on_init --> on_init_done --> add to the addon store
 *
 * Developers could override the 'on_init' function to perform user-defined
 * operations the addon needs.
 */
static void ten_addon_register_internal(ten_addon_store_t *addon_store,
                                        ten_addon_host_t *addon_host,
                                        const char *name, const char *base_dir,
                                        ten_addon_t *addon) {
  TEN_ASSERT(addon_host && ten_addon_host_check_integrity(addon_host),
             "Should not happen.");
  TEN_ASSERT(name, "Should not happen.");

  addon_host->addon = addon;
  addon_host->store = addon_store;

  TEN_ASSERT(!addon_host->ten_env, "Should not happen.");
  addon_host->ten_env = ten_env_create_for_addon(addon_host);

  ten_string_set_formatted(&addon_host->name, "%s", name);

  if (base_dir) {
    if (!strcmp(base_dir, TEN_STR_ADDON_BASE_DIR_FIND_FROM_APP_BASE_DIR)) {
      ten_string_t *found_base_dir = ten_addon_find_base_dir_from_app(
          ten_addon_type_to_string(addon_host->type), name);
      TEN_ASSERT(found_base_dir, "Failed to find the base_dir of addon.");

      ten_string_copy(&addon_host->base_dir, found_base_dir);

      ten_string_destroy(found_base_dir);
    } else {
      // In some special cases, such as built-in addons, their logic does not
      // require a base directory at all, so `NULL` might be passed as the
      // base_dir parameter value.
      TEN_LOGV("Addon %s base_dir: %s", name, base_dir);
      ten_addon_host_find_and_set_base_dir(addon_host, base_dir);
    }
  }

  TEN_LOGV("Register addon: %s as %s", name,
           ten_addon_type_to_string(addon_host->type));

  ten_addon_host_load_metadata(addon_host, addon_host->ten_env,
                               addon_host->addon->on_init);
}

static void ten_app_create_addon_instance(ten_app_t *app,
                                          ten_addon_context_t *addon_context) {
  TEN_ASSERT(app && ten_app_check_integrity(app, true), "Invalid argument.");
  TEN_ASSERT(addon_context, "Invalid argument.");

  TEN_ADDON_TYPE addon_type = addon_context->addon_type;
  TEN_ASSERT(addon_type != TEN_ADDON_TYPE_INVALID, "Should not happen.");

  const char *addon_name = ten_string_get_raw_str(&addon_context->addon_name);
  TEN_ASSERT(addon_name, "Should not happen.");

  const char *instance_name =
      ten_string_get_raw_str(&addon_context->instance_name);
  TEN_ASSERT(instance_name, "Should not happen.");

  TEN_LOGD("Try to find addon for %s", addon_name);

  int lock_operation_rc = ten_addon_store_lock_by_type(addon_type);
  TEN_ASSERT(!lock_operation_rc, "Should not happen.");

  ten_addon_host_t *addon_host =
      ten_addon_store_find_by_type(addon_type, addon_name);
  if (!addon_host) {
    ten_addon_register_ctx_t *register_ctx = ten_addon_register_ctx_create(app);
    TEN_ASSERT(register_ctx, "Should not happen.");

    ten_error_t err;
    TEN_ERROR_INIT(err);

    // First, try to load it using the built-in native addon loader (i.e.,
    // `dlopen`).
    if (!ten_addon_try_load_specific_addon_using_native_addon_loader(
            ten_string_get_raw_str(&app->base_dir), addon_type, addon_name,
            register_ctx, &err)) {
      TEN_LOGI(
          "Unable to load addon %s:%s using native addon loader, will try "
          "other methods. error: %s",
          ten_addon_type_to_string(addon_type), addon_name,
          ten_error_message(&err));

      ten_addon_try_load_specific_addon_using_all_addon_loaders(addon_type,
                                                                addon_name);

      // The specific addon_name addon could be loaded by one of the addon
      // loaders. Register the addon again.
      if (!ten_addon_manager_register_specific_addon(
              ten_addon_manager_get_instance(), addon_type, addon_name,
              register_ctx)) {
        TEN_LOGI(
            "Unable to load addon %s:%s using all installed addon loaders. "
            "error: %s",
            ten_addon_type_to_string(addon_type), addon_name,
            ten_error_message(&err));
      }
    }

    // Find again.
    addon_host = ten_addon_store_find_by_type(addon_type, addon_name);

    ten_addon_register_ctx_destroy(register_ctx);
    ten_error_deinit(&err);
  }

  lock_operation_rc = ten_addon_store_unlock_by_type(addon_type);
  TEN_ASSERT(!lock_operation_rc, "Should not happen.");

  if (!addon_host) {
    TEN_ASSERT(0,
               "Failed to find addon %s:%s, please make sure the addon is "
               "installed.",
               ten_addon_type_to_string(addon_type), addon_name);
  }

  ten_addon_host_create_instance_async(addon_host, instance_name,
                                       addon_context);
}

static void ten_app_create_addon_instance_task(void *self_, void *arg) {
  ten_app_t *self = (ten_app_t *)self_;
  TEN_ASSERT(self && ten_app_check_integrity(self, true), "Invalid argument.");

  ten_addon_context_t *addon_context = (ten_addon_context_t *)arg;
  TEN_ASSERT(addon_context, "Invalid argument.");

  ten_app_create_addon_instance(self, addon_context);
}

/**
 * @param ten Might be the ten of the 'engine', or the ten of an extension
 * thread(group).
 * @param cb The callback when the creation is completed. Because there might be
 * more than one extension threads to create extensions from the corresponding
 * extension addons simultaneously. So we can _not_ save the function pointer
 * of @a cb into @a ten, instead we need to pass the function pointer of @a cb
 * through a parameter.
 * @param cb_data The user data of @a cb. Refer the comments of @a cb for the
 * reason why we pass the pointer of @a cb_data through a parameter rather than
 * saving it into @a ten.
 *
 * @note We will save the pointers of @a cb and @a cb_data into a 'ten' object
 * later in the call flow when the 'ten' object at that time belongs to a more
 * specific scope, so that we can minimize the parameters count then.
 */
bool ten_addon_create_instance_async(ten_env_t *ten_env,
                                     ten_addon_context_t *addon_context) {
  TEN_ASSERT(addon_context, "Should not happen.");

  TEN_ADDON_TYPE addon_type = addon_context->addon_type;
  TEN_ASSERT(addon_type != TEN_ADDON_TYPE_INVALID, "Should not happen.");

  TEN_ASSERT(ten_env, "Should not happen.");
  TEN_ASSERT(ten_env_check_integrity(ten_env, true), "Should not happen.");

  ten_app_t *app = ten_env_get_belonging_app(ten_env);
  TEN_ASSERT(app && ten_app_check_integrity(app,
                                            // TEN_NOLINTNEXTLINE(thread-check)
                                            // thread-check: This function could
                                            // be called from the engine thread
                                            // or the extension thread.
                                            false),
             "Should not happen.");

  // Check if the current thread is the app thread. If not, we need to post a
  // task to the app thread to perform the following operations.
  if (ten_app_thread_call_by_me(app)) {
    // Directly perform the following operations on the app thread.
    ten_app_create_addon_instance(app, addon_context);
  } else {
    // Post a task to the app thread to perform the following operations.
    int rc = ten_runloop_post_task_tail(
        app->loop, ten_app_create_addon_instance_task, app, addon_context);
    TEN_ASSERT(!rc, "Should not happen.");
  }

  return true;
}

ten_addon_host_t *ten_addon_register(TEN_ADDON_TYPE addon_type,
                                     const char *addon_name,
                                     const char *base_dir, ten_addon_t *addon,
                                     void *register_ctx) {
  TEN_ASSERT(addon_type != TEN_ADDON_TYPE_INVALID, "Invalid argument.");

  if (!addon_name || strlen(addon_name) == 0) {
    TEN_LOGE("The addon name is required.");
    // NOLINTNEXTLINE(concurrency-mt-unsafe)
    exit(EXIT_FAILURE);
  }

  ten_addon_host_t *addon_host =
      ten_addon_store_find_by_type(addon_type, addon_name);
  if (addon_host) {
    // This addon already exists; do not register it again.
    goto done;
  }

  addon_host = ten_addon_host_create(addon_type);

  ten_addon_register_ctx_t *register_ctx_ =
      (ten_addon_register_ctx_t *)register_ctx;
  TEN_ASSERT(register_ctx_, "Invalid argument.");

  ten_app_t *app = register_ctx_->app;
  TEN_ASSERT(app, "Should not happen.");
  // TODO(xilin): This function should be called on the app thread according to
  // the register_ctx->app.

  // TEN_ASSERT(ten_app_check_integrity(app, true), "Should not happen.");

  addon_host->attached_app = app;

  ten_addon_store_t *addon_store = NULL;
  switch (addon_type) {
  case TEN_ADDON_TYPE_EXTENSION:
    addon_store = ten_extension_get_global_store();
    break;
  case TEN_ADDON_TYPE_EXTENSION_GROUP:
    addon_store = ten_extension_group_get_global_store();
    break;
  case TEN_ADDON_TYPE_PROTOCOL:
    addon_store = ten_protocol_get_global_store();
    break;
  case TEN_ADDON_TYPE_ADDON_LOADER:
    addon_store = ten_addon_loader_get_global_store();
    break;
  default:
    break;
  }
  TEN_ASSERT(addon_store, "Should not happen.");

  ten_addon_store_add(addon_store, addon_host);

  ten_addon_register_internal(addon_store, addon_host, addon_name, base_dir,
                              addon);

done:
  return addon_host;
}

/**
 * @brief The unregistration flow of an addon is as follows.
 *
 *   unregister -> remove from the addon store -> on_deinit -> on_deinit_done
 *
 * Remove the 'addon' from the store first, so that we can ensure that there
 * are no more usages of this 'addon' anymore, and could be safe to perform
 * the on_init operation.
 *
 * The default behavior of the 'on_deinit' stage is to perform nothing but
 * 'on_deinit_done'. However, developers could override this through providing a
 * user-defined 'on_deinit' function.
 */
ten_addon_t *ten_addon_unregister(ten_addon_store_t *store,
                                  const char *addon_name) {
  TEN_ASSERT(store && addon_name, "Should not happen.");

  TEN_LOGV("Unregistered addon '%s'", addon_name);

  return ten_addon_store_del(store, addon_name);
}

static void ten_addon_unregister_all_except_addon_loader_addon(void) {
  ten_addon_unregister_all_extension();
  ten_addon_unregister_all_extension_group();
  ten_addon_unregister_all_protocol();

  // Destroy the addon manager to avoid memory leak.
  ten_addon_manager_destroy(ten_addon_manager_get_instance());
}

void ten_addon_unregister_all_and_cleanup(void) {
  // Since Python addons (e.g., Python extension addons) require access to the
  // Python VM when performing `addon_t` deinitialization, and the Python addon
  // loader will destroy the Python VM during its own destruction, the Python
  // addon loader must only be unloaded after all other non-addon-loader types
  // of addons have been fully unloaded. Only then can the addon loader itself
  // be unloaded.

  ten_addon_unregister_all_except_addon_loader_addon();

  // Destroy all addon loaders' singleton to avoid memory leak.
  ten_addon_loader_destroy_all_singleton_instances_immediately();

  ten_addon_unregister_all_addon_loader();
}

static void ten_on_all_addons_unregistered_cb(ten_env_t *ten_env,
                                              void *cb_data) {
  ten_on_all_addons_unregistered_ctx_t *ctx =
      (ten_on_all_addons_unregistered_ctx_t *)cb_data;
  TEN_ASSERT(ctx, "Invalid argument.");

  ten_addon_unregister_all_addon_loader();

  if (ctx->cb) {
    ctx->cb(ten_env, ctx->cb_data);
  }

  TEN_FREE(ctx);
}

TEN_RUNTIME_API void ten_addon_unregister_all_and_cleanup_after_app_close(
    ten_env_t *ten_env, ten_env_on_all_addons_unregistered_cb_t cb,
    void *cb_data) {
  TEN_ASSERT(ten_env, "Invalid argument.");
  TEN_ASSERT(ten_env_check_integrity(ten_env, true), "Invalid argument.");

  TEN_ASSERT(ten_env_get_attach_to(ten_env) == TEN_ENV_ATTACH_TO_APP,
             "Should not happen.");

  ten_app_t *app = ten_env_get_attached_app(ten_env);
  TEN_ASSERT(app && ten_app_check_integrity(app, true), "Should not happen.");

  ten_on_all_addons_unregistered_ctx_t *ctx =
      TEN_MALLOC(sizeof(ten_on_all_addons_unregistered_ctx_t));
  TEN_ASSERT(ctx, "Should not happen.");

  ctx->cb = cb;
  ctx->cb_data = cb_data;

  ten_addon_unregister_all_except_addon_loader_addon();

  ten_addon_loader_destroy_all_singleton_instances(
      ten_env, ten_on_all_addons_unregistered_cb, ctx);
}
