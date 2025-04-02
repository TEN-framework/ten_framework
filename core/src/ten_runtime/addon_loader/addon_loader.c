//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#include "include_internal/ten_runtime/addon_loader/addon_loader.h"

#include "include_internal/ten_runtime/addon/addon_host.h"
#include "include_internal/ten_runtime/addon/addon_loader/addon_loader.h"
#include "include_internal/ten_runtime/addon/common/common.h"
#include "include_internal/ten_runtime/app/app.h"
#include "include_internal/ten_runtime/app/on_xxx.h"
#include "include_internal/ten_runtime/ten_env/ten_env.h"
#include "ten_runtime/app/app.h"
#include "ten_runtime/ten_env/ten_env.h"
#include "ten_utils/container/list.h"
#include "ten_utils/io/runloop.h"
#include "ten_utils/lib/signature.h"
#include "ten_utils/macro/memory.h"

static ten_addon_loader_singleton_store_t g_addon_loader_singleton_store = {
    false,
    NULL,
    TEN_LIST_INIT_VAL,
};

static void ten_addon_loader_singleton_store_init(
    ten_addon_loader_singleton_store_t *store) {
  TEN_ASSERT(store, "Can not init empty addon store.");

  // The addon store should be initted only once.
  if (ten_atomic_test_set(&store->valid, 1) == 1) {
    return;
  }

  store->lock = ten_mutex_create();
  ten_list_init(&store->store);
}

static ten_addon_loader_singleton_store_t *
ten_addon_loader_singleton_get_global_store(void) {
  ten_addon_loader_singleton_store_init(&g_addon_loader_singleton_store);
  return &g_addon_loader_singleton_store;
}

ten_list_t *ten_addon_loader_singleton_get_all(void) {
  return &ten_addon_loader_singleton_get_global_store()->store;
}

int ten_addon_loader_singleton_store_lock(void) {
  return ten_mutex_lock(ten_addon_loader_singleton_get_global_store()->lock);
}

int ten_addon_loader_singleton_store_unlock(void) {
  return ten_mutex_unlock(ten_addon_loader_singleton_get_global_store()->lock);
}

bool ten_addon_loader_check_integrity(ten_addon_loader_t *self) {
  TEN_ASSERT(self, "Should not happen.");

  if (ten_signature_get(&self->signature) !=
      (ten_signature_t)TEN_ADDON_LOADER_SIGNATURE) {
    return false;
  }

  return true;
}

ten_env_t *ten_addon_loader_get_ten_env(ten_addon_loader_t *self) {
  TEN_ASSERT(self, "Invalid argument.");
  TEN_ASSERT(ten_addon_loader_check_integrity(self), "Invalid argument.");

  return self->ten_env;
}

ten_addon_loader_t *ten_addon_loader_create(
    ten_addon_loader_on_init_func_t on_init,
    ten_addon_loader_on_deinit_func_t on_deinit,
    ten_addon_loader_on_load_addon_func_t on_load_addon) {
  ten_addon_loader_t *self =
      (ten_addon_loader_t *)TEN_MALLOC(sizeof(ten_addon_loader_t));
  TEN_ASSERT(self, "Failed to allocate memory.");

  ten_signature_set(&self->signature, TEN_ADDON_LOADER_SIGNATURE);

  self->addon_host = NULL;

  self->on_init = on_init;
  self->on_deinit = on_deinit;
  self->on_load_addon = on_load_addon;

  self->on_init_done_cb = NULL;
  self->on_init_done_cb_data = NULL;

  self->ten_env = ten_env_create_for_addon_loader(self);

  return self;
}

void ten_addon_loader_destroy(ten_addon_loader_t *self) {
  TEN_ASSERT(self, "Invalid argument.");
  TEN_ASSERT(ten_addon_loader_check_integrity(self), "Invalid argument.");
  TEN_ASSERT(self->ten_env, "Should not happen.");

  ten_signature_set(&self->signature, 0);

  ten_env_destroy(self->ten_env);

  TEN_FREE(self);
}

static void ten_addon_loader_init(ten_addon_loader_t *self,
                                  ten_addon_loader_on_init_done_cb_t cb,
                                  void *cb_data) {
  TEN_ASSERT(self, "Invalid argument.");
  TEN_ASSERT(ten_addon_loader_check_integrity(self), "Invalid argument.");

  self->on_init_done_cb = cb;
  self->on_init_done_cb_data = cb_data;

  if (self->on_init) {
    self->on_init(self, self->ten_env);
  } else {
    ten_env_on_init_done(self->ten_env, NULL);
  }
}

