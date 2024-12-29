//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#include "include_internal/ten_runtime/addon/lang_addon_loader/lang_addon_loader.h"

#include "include_internal/ten_runtime/addon/addon.h"
#include "include_internal/ten_runtime/addon/common/store.h"
#include "include_internal/ten_runtime/common/constant_str.h"
#include "ten_utils/container/list.h"
#include "ten_utils/container/list_node.h"
#include "ten_utils/lib/string.h"
#include "ten_utils/lib/uri.h"
#include "ten_utils/macro/check.h"
#include "ten_utils/value/value.h"
#include "ten_utils/value/value_get.h"
#include "ten_utils/value/value_object.h"

static ten_addon_store_t g_lang_addon_loader_store = {
    false,
    NULL,
    TEN_LIST_INIT_VAL,
};

ten_addon_store_t *ten_lang_addon_loader_get_global_store(void) {
  ten_addon_store_init(&g_lang_addon_loader_store);
  return &g_lang_addon_loader_store;
}

ten_addon_t *ten_addon_unregister_lang_addon_loader(const char *name) {
  TEN_ASSERT(name, "Should not happen.");

  return ten_addon_unregister(ten_lang_addon_loader_get_global_store(), name);
}

ten_addon_host_t *ten_addon_register_lang_addon_loader(const char *name,
                                                       const char *base_dir,
                                                       ten_addon_t *addon,
                                                       void *register_ctx) {
  return ten_addon_register(TEN_ADDON_TYPE_LANG_ADDON_LOADER, name, base_dir,
                            addon, register_ctx);
}

static bool ten_addon_lang_addon_loader_match_lang_addon_loader(
    ten_addon_host_t *self, const char *lang_addon_loader) {
  TEN_ASSERT(self && self->type == TEN_ADDON_TYPE_PROTOCOL && lang_addon_loader,
             "Should not happen.");

  ten_value_t *manifest = &self->manifest;
  TEN_ASSERT(manifest, "Invalid argument.");
  TEN_ASSERT(ten_value_check_integrity(manifest), "Invalid use of manifest %p.",
             manifest);

  bool found = false;
  ten_list_t *addon_lang_addon_loaders =
      ten_value_object_peek_array(manifest, TEN_STR_PROTOCOL);
  TEN_ASSERT(addon_lang_addon_loaders, "Should not happen.");

  ten_list_foreach (addon_lang_addon_loaders, iter) {
    ten_value_t *addon_lang_addon_loader_value =
        ten_ptr_listnode_get(iter.node);
    TEN_ASSERT(addon_lang_addon_loader_value &&
                   ten_value_check_integrity(addon_lang_addon_loader_value),
               "Should not happen.");

    const char *addon_lang_addon_loader =
        ten_value_peek_raw_str(addon_lang_addon_loader_value, NULL);
    if (!strcmp(addon_lang_addon_loader, lang_addon_loader)) {
      found = true;
      break;
    }
  }

  return found;
}

ten_addon_host_t *ten_addon_lang_addon_loader_find(
    const char *lang_addon_loader) {
  TEN_ASSERT(lang_addon_loader, "Should not happen.");

  ten_addon_host_t *result = NULL;

  ten_addon_store_t *store = ten_lang_addon_loader_get_global_store();
  TEN_ASSERT(store, "Should not happen.");

  ten_mutex_lock(store->lock);

  ten_list_foreach (&store->store, iter) {
    ten_addon_host_t *addon = ten_ptr_listnode_get(iter.node);
    TEN_ASSERT(addon && addon->type == TEN_ADDON_TYPE_PROTOCOL,
               "Should not happen.");

    if (!ten_addon_lang_addon_loader_match_lang_addon_loader(
            addon, lang_addon_loader)) {
      continue;
    }

    result = addon;
    break;
  }

  ten_mutex_unlock(store->lock);

  return result;
}

void ten_addon_unregister_all_lang_addon_loader(void) {
  ten_addon_store_del_all(ten_lang_addon_loader_get_global_store());
}
