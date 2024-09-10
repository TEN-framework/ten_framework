//
// This file is part of the TEN Framework project.
// See https://github.com/TEN-framework/ten_framework/LICENSE for license
// information.
//
#include "ten_runtime/ten_config.h"

#include "include_internal/ten_runtime/extension/extension.h"

typedef struct ten_extension_test_t {
  ten_extension_thread_t *test_extension_thread;
  ten_extension_group_t *test_extension_group;
  ten_list_t pre_created_extensions;
  ten_thread_t *test_thread;
} ten_extension_test_t;

TEN_RUNTIME_API ten_extension_test_t *ten_extension_test_create(
    ten_extension_t *test_extension, ten_extension_t *target_extension);

TEN_RUNTIME_API void ten_extension_test_start(ten_extension_test_t *self);

TEN_RUNTIME_API void ten_extension_test_wait(ten_extension_test_t *self);

TEN_RUNTIME_API void ten_extension_test_destroy(ten_extension_test_t *self);