static void ten_addon_loader_deinit(ten_addon_loader_t *self,
                                    ten_addon_loader_on_deinit_done_cb_t cb,
                                    void *cb_data) {
  TEN_ASSERT(self, "Invalid argument.");
  TEN_ASSERT(ten_addon_loader_check_integrity(self), "Invalid argument.");

  self->on_deinit_done_cb = cb;
  self->on_deinit_done_cb_data = cb_data;

  if (self->on_deinit) {
    self->on_deinit(self, self->ten_env);
  } else {
    ten_env_on_deinit_done(self->ten_env, NULL);
  }
}

void ten_addon_loader_load_addon(ten_addon_loader_t *self,
                                 TEN_ADDON_TYPE addon_type,
                                 const char *addon_name) {
  TEN_ASSERT(self, "Invalid argument.");
  TEN_ASSERT(ten_addon_loader_check_integrity(self), "Invalid argument.");

  if (self->on_load_addon) {
    self->on_load_addon(self, self->ten_env, addon_type, addon_name);
  }
}

static void ten_app_thread_on_addon_loader_init_done(void *app, void *cb_data) {
  TEN_ASSERT(app, "Invalid argument.");
  TEN_ASSERT(ten_app_check_integrity(app, true), "Invalid argument.");

  ten_app_on_addon_loader_init_done_ctx_t *ctx =
      (ten_app_on_addon_loader_init_done_ctx_t *)cb_data;
  TEN_ASSERT(ctx, "Invalid argument.");

  ten_addon_loader_on_all_singleton_instances_created_ctx_t
      *on_all_created_ctx = ctx->cb_data;
  TEN_ASSERT(on_all_created_ctx, "Invalid argument.");

  size_t desired_count = on_all_created_ctx->desired_count;

  ten_list_push_ptr_back(ten_addon_loader_singleton_get_all(),
                         ctx->addon_loader, NULL);

  TEN_FREE(ctx);

  if (ten_list_size(ten_addon_loader_singleton_get_all()) == desired_count) {
    // Call the callback to notify all addon_loader instances have been created.
    on_all_created_ctx->cb(on_all_created_ctx->ten_env,
                           on_all_created_ctx->cb_data);

    TEN_FREE(on_all_created_ctx);
  }
}

static void ten_app_thread_on_addon_loader_deinit_done(void *app,
                                                       void *cb_data) {
  TEN_ASSERT(app, "Invalid argument.");
  TEN_ASSERT(ten_app_check_integrity(app, true), "Invalid argument.");

  ten_app_on_addon_loader_deinit_done_ctx_t *ctx =
      (ten_app_on_addon_loader_deinit_done_ctx_t *)cb_data;
  TEN_ASSERT(ctx, "Invalid argument.");

  ten_addon_loader_on_all_singleton_instances_destroyed_ctx_t
      *on_all_destroyed_ctx = ctx->cb_data;
  TEN_ASSERT(on_all_destroyed_ctx, "Invalid argument.");

  ten_addon_loader_t *addon_loader = ctx->addon_loader;
  TEN_ASSERT(addon_loader, "Invalid argument.");
  TEN_ASSERT(ten_addon_loader_check_integrity(addon_loader),
             "Invalid argument.");

  ten_addon_host_t *addon_host = addon_loader->addon_host;
  TEN_ASSERT(addon_host, "Should not happen.");

  addon_host->addon->on_destroy_instance(addon_host->addon, addon_host->ten_env,
                                         addon_loader, NULL);

  ten_list_remove_ptr(ten_addon_loader_singleton_get_all(), addon_loader);

  if (ten_list_size(ten_addon_loader_singleton_get_all()) == 0) {
    int lock_operation_rc = ten_addon_loader_singleton_store_unlock();
    TEN_ASSERT(!lock_operation_rc, "Should not happen.");

    if (on_all_destroyed_ctx->cb) {
      on_all_destroyed_ctx->cb(on_all_destroyed_ctx->ten_env,
                               on_all_destroyed_ctx->cb_data);
    }

    TEN_FREE(on_all_destroyed_ctx);
  }

  TEN_FREE(ctx);
}

