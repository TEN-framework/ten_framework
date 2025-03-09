//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#pragma once

#include "ten_runtime/ten_config.h"

#include "include_internal/ten_runtime/addon/addon_loader/addon_loader.h"
#include "include_internal/ten_runtime/binding/common.h"
#include "ten_runtime/addon/addon.h"
#include "ten_utils/lib/mutex.h"
#include "ten_utils/lib/signature.h"

#define TEN_ADDON_LOADER_SIGNATURE 0xAE4FCDE7983727E4U

typedef struct ten_addon_loader_t ten_addon_loader_t;
typedef struct ten_addon_host_t ten_addon_host_t;

typedef struct ten_addon_loader_singleton_store_t {
  ten_atomic_t valid;
  ten_mutex_t *lock;
  ten_list_t store;  // ten_addon_loader_t
} ten_addon_loader_singleton_store_t;

typedef void (*ten_addon_loader_on_init_func_t)(
    ten_addon_loader_t *addon_loader);

typedef void (*ten_addon_loader_on_deinit_func_t)(
    ten_addon_loader_t *addon_loader);

typedef void (*ten_addon_loader_on_load_addon_func_t)(
    ten_addon_loader_t *addon_loader, TEN_ADDON_TYPE addon_type,
    const char *addon_name);

typedef void (*ten_addon_loader_on_init_done_cb_t)(void *from, void *cb_data);

typedef void (*ten_addon_loader_on_deinit_done_cb_t)(void *from, void *cb_data);

typedef struct ten_addon_loader_t {
  ten_binding_handle_t binding_handle;
  ten_signature_t signature;

  ten_addon_host_t *addon_host;

  ten_addon_loader_on_init_func_t on_init;
  ten_addon_loader_on_deinit_func_t on_deinit;
  ten_addon_loader_on_load_addon_func_t on_load_addon;

  ten_addon_loader_on_init_done_cb_t on_init_done_cb;
  ten_addon_loader_on_init_done_ctx_t *on_init_done_cb_data;

  ten_addon_loader_on_deinit_done_cb_t on_deinit_done_cb;
  ten_addon_loader_on_deinit_done_ctx_t *on_deinit_done_cb_data;
} ten_addon_loader_t;

TEN_RUNTIME_PRIVATE_API int ten_addon_loader_singleton_store_lock(void);

TEN_RUNTIME_PRIVATE_API int ten_addon_loader_singleton_store_unlock(void);

TEN_RUNTIME_PRIVATE_API ten_list_t *ten_addon_loader_singleton_get_all(void);

TEN_RUNTIME_API ten_addon_loader_t *ten_addon_loader_create(
    ten_addon_loader_on_init_func_t on_init,
    ten_addon_loader_on_deinit_func_t on_deinit,
    ten_addon_loader_on_load_addon_func_t on_load_addon);

TEN_RUNTIME_API void ten_addon_loader_destroy(ten_addon_loader_t *self);

TEN_RUNTIME_PRIVATE_API void ten_addon_loader_load_addon(
    ten_addon_loader_t *self, TEN_ADDON_TYPE addon_type,
    const char *addon_name);
