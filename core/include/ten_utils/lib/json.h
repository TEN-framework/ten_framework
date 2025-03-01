//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#pragma once

#include "ten_utils/ten_config.h"

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#include "ten_utils/lib/error.h"
#include "ten_utils/value/type.h"

#define TEN_JSON_SIGNATURE 0xACA499C637670350U

#define TEN_JSON_INIT_VAL \
  (ten_json_t) { .signature = TEN_JSON_SIGNATURE, .owned = false, .json = NULL }

#define ten_json_array_foreach(array, index, item)                          \
  ten_json_t __item = TEN_JSON_INIT_VAL;                                    \
  for ((index) = 0;                                                         \
       (index) < ten_json_array_get_size(array) &&                          \
       ten_json_array_peek_item(array, index, &__item) && (item = &__item); \
       (index)++)

#define ten_json_object_foreach(object, key, value)                       \
  ten_json_t __value = TEN_JSON_INIT_VAL;                                 \
  for ((key) = ten_json_object_iter_key(ten_json_object_iter(object));    \
       (key) &&                                                           \
       (ten_json_object_iter_peek_value(ten_json_object_key_to_iter(key), \
                                        &__value)) &&                     \
       (value = &__value);                                                \
       (key) = ten_json_object_iter_key(ten_json_object_iter_next(        \
           object, ten_json_object_key_to_iter(key))))

typedef struct json_t json_t;

typedef struct ten_json_t {
  ten_signature_t signature;

  bool owned;
  json_t *json;
} ten_json_t;

typedef void *ten_json_object_iter_t;

TEN_UTILS_API bool ten_json_check_integrity(ten_json_t *self);

TEN_UTILS_API void ten_json_init(ten_json_t *self);

TEN_UTILS_API void ten_json_deinit(ten_json_t *self);

TEN_UTILS_API ten_json_t *ten_json_create(void);

TEN_UTILS_API void ten_json_destroy(ten_json_t *self);

TEN_UTILS_API TEN_TYPE ten_json_get_type(ten_json_t *self);

TEN_UTILS_API bool ten_json_object_peek(ten_json_t *self, const char *key,
                                        ten_json_t *value);

TEN_UTILS_API const char *ten_json_object_peek_string(ten_json_t *self,
                                                      const char *key);

TEN_UTILS_API bool ten_json_object_peek_object_forcibly(ten_json_t *self,
                                                        const char *key,
                                                        ten_json_t *object);

TEN_UTILS_API bool ten_json_object_peek_array_forcibly(ten_json_t *self,
                                                       const char *key,
                                                       ten_json_t *array);

TEN_UTILS_API bool ten_json_object_set_new(ten_json_t *self, const char *key,
                                           ten_json_t *value);

TEN_UTILS_API bool ten_json_object_set_string(ten_json_t *self, const char *key,
                                              const char *value);

TEN_UTILS_API bool ten_json_object_set_int(ten_json_t *self, const char *key,
                                           int64_t value);

TEN_UTILS_API bool ten_json_object_set_real(ten_json_t *self, const char *key,
                                            double value);

TEN_UTILS_API bool ten_json_object_set_bool(ten_json_t *self, const char *key,
                                            bool value);

TEN_UTILS_API const char *ten_json_object_iter_key(
    ten_json_object_iter_t *iter);

TEN_UTILS_API bool ten_json_object_iter_peek_value(ten_json_object_iter_t *iter,
                                                   ten_json_t *value);

TEN_UTILS_API ten_json_object_iter_t *ten_json_object_iter(ten_json_t *self);

TEN_UTILS_API ten_json_object_iter_t *ten_json_object_key_to_iter(
    const char *key);

TEN_UTILS_API ten_json_object_iter_t *ten_json_object_iter_next(
    ten_json_t *self, ten_json_object_iter_t *iter);

TEN_UTILS_API bool ten_json_array_append_new(ten_json_t *self,
                                             ten_json_t *item);

TEN_UTILS_API bool ten_json_array_append_object_and_peak(ten_json_t *self,
                                                         ten_json_t *object);

TEN_UTILS_API bool ten_json_array_append_array_and_peak(ten_json_t *self,
                                                        ten_json_t *array);

TEN_UTILS_API size_t ten_json_array_get_size(ten_json_t *self);

TEN_UTILS_API bool ten_json_array_peek_item(ten_json_t *self, size_t index,
                                            ten_json_t *item);

TEN_UTILS_API const char *ten_json_to_string(ten_json_t *self, const char *key,
                                             bool *must_free);

TEN_UTILS_API ten_json_t *ten_json_from_string(const char *value,
                                               ten_error_t *err);

TEN_UTILS_API bool ten_json_is_object(ten_json_t *self);

TEN_UTILS_API bool ten_json_is_array(ten_json_t *self);

TEN_UTILS_API bool ten_json_is_string(ten_json_t *self);

TEN_UTILS_API bool ten_json_is_integer(ten_json_t *self);

TEN_UTILS_API bool ten_json_is_boolean(ten_json_t *self);

TEN_UTILS_API bool ten_json_is_real(ten_json_t *self);

TEN_UTILS_API bool ten_json_is_true(ten_json_t *self);

TEN_UTILS_API bool ten_json_is_null(ten_json_t *self);

TEN_UTILS_API const char *ten_json_peek_string_value(ten_json_t *self);

TEN_UTILS_API int64_t ten_json_get_integer_value(ten_json_t *self);

TEN_UTILS_API bool ten_json_get_boolean_value(ten_json_t *self);

TEN_UTILS_API double ten_json_get_real_value(ten_json_t *self);

TEN_UTILS_API double ten_json_get_number_value(ten_json_t *self);

TEN_UTILS_API bool ten_json_set_string(ten_json_t *self, const char *value);

TEN_UTILS_API bool ten_json_set_integer(ten_json_t *self, int64_t value);

TEN_UTILS_API bool ten_json_set_real(ten_json_t *self, double value);

TEN_UTILS_API bool ten_json_set_boolean(ten_json_t *self, bool value);

TEN_UTILS_API bool ten_json_set_null(ten_json_t *self);

TEN_UTILS_API bool ten_json_set_object(ten_json_t *self);

TEN_UTILS_API bool ten_json_set_array(ten_json_t *self);

TEN_UTILS_API ten_json_t *ten_json_create_root_object(void);