static void ten_addon_loader_init_done(ten_env_t *ten_env, void *cb_data) {
  TEN_ASSERT(ten_env, "Invalid argument.");
  TEN_ASSERT(ten_env_check_integrity(ten_env, true), "Invalid argument.");
  TEN_ASSERT(cb_data, "Invalid argument.");

  ten_addon_loader_t *addon_loader = ten_env_get_attached_addon_loader(ten_env);
  TEN_ASSERT(addon_loader, "Should not happen.");
  TEN_ASSERT(ten_addon_loader_check_integrity(addon_loader),
             "Should not happen.");

  ten_addon_loader_on_all_singleton_instances_created_ctx_t *ctx =
      (ten_addon_loader_on_all_singleton_instances_created_ctx_t *)cb_data;
  TEN_ASSERT(ctx, "Invalid argument.");

  ten_env_t *caller_ten_env = ctx->ten_env;
  TEN_ASSERT(caller_ten_env, "Invalid argument.");
  // TEN_NOLINTNEXTLINE(thread-check)
  // thread-check: This function is intended to be called in any threads.
  TEN_ASSERT(ten_env_check_integrity(caller_ten_env, false),
             "Invalid argument.");

  TEN_ASSERT(caller_ten_env->attach_to == TEN_ENV_ATTACH_TO_APP,
             "Should not happen.");

  ten_app_t *app = ten_env_get_attached_app(caller_ten_env);
  TEN_ASSERT(app, "Invalid argument.");
  // TEN_NOLINTNEXTLINE(thread-check)
  // thread-check: This function is intended to be called in any threads.
  TEN_ASSERT(ten_app_check_integrity(app, false), "Invalid argument.");

  ten_app_on_addon_loader_init_done_ctx_t *on_addon_loader_init_done_ctx =
      TEN_MALLOC(sizeof(ten_app_on_addon_loader_init_done_ctx_t));
  TEN_ASSERT(on_addon_loader_init_done_ctx, "Should not happen.");

  on_addon_loader_init_done_ctx->addon_loader = addon_loader;
  on_addon_loader_init_done_ctx->cb_data = ctx;

  int rc = ten_runloop_post_task_tail(ten_app_get_attached_runloop(app),
                                      ten_app_thread_on_addon_loader_init_done,
                                      app, on_addon_loader_init_done_ctx);
  if (rc) {
    TEN_LOGE("Failed to post task to app runloop.");
    TEN_ASSERT(0, "Should not happen.");
  }
}

static void create_addon_loader_done(ten_env_t *ten_env,
                                     ten_addon_loader_t *addon_loader,
                                     void *cb_data) {
  TEN_ASSERT(ten_env, "Invalid argument.");
  TEN_ASSERT(ten_env_check_integrity(ten_env, true), "Invalid argument.");
  TEN_ASSERT(addon_loader && ten_addon_loader_check_integrity(addon_loader),
             "Invalid argument.");
  TEN_ASSERT(ten_env_get_attach_to(ten_env) == TEN_ENV_ATTACH_TO_APP,
             "Invalid argument.");

  ten_app_t *app = ten_env_get_attached_target(ten_env);
  TEN_ASSERT(app, "Invalid argument.");
  TEN_ASSERT(ten_app_check_integrity(app, true), "Invalid argument.");

  // Call `on_init` of the addon_loader to initialize the addon_loader.
  ten_addon_loader_init(addon_loader, ten_addon_loader_init_done, cb_data);
}

static ten_addon_loader_on_all_singleton_instances_created_ctx_t *
ten_addon_loader_on_all_singleton_instances_created_ctx_create(
    ten_env_t *ten_env, size_t desired_count,
    ten_addon_loader_on_all_singleton_instances_created_cb_t cb,
    void *cb_data) {
  ten_addon_loader_on_all_singleton_instances_created_ctx_t *ctx = TEN_MALLOC(
      sizeof(ten_addon_loader_on_all_singleton_instances_created_ctx_t));
  TEN_ASSERT(ctx, "Should not happen.");

  ctx->ten_env = ten_env;
  ctx->desired_count = desired_count;
  ctx->cb = cb;
  ctx->cb_data = cb_data;

  return ctx;
}

