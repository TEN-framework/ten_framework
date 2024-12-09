//
// Copyright © 2024 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#include "include_internal/ten_runtime/addon/extension_group/extension_group.h"

#include "include_internal/ten_runtime/addon/addon.h"
#include "include_internal/ten_runtime/addon/common/store.h"
#include "include_internal/ten_runtime/engine/engine.h"
#include "include_internal/ten_runtime/extension_group/extension_group.h"
#include "include_internal/ten_runtime/ten_env/ten_env.h"
#include "ten_utils/macro/check.h"

static ten_addon_store_t g_extension_group_store = {
    false,
    NULL,
    TEN_LIST_INIT_VAL,
};

ten_addon_store_t *ten_extension_group_get_global_store(void) {
  ten_addon_store_init(&g_extension_group_store);
  return &g_extension_group_store;
}

ten_addon_host_t *ten_addon_register_extension_group(const char *name,
                                                     const char *base_dir,
                                                     ten_addon_t *addon,
                                                     void *register_ctx) {
  if (!name || strlen(name) == 0) {
    TEN_LOGE("The addon name is required.");
    exit(EXIT_FAILURE);
  }

  ten_addon_host_t *addon_host =
      ten_addon_host_create(TEN_ADDON_TYPE_EXTENSION_GROUP);
  TEN_ASSERT(addon_host, "Should not happen.");

  ten_addon_register(ten_extension_group_get_global_store(), addon_host, name,
                     base_dir, addon);

  return addon_host;
}

ten_addon_t *ten_addon_unregister_extension_group(const char *name) {
  TEN_ASSERT(name, "Should not happen.");

  return ten_addon_unregister(ten_extension_group_get_global_store(), name);
}

bool ten_addon_create_extension_group(
    ten_env_t *ten_env, const char *addon_name, const char *instance_name,
    ten_env_addon_create_instance_done_cb_t cb, void *user_data) {
  TEN_ASSERT(addon_name && instance_name, "Should not happen.");
  TEN_ASSERT(ten_env && ten_env_check_integrity(ten_env, true),
             "Should not happen.");
  TEN_ASSERT(ten_env->attach_to == TEN_ENV_ATTACH_TO_ENGINE,
             "Should not happen.");

  // Because there might be more than one engine (threads) to create extension
  // groups from the corresponding extension group addons simultaneously. So we
  // can _not_ save the pointers of 'cb' and 'user_data' into 'ten_env', they
  // will override each other. Instead we need to pass those pointers through
  // parameters below.

  ten_engine_t *engine = ten_env_get_attached_engine(ten_env);
  TEN_ASSERT(engine && ten_engine_check_integrity(engine, true),
             "Should not happen.");

  return ten_addon_create_instance_async(ten_env, addon_name, instance_name,
                                         TEN_ADDON_TYPE_EXTENSION_GROUP, cb,
                                         user_data);
}

bool ten_addon_destroy_extension_group(
    ten_env_t *ten_env, ten_extension_group_t *extension_group,
    ten_env_addon_destroy_instance_done_cb_t cb, void *cb_data) {
  TEN_ASSERT(ten_env && ten_env_check_integrity(ten_env, true) && cb,
             "Should not happen.");
  TEN_ASSERT(extension_group &&
                 // TEN_NOLINTNEXTLINE(thread-check)
                 // thread-check: The extension thread has been stopped.
                 ten_extension_group_check_integrity(extension_group, false),
             "Should not happen.");
  TEN_ASSERT(ten_env->attach_to == TEN_ENV_ATTACH_TO_ENGINE,
             "Should not happen.");

  ten_engine_t *engine = ten_env_get_attached_engine(ten_env);
  TEN_ASSERT(engine && ten_engine_check_integrity(engine, true),
             "Should not happen.");

  ten_addon_host_t *addon_host = extension_group->addon_host;
  TEN_ASSERT(addon_host,
             "Should not happen, otherwise, memory leakage will occur.");

  return ten_addon_host_destroy_instance_async(addon_host, ten_env,
                                               extension_group, cb, cb_data);
}
