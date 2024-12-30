//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#pragma once

#include "ten_utils/ten_config.h"

#include <stdbool.h>
#include <stdint.h>

#include "ten_utils/lib/signature.h"
#include "ten_utils/lib/string.h"

typedef int64_t ten_errno_t;

#define TEN_ERROR_SIGNATURE 0xCA49E5F63FC43623U

// 0 is a special TEN errno value, representing success/ok.
#define TEN_ERRNO_OK 0

typedef struct ten_error_t {
  ten_signature_t signature;

  ten_errno_t err_no;
  ten_string_t err_msg;
} ten_error_t;

TEN_UTILS_API bool ten_error_check_integrity(ten_error_t *self);

TEN_UTILS_API void ten_error_init(ten_error_t *self);

TEN_UTILS_API void ten_error_deinit(ten_error_t *self);

TEN_UTILS_API ten_error_t *ten_error_create(void);

TEN_UTILS_API void ten_error_copy(ten_error_t *self, ten_error_t *other);

// Set error info, return true if set success, false otherwise.
TEN_UTILS_API bool ten_error_set(ten_error_t *self, ten_errno_t err_no,
                                 const char *fmt, ...);

TEN_UTILS_API bool ten_error_vset(ten_error_t *self, ten_errno_t err_no,
                                  const char *fmt, va_list ap);

TEN_UTILS_API bool ten_error_prepend_errmsg(ten_error_t *self, const char *fmt,
                                            ...);

TEN_UTILS_API bool ten_error_append_errmsg(ten_error_t *self, const char *fmt,
                                           ...);

// Get last errno in current context, return TEN_ERRNO_OK if no error set
// before.
TEN_UTILS_API ten_errno_t ten_error_errno(ten_error_t *self);

TEN_UTILS_API const char *ten_error_errmsg(ten_error_t *self);

TEN_UTILS_API void ten_error_reset(ten_error_t *self);

TEN_UTILS_API void ten_error_destroy(ten_error_t *self);

TEN_UTILS_API bool ten_error_is_success(ten_error_t *self);
