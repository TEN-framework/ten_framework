//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#include "include_internal/ten_runtime/addon/common/store.h"

#include <stdlib.h>

#include "include_internal/ten_runtime/addon/addon.h"
#include "include_internal/ten_runtime/addon/addon_host.h"
#include "include_internal/ten_runtime/addon/addon_loader/addon_loader.h"
#include "include_internal/ten_runtime/addon/extension/extension.h"
#include "include_internal/ten_runtime/addon/extension_group/extension_group.h"
#include "include_internal/ten_runtime/addon/protocol/protocol.h"
#include "ten_utils/container/list.h"
#include "ten_utils/lib/mutex.h"
#include "ten_utils/lib/ref.h"
#include "ten_utils/lib/string.h"
#include "ten_utils/macro/check.h"
#include "ten_utils/macro/memory.h"

// The addon store should be initted only once.
void ten_addon_store_init(ten_addon_store_t *store) {
  TEN_ASSERT(store, "Can not init empty addon store.");
  TEN_ASSERT(!store->lock, "Should not happen.");

  store->lock = ten_mutex_create();
  TEN_ASSERT(store->lock, "Failed to create mutex.");

  ten_list_init(&store->store);
}

static void ten_addon_host_remove_from_store(ten_addon_host_t *addon_host) {
  TEN_ASSERT(addon_host, "Invalid argument.");

  ten_ref_dec_ref(&addon_host->ref);
}

ten_addon_host_t *ten_addon_store_find(ten_addon_store_t *store,
                                       const char *name) {
  TEN_ASSERT(store, "Invalid argument.");
  TEN_ASSERT(name, "Invalid argument.");

  ten_addon_host_t *result = NULL;

  ten_list_foreach(&store->store, iter) {
    ten_addon_host_t *addon = ten_ptr_listnode_get(iter.node);
    TEN_ASSERT(addon, "Should not happen.");

    if (ten_string_is_equal_c_str(&addon->name, name)) {
      result = addon;
      break;
    }
  }

  return result;
}

void ten_addon_store_add(ten_addon_store_t *store, ten_addon_host_t *addon) {
  TEN_ASSERT(store, "Invalid argument.");
  TEN_ASSERT(addon, "Invalid argument.");

  ten_list_push_ptr_back(
      &store->store, addon,
      (ten_ptr_listnode_destroy_func_t)ten_addon_host_remove_from_store);
}

ten_addon_t *ten_addon_store_del(ten_addon_store_t *store, const char *name) {
  TEN_ASSERT(store, "Invalid argument.");
  TEN_ASSERT(name, "Invalid argument.");

  ten_addon_t *addon = NULL;

  ten_mutex_lock(store->lock);

  ten_list_foreach(&store->store, iter) {
    ten_addon_host_t *addon_host = ten_ptr_listnode_get(iter.node);
    TEN_ASSERT(addon_host, "Should not happen.");

    if (ten_string_is_equal_c_str(&addon_host->name, name)) {
      addon = addon_host->addon;
      ten_list_remove_node(&store->store, iter.node);
      break;
    }
  }

  ten_mutex_unlock(store->lock);

  return addon;
}

void ten_addon_store_del_all(ten_addon_store_t *store) {
  TEN_ASSERT(store, "Invalid argument.");

  ten_mutex_lock(store->lock);

  // Clear the store's list, which will call the destroy function for each node,
  // properly decreasing the refcount of each addon.
  ten_list_clear(&store->store);

  ten_mutex_unlock(store->lock);
}

static void ten_addon_on_addon_deinit_done(ten_env_t *ten_env, void *cb_data) {
  ten_addon_store_on_all_addons_deinit_done_ctx_t *ctx = cb_data;
  ten_addon_store_t *store = ctx->store;

  if (ten_atomic_sub_fetch(&ctx->deiniting_count, 1) == 0) {
    ctx->cb(store, ctx->cb_data);
    TEN_FREE(ctx);
  }
}

void ten_addon_store_del_all_ex(
    ten_addon_store_t *store, ten_addon_store_on_all_addons_deinit_done_cb_t cb,
    void *cb_data) {
  TEN_ASSERT(store, "Invalid argument.");

  ten_addon_store_on_all_addons_deinit_done_ctx_t *ctx =
      TEN_MALLOC(sizeof(ten_addon_store_on_all_addons_deinit_done_ctx_t));
  TEN_ASSERT(ctx, "Failed to allocate memory.");

  ctx->store = store;
  ctx->cb = cb;
  ctx->cb_data = cb_data;

  ten_mutex_lock(store->lock);

  ten_atomic_store(&ctx->deiniting_count,
                   (int64_t)ten_list_size(&store->store));

  if (ten_list_size(&store->store) == 0) {
    ten_mutex_unlock(store->lock);
    TEN_FREE(ctx);
    cb(store, cb_data);
    return;
  }

  ten_list_foreach(&store->store, iter) {
    ten_addon_host_t *addon_host = ten_ptr_listnode_get(iter.node);
    TEN_ASSERT(addon_host, "Should not happen.");

    addon_host->on_deinit_done_cb = ten_addon_on_addon_deinit_done;
    addon_host->on_deinit_done_cb_data = ctx;
  }

  ten_list_clear(&store->store);

  ten_mutex_unlock(store->lock);
}

int ten_addon_store_lock(ten_addon_store_t *store) {
  TEN_ASSERT(store, "Invalid argument.");
  return ten_mutex_lock(store->lock);
}

int ten_addon_store_unlock(ten_addon_store_t *store) {
  TEN_ASSERT(store, "Invalid argument.");
  return ten_mutex_unlock(store->lock);
}
