//
// This file is part of the TEN Framework project.
// See https://github.com/TEN-framework/ten_framework/LICENSE for license
// information.
//
#include "ten_utils/ten_config.h"

#include <stdbool.h>
#include <stddef.h>

typedef struct ten_vector_t {
  void *data;
  size_t size;
  size_t capacity;
} ten_vector_t;

TEN_UTILS_API void ten_vector_init(ten_vector_t *self, size_t capacity);

/**
 * Create a new vector of the given capacity.
 */
TEN_UTILS_API ten_vector_t *ten_vector_create(size_t capacity);

TEN_UTILS_API void ten_vector_deinit(ten_vector_t *self);

/**
 * Free vector related memory.
 */
TEN_UTILS_API void ten_vector_destroy(ten_vector_t *self);

TEN_UTILS_API void *ten_vector_grow(ten_vector_t *self, size_t size);

TEN_UTILS_API bool ten_vector_release_remaining_space(ten_vector_t *self);

TEN_UTILS_API void *ten_vector_take_out(ten_vector_t *self);
