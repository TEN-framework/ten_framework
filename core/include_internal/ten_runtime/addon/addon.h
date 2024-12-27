//
// Copyright © 2024 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#pragma once

#include "ten_runtime/ten_config.h"

#include "include_internal/ten_runtime/addon/common/store.h"
#include "include_internal/ten_runtime/binding/common.h"
#include "ten_runtime/addon/addon.h"
#include "ten_runtime/binding/common.h"
#include "ten_runtime/ten_env/ten_env.h"
#include "ten_utils/lib/ref.h"
#include "ten_utils/lib/signature.h"

#define TEN_ADDON_HOST_SIGNATURE 0x44FAE6B3F920A44EU
#define TEN_ADDON_SIGNATURE 0xDB9CA797E07377D4U

typedef struct ten_app_t ten_app_t;

typedef enum TEN_ADDON_TYPE {
  TEN_ADDON_TYPE_INVALID,

  TEN_ADDON_TYPE_PROTOCOL,
  TEN_ADDON_TYPE_EXTENSION,
  TEN_ADDON_TYPE_EXTENSION_GROUP,
} TEN_ADDON_TYPE;

typedef void (*ten_env_addon_create_instance_done_cb_t)(ten_env_t *ten_env,
                                                        void *instance,
                                                        void *cb_data);

typedef void (*ten_env_addon_destroy_instance_done_cb_t)(ten_env_t *ten_env,
                                                         void *cb_data);

typedef struct ten_addon_context_t {
  ten_env_t *caller_ten;

  ten_env_addon_create_instance_done_cb_t create_instance_done_cb;
  void *create_instance_done_cb_data;

  ten_env_addon_destroy_instance_done_cb_t destroy_instance_done_cb;
  void *destroy_instance_done_cb_data;
} ten_addon_context_t;

typedef struct ten_addon_t {
  ten_binding_handle_t binding_handle;

  ten_signature_t signature;

  ten_addon_on_init_func_t on_init;
  ten_addon_on_deinit_func_t on_deinit;

  ten_addon_on_create_instance_func_t on_create_instance;
  ten_addon_on_destroy_instance_func_t on_destroy_instance;

  ten_addon_on_destroy_func_t on_destroy;

  void *user_data;
} ten_addon_t;

typedef struct ten_addon_host_t {
  ten_signature_t signature;

  ten_string_t name;  // The name of the addon.
  ten_string_t base_dir;

  ten_value_t manifest;
  ten_value_t property;

  ten_metadata_info_t *manifest_info;
  ten_metadata_info_t *property_info;

  ten_addon_t *addon;
  ten_addon_store_t *store;

  ten_ref_t ref;  // Used to control the timing of addon destruction.
  ten_env_t *ten_env;

  TEN_ADDON_TYPE type;

  void *user_data;
} ten_addon_host_t;

typedef struct ten_addon_on_create_instance_info_t {
  ten_string_t addon_name;
  ten_string_t instance_name;
  TEN_ADDON_TYPE addon_type;  // Used to retrieve addon from the correct store.
  ten_env_addon_create_instance_done_cb_t cb;
  void *cb_data;
} ten_addon_on_create_instance_info_t;

typedef struct ten_addon_on_destroy_instance_info_t {
  ten_addon_host_t *addon_host;
  void *instance;
  ten_env_addon_destroy_instance_done_cb_t cb;
  void *cb_data;
} ten_addon_on_destroy_instance_info_t;

TEN_RUNTIME_PRIVATE_API bool ten_addon_host_check_integrity(
    ten_addon_host_t *self);

TEN_RUNTIME_API bool ten_addon_check_integrity(ten_addon_t *self);

TEN_RUNTIME_PRIVATE_API void ten_addon_host_init(ten_addon_host_t *self);

TEN_RUNTIME_PRIVATE_API ten_addon_host_t *ten_addon_host_create(
    TEN_ADDON_TYPE type);

TEN_RUNTIME_API void ten_addon_host_destroy(ten_addon_host_t *self);

TEN_RUNTIME_PRIVATE_API TEN_ADDON_TYPE
ten_addon_type_from_string(const char *addon_type_str);

TEN_RUNTIME_PRIVATE_API ten_addon_t *ten_addon_unregister(
    ten_addon_store_t *store, const char *addon_name);

TEN_RUNTIME_PRIVATE_API const char *ten_addon_host_get_name(
    ten_addon_host_t *self);

TEN_RUNTIME_API ten_addon_host_t *ten_addon_host_find(const char *addon_name,
                                                      TEN_ADDON_TYPE type);

TEN_RUNTIME_PRIVATE_API ten_addon_on_create_instance_info_t *
ten_addon_on_create_instance_info_create(
    const char *addon_name, const char *instance_name,
    TEN_ADDON_TYPE addon_type, ten_env_addon_create_instance_done_cb_t cb,
    void *cb_data);

TEN_RUNTIME_PRIVATE_API void ten_addon_on_create_instance_info_destroy(
    ten_addon_on_create_instance_info_t *self);

TEN_RUNTIME_PRIVATE_API ten_addon_on_destroy_instance_info_t *
ten_addon_host_on_destroy_instance_info_create(
    ten_addon_host_t *self, void *instance,
    ten_env_addon_destroy_instance_done_cb_t cb, void *cb_data);

TEN_RUNTIME_PRIVATE_API void ten_addon_on_destroy_instance_info_destroy(
    ten_addon_on_destroy_instance_info_t *self);

TEN_RUNTIME_PRIVATE_API ten_addon_store_t *ten_addon_get_store(void);

TEN_RUNTIME_PRIVATE_API bool ten_addon_create_instance_async(
    ten_env_t *ten_env, const char *addon_name, const char *instance_name,
    TEN_ADDON_TYPE type, ten_env_addon_create_instance_done_cb_t cb,
    void *cb_data);

TEN_RUNTIME_PRIVATE_API bool ten_addon_host_destroy_instance_async(
    ten_addon_host_t *self, ten_env_t *ten_env, void *instance,
    ten_env_addon_destroy_instance_done_cb_t cb, void *cb_data);

TEN_RUNTIME_PRIVATE_API bool ten_addon_host_destroy_instance(
    ten_addon_host_t *self, ten_env_t *ten_env, void *instance);

/**
 * @brief The base directory of the loaded addon. This function can be called
 * before any TEN app starts. Note that the returned string must be destroyed by
 * users.
 */
TEN_RUNTIME_PRIVATE_API const char *ten_addon_host_get_base_dir(
    ten_addon_host_t *self);

TEN_RUNTIME_PRIVATE_API void ten_addon_context_destroy(
    ten_addon_context_t *self);

TEN_RUNTIME_PRIVATE_API ten_string_t *ten_addon_find_base_dir_from_app(
    const char *addon_type, const char *addon_name);

TEN_RUNTIME_PRIVATE_API void ten_addon_find_and_set_base_dir(
    ten_addon_host_t *self, const char *path);

TEN_RUNTIME_PRIVATE_API ten_addon_host_t *ten_addon_register(
    TEN_ADDON_TYPE addon_type, const char *name, const char *base_dir,
    ten_addon_t *addon, void *register_ctx);