static ten_addon_loader_on_all_singleton_instances_destroyed_ctx_t *
ten_addon_loader_on_all_singleton_instances_destroyed_ctx_create(
    ten_env_t *ten_env,
    ten_addon_loader_on_all_singleton_instances_destroyed_cb_t cb,
    void *cb_data) {
  ten_addon_loader_on_all_singleton_instances_destroyed_ctx_t *ctx = TEN_MALLOC(
      sizeof(ten_addon_loader_on_all_singleton_instances_destroyed_ctx_t));
  TEN_ASSERT(ctx, "Failed to allocate memory.");

  ctx->ten_env = ten_env;
  ctx->cb = cb;
  ctx->cb_data = cb_data;

  return ctx;
}

void ten_addon_loader_addons_create_singleton_instance(
    ten_env_t *ten_env,
    ten_addon_loader_on_all_singleton_instances_created_cb_t cb,
    void *cb_data) {
  bool need_to_wait_all_addon_loaders_created = true;

  int lock_operation_rc = ten_addon_loader_singleton_store_lock();
  TEN_ASSERT(!lock_operation_rc, "Should not happen.");

  ten_addon_store_t *addon_loader_store = ten_addon_loader_get_global_store();
  TEN_ASSERT(addon_loader_store, "Should not happen.");

  size_t desired_count = ten_list_size(&addon_loader_store->store);
  if (!desired_count ||
      ten_list_size(ten_addon_loader_singleton_get_all()) == desired_count) {
    need_to_wait_all_addon_loaders_created = false;
    goto done;
  }

  ten_addon_loader_on_all_singleton_instances_created_ctx_t *ctx =
      ten_addon_loader_on_all_singleton_instances_created_ctx_create(
          ten_env, desired_count, cb, cb_data);
  TEN_ASSERT(ctx, "Should not happen.");

  ten_error_t err;
  TEN_ERROR_INIT(err);

  ten_list_foreach (&addon_loader_store->store, iter) {
    ten_addon_host_t *loader_addon_host = ten_ptr_listnode_get(iter.node);
    TEN_ASSERT(loader_addon_host, "Should not happen.");

    bool res = ten_addon_create_addon_loader(
        ten_env, ten_string_get_raw_str(&loader_addon_host->name),
        ten_string_get_raw_str(&loader_addon_host->name),
        (ten_env_addon_create_instance_done_cb_t)create_addon_loader_done, ctx,
        &err);

    if (!res) {
      TEN_LOGE("Failed to create addon_loader instance %s, %s.",
               ten_string_get_raw_str(&loader_addon_host->name),
               ten_error_message(&err));
#if defined(_DEBUG)
      TEN_ASSERT(0, "Should not happen.");
#endif
    }
  }

  ten_error_deinit(&err);

done:
  if (!need_to_wait_all_addon_loaders_created && cb) {
    cb(ten_env, cb_data);
  }
}

static void ten_addon_loader_deinit_done(ten_env_t *ten_env, void *cb_data) {
  TEN_ASSERT(ten_env, "Invalid argument.");
  TEN_ASSERT(ten_env_check_integrity(ten_env, true), "Invalid argument.");
  TEN_ASSERT(cb_data, "Invalid argument.");

  ten_addon_loader_t *addon_loader = ten_env_get_attached_addon_loader(ten_env);
  TEN_ASSERT(addon_loader, "Should not happen.");
  TEN_ASSERT(ten_addon_loader_check_integrity(addon_loader),
             "Should not happen.");

  ten_addon_loader_on_all_singleton_instances_destroyed_ctx_t *ctx =
      (ten_addon_loader_on_all_singleton_instances_destroyed_ctx_t *)cb_data;
  TEN_ASSERT(ctx, "Invalid argument.");

  ten_env_t *caller_ten_env = ctx->ten_env;
  TEN_ASSERT(caller_ten_env, "Invalid argument.");
  // TEN_NOLINTNEXTLINE(thread-check)
  // thread-check: This function is intended to be called in any threads.
  TEN_ASSERT(ten_env_check_integrity(caller_ten_env, false),
             "Invalid argument.");

  TEN_ASSERT(caller_ten_env->attach_to == TEN_ENV_ATTACH_TO_APP,
             "Should not happen.");

  ten_app_t *app = ten_env_get_attached_app(caller_ten_env);
  TEN_ASSERT(app, "Invalid argument.");
  // TEN_NOLINTNEXTLINE(thread-check)
  // thread-check: This function is intended to be called in any threads.
  TEN_ASSERT(ten_app_check_integrity(app, false), "Invalid argument.");

  ten_app_on_addon_loader_deinit_done_ctx_t *on_addon_loader_deinit_done_ctx =
      TEN_MALLOC(sizeof(ten_app_on_addon_loader_deinit_done_ctx_t));
  TEN_ASSERT(on_addon_loader_deinit_done_ctx, "Should not happen.");

  on_addon_loader_deinit_done_ctx->addon_loader = addon_loader;
  on_addon_loader_deinit_done_ctx->cb_data = ctx;

  int rc =
      ten_runloop_post_task_tail(ten_app_get_attached_runloop(app),
                                 ten_app_thread_on_addon_loader_deinit_done,
                                 app, on_addon_loader_deinit_done_ctx);
  if (rc) {
    TEN_LOGE("Failed to post task to app runloop.");
    TEN_ASSERT(0, "Should not happen.");
  }
}

