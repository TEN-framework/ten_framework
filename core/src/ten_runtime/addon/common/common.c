//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#include "include_internal/ten_runtime/addon/common/common.h"

#include <stdlib.h>
#include <string.h>

#include "include_internal/ten_runtime/addon/addon_host.h"
#include "include_internal/ten_runtime/addon/addon_loader/addon_loader.h"
#include "include_internal/ten_runtime/addon/common/store.h"
#include "include_internal/ten_runtime/addon/extension/extension.h"
#include "include_internal/ten_runtime/addon/extension_group/extension_group.h"
#include "include_internal/ten_runtime/addon/protocol/protocol.h"
#include "ten_runtime/addon/addon.h"
#include "ten_utils/macro/check.h"

ten_addon_host_t *ten_addon_store_find_by_type(TEN_ADDON_TYPE addon_type,
                                               const char *addon_name) {
  TEN_ASSERT(addon_name, "Should not happen.");

  switch (addon_type) {
    case TEN_ADDON_TYPE_EXTENSION:
      return ten_addon_store_find(ten_extension_get_global_store(), addon_name);

    case TEN_ADDON_TYPE_EXTENSION_GROUP:
      return ten_addon_store_find(ten_extension_group_get_global_store(),
                                  addon_name);

    case TEN_ADDON_TYPE_PROTOCOL:
      return ten_addon_store_find(ten_protocol_get_global_store(), addon_name);

    case TEN_ADDON_TYPE_ADDON_LOADER:
      return ten_addon_store_find(ten_addon_loader_get_global_store(),
                                  addon_name);

    default:
      TEN_ASSERT(0, "Should not happen.");
      break;
  }

  return NULL;
}

int ten_addon_store_lock_by_type(TEN_ADDON_TYPE addon_type) {
  switch (addon_type) {
    case TEN_ADDON_TYPE_EXTENSION:
      return ten_addon_store_lock(ten_extension_get_global_store());

    case TEN_ADDON_TYPE_EXTENSION_GROUP:
      return ten_addon_store_lock(ten_extension_group_get_global_store());

    case TEN_ADDON_TYPE_PROTOCOL:
      return ten_addon_store_lock(ten_protocol_get_global_store());

    case TEN_ADDON_TYPE_ADDON_LOADER:
      return ten_addon_store_lock(ten_addon_loader_get_global_store());

    default:
      TEN_ASSERT(0, "Should not happen.");
      return -1;
  }
}

int ten_addon_store_lock_all_type(void) {
  int rc = ten_addon_store_lock_by_type(TEN_ADDON_TYPE_ADDON_LOADER);
  TEN_ASSERT(!rc, "Should not happen.");

  rc = ten_addon_store_lock_by_type(TEN_ADDON_TYPE_EXTENSION);
  TEN_ASSERT(!rc, "Should not happen.");

  rc = ten_addon_store_lock_by_type(TEN_ADDON_TYPE_EXTENSION_GROUP);
  TEN_ASSERT(!rc, "Should not happen.");

  rc = ten_addon_store_lock_by_type(TEN_ADDON_TYPE_PROTOCOL);
  TEN_ASSERT(!rc, "Should not happen.");

  return rc;
}

int ten_addon_store_unlock_by_type(TEN_ADDON_TYPE addon_type) {
  switch (addon_type) {
    case TEN_ADDON_TYPE_EXTENSION:
      return ten_addon_store_unlock(ten_extension_get_global_store());

    case TEN_ADDON_TYPE_EXTENSION_GROUP:
      return ten_addon_store_unlock(ten_extension_group_get_global_store());

    case TEN_ADDON_TYPE_PROTOCOL:
      return ten_addon_store_unlock(ten_protocol_get_global_store());

    case TEN_ADDON_TYPE_ADDON_LOADER:
      return ten_addon_store_unlock(ten_addon_loader_get_global_store());

    default:
      TEN_ASSERT(0, "Should not happen.");
      return -1;
  }
}

int ten_addon_store_unlock_all_type(void) {
  int rc = ten_addon_store_unlock_by_type(TEN_ADDON_TYPE_ADDON_LOADER);
  TEN_ASSERT(!rc, "Should not happen.");

  rc = ten_addon_store_unlock_by_type(TEN_ADDON_TYPE_EXTENSION);
  TEN_ASSERT(!rc, "Should not happen.");

  rc = ten_addon_store_unlock_by_type(TEN_ADDON_TYPE_EXTENSION_GROUP);
  TEN_ASSERT(!rc, "Should not happen.");

  rc = ten_addon_store_unlock_by_type(TEN_ADDON_TYPE_PROTOCOL);
  TEN_ASSERT(!rc, "Should not happen.");

  return rc;
}
