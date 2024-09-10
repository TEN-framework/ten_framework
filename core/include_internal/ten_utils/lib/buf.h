//
// This file is part of the TEN Framework project.
// See https://github.com/TEN-framework/ten_framework/LICENSE for license
// information.
//
#pragma once

#include "ten_utils/ten_config.h"

#include <stdbool.h>

#include "ten_utils/lib/buf.h"

TEN_UTILS_PRIVATE_API bool ten_buf_get_back(ten_buf_t *self, uint8_t *dest,
                                            size_t size);

/**
 * @brief Append @a data with @a size bytes to the end of @a self. The source
 * buffer will be _copied_ into the buf.
 *
 * @param self The buf.
 */
TEN_UTILS_API bool ten_buf_push(ten_buf_t *self, const uint8_t *src,
                                size_t size);

TEN_UTILS_PRIVATE_API bool ten_buf_pop(ten_buf_t *self, uint8_t *dest,
                                       size_t size);

/**
 * @brief Ensure buffer has room for size @a len, grows buffer size if needed.
 *
 * @param self The buf.
 */
TEN_UTILS_API bool ten_buf_reserve(ten_buf_t *self, size_t len);

TEN_UTILS_API void ten_buf_set_fixed_size(ten_buf_t *self, bool fixed);

TEN_UTILS_API size_t ten_buf_get_content_size(ten_buf_t *self);

TEN_UTILS_API void ten_buf_take_ownership(ten_buf_t *self);

TEN_UTILS_API void ten_buf_release_ownership(ten_buf_t *self);