void ten_addon_loader_destroy_all_singleton_instances(
    ten_env_t *ten_env,
    ten_addon_loader_on_all_singleton_instances_destroyed_cb_t cb,
    void *cb_data) {
  TEN_ASSERT(ten_env, "Invalid argument.");
  TEN_ASSERT(ten_env_check_integrity(ten_env, true), "Invalid argument.");

  TEN_ASSERT(ten_env_get_attach_to(ten_env) == TEN_ENV_ATTACH_TO_APP,
             "Should not happen.");

  bool need_to_wait_all_addon_loaders_destroyed = true;

  int lock_operation_rc = ten_addon_loader_singleton_store_lock();
  TEN_ASSERT(!lock_operation_rc, "Should not happen.");

  if (ten_list_size(ten_addon_loader_singleton_get_all()) == 0) {
    need_to_wait_all_addon_loaders_destroyed = false;
    goto done;
  }

  ten_addon_loader_on_all_singleton_instances_destroyed_ctx_t *ctx =
      ten_addon_loader_on_all_singleton_instances_destroyed_ctx_create(
          ten_env, cb, cb_data);
  TEN_ASSERT(ctx, "Should not happen.");

  ten_list_foreach (ten_addon_loader_singleton_get_all(), iter) {
    ten_addon_loader_t *addon_loader = ten_ptr_listnode_get(iter.node);
    TEN_ASSERT(addon_loader, "Should not happen.");
    TEN_ASSERT(ten_addon_loader_check_integrity(addon_loader),
               "Should not happen.");

    ten_addon_host_t *addon_host = addon_loader->addon_host;
    TEN_ASSERT(addon_host, "Should not happen.");
    TEN_ASSERT(ten_addon_host_check_integrity(addon_host),
               "Should not happen.");

    ten_addon_loader_deinit(addon_loader, ten_addon_loader_deinit_done, ctx);
  }

done:
  if (!need_to_wait_all_addon_loaders_destroyed && cb) {
    cb(ten_env, cb_data);
  }
}

// Destroy all addon loader instances in the system.
void ten_addon_loader_destroy_all_singleton_instances_immediately(void) {
  int lock_operation_rc = ten_addon_loader_singleton_store_lock();
  TEN_ASSERT(!lock_operation_rc, "Should not happen.");

  ten_list_foreach (ten_addon_loader_singleton_get_all(), iter) {
    ten_addon_loader_t *addon_loader = ten_ptr_listnode_get(iter.node);
    TEN_ASSERT(addon_loader, "Should not happen.");
    TEN_ASSERT(ten_addon_loader_check_integrity(addon_loader),
               "Should not happen.");

    ten_addon_host_t *addon_host = addon_loader->addon_host;
    TEN_ASSERT(addon_host, "Should not happen.");
    TEN_ASSERT(ten_addon_host_check_integrity(addon_host),
               "Should not happen.");

    ten_addon_loader_deinit(addon_loader, NULL, NULL);

    ten_addon_t *addon = addon_host->addon;
    TEN_ASSERT(addon, "Should not happen.");
    TEN_ASSERT(ten_addon_check_integrity(addon), "Should not happen.");

    addon->on_destroy_instance(addon, addon_host->ten_env, addon_loader, NULL);
  }

  ten_list_clear(ten_addon_loader_singleton_get_all());

  lock_operation_rc = ten_addon_loader_singleton_store_unlock();
  TEN_ASSERT(!lock_operation_rc, "Should not happen.");
}
