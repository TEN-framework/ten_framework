//
// This file is part of the TEN Framework project.
// See https://github.com/TEN-framework/ten_framework/LICENSE for license
// information.
//
#pragma once

#include "ten_runtime/ten_config.h"

#include "ten_runtime/ten_env/internal/metadata.h"
#include "ten_utils/lib/error.h"
#include "ten_utils/lib/event.h"

typedef struct ten_env_t ten_env_t;
typedef struct ten_value_t ten_value_t;
typedef struct ten_extension_t ten_extension_t;
typedef struct ten_extension_group_t ten_extension_group_t;

typedef struct ten_peek_manifest_sync_context_t {
  ten_value_t *res;
  ten_event_t *completed;
} ten_peek_manifest_sync_context_t;

typedef struct ten_env_peek_property_async_context_t {
  ten_env_t *ten_env;
  ten_env_peek_property_async_cb_t cb;
  void *cb_data;
  ten_value_t *res;

  union {
    ten_extension_t *extension;
    ten_extension_group_t *extension_group;
  } from;
} ten_env_peek_property_async_context_t;

typedef struct ten_env_peek_property_sync_context_t {
  ten_value_t *res;
  ten_event_t *completed;
} ten_env_peek_property_sync_context_t;

typedef struct ten_env_set_property_async_context_t {
  ten_env_t *ten_env;
  ten_env_set_property_async_cb_t cb;
  void *cb_data;

  // TODO(Liu): Replace with ten_error_t.
  bool res;

  union {
    ten_extension_t *extension;
    ten_extension_group_t *extension_group;
  } from;
} ten_env_set_property_async_context_t;

typedef struct ten_env_set_property_sync_context_t {
  ten_error_t *err;
  ten_event_t *completed;
} ten_env_set_property_sync_context_t;
