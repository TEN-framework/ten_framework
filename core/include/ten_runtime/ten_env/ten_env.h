//
// This file is part of the TEN Framework project.
// See https://github.com/TEN-framework/ten_framework/LICENSE for license
// information.
//
#pragma once

#include "ten_runtime/ten_config.h"

#include "ten_runtime/common/errno.h"                  // IWYU pragma: keep
#include "ten_runtime/ten_env/internal/metadata.h"     // IWYU pragma: keep
#include "ten_runtime/ten_env/internal/on_xxx_done.h"  // IWYU pragma: keep
#include "ten_runtime/ten_env/internal/return.h"       // IWYU pragma: keep
#include "ten_runtime/ten_env/internal/send.h"         // IWYU pragma: keep

typedef struct ten_env_t ten_env_t;
typedef struct ten_extension_group_t ten_extension_group_t;

typedef void (*ten_env_addon_on_create_instance_async_cb_t)(ten_env_t *ten_env,
                                                            void *instance,
                                                            void *cb_data);

typedef void (*ten_env_addon_on_destroy_instance_async_cb_t)(ten_env_t *ten_env,
                                                             void *cb_data);

typedef void (*ten_env_is_cmd_connected_async_cb_t)(ten_env_t *ten_env,
                                                    bool result, void *cb_data,
                                                    ten_error_t *err);

TEN_RUNTIME_API bool ten_env_check_integrity(ten_env_t *self,
                                             bool check_thread);

TEN_RUNTIME_API bool ten_env_is_cmd_connected(ten_env_t *self,
                                              const char *cmd_name,
                                              ten_error_t *err);

TEN_RUNTIME_API void *ten_env_get_attached_target(ten_env_t *self);

TEN_RUNTIME_API ten_env_t *ten_env_mock_create(void);

TEN_RUNTIME_API void ten_env_destroy(ten_env_t *self);
