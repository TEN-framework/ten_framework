//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#include "ten_utils/lib/json.h"

#include <assert.h>
#include <stdbool.h>

#include "ten_runtime/common/error_code.h"
#include "ten_utils/lib/error.h"
#include "ten_utils/lib/signature.h"
#include "ten_utils/log/log.h"
#include "ten_utils/macro/check.h"
#include "ten_utils/macro/memory.h"
#include "ten_utils/value/type.h"
#include "yyjson.h"

bool ten_json_check_integrity(ten_json_t *self) {
  TEN_ASSERT(self, "Invalid argument.");

  bool result = true;

  if (ten_signature_get(&self->signature) != TEN_JSON_SIGNATURE) {
    result = false;
  }

  return result;
}

void ten_json_init(ten_json_t *self, void *ctx) {
  TEN_ASSERT(self, "Invalid argument.");

  ten_signature_set(&self->signature, TEN_JSON_SIGNATURE);

  self->json = NULL;
  self->owned_json = false;

  self->ctx = ctx;
  self->owned_ctx = false;
}

ten_json_t *ten_json_create(void *ctx) {
  ten_json_t *self = TEN_MALLOC(sizeof(ten_json_t));
  TEN_ASSERT(self, "Failed to allocate memory.");

  ten_json_init(self, ctx);

  return self;
}

void ten_json_deinit(ten_json_t *self) {
  TEN_ASSERT(self && ten_json_check_integrity(self), "Invalid argument.");

  if (self->owned_json && self->json) {
    TEN_ASSERT(self->ctx, "Invalid argument.");
    // In yyjson, we do not need to free the yyjson_val itself; we only need
    // to free its associated yyjson_doc (i.e., our `ctx`).
  }
  self->json = NULL;
  self->owned_json = false;

  if (self->owned_ctx && self->ctx) {
    yyjson_mut_doc_free(self->ctx);
  }
  self->ctx = NULL;
  self->owned_ctx = false;
}

void ten_json_destroy(ten_json_t *self) {
  TEN_ASSERT(self && ten_json_check_integrity(self), "Invalid argument.");

  ten_json_deinit(self);

  TEN_FREE(self);
}

TEN_TYPE ten_json_get_type(ten_json_t *self) {
  TEN_ASSERT(self && ten_json_check_integrity(self), "Invalid argument.");

  switch (yyjson_mut_get_type((yyjson_mut_val *)self->json)) {
  case YYJSON_TYPE_OBJ:
    return TEN_TYPE_OBJECT;
  case YYJSON_TYPE_ARR:
    return TEN_TYPE_ARRAY;
  case YYJSON_TYPE_STR:
    return TEN_TYPE_STRING;
  case YYJSON_TYPE_NUM:
    if (yyjson_mut_is_uint(self->json)) {
      return TEN_TYPE_UINT64;
    } else if (yyjson_mut_is_int(self->json)) {
      return TEN_TYPE_INT64;
    } else if (yyjson_mut_is_real(self->json)) {
      return TEN_TYPE_FLOAT64;
    }
  case YYJSON_TYPE_BOOL:
    return TEN_TYPE_BOOL;
  case YYJSON_TYPE_NULL:
    return TEN_TYPE_NULL;
  default:
    TEN_ASSERT(0, "Should not happen.");
    return TEN_TYPE_INVALID;
  }
}

const char *ten_json_object_peek_string(ten_json_t *self, const char *key) {
  TEN_ASSERT(self && ten_json_check_integrity(self), "Invalid argument.");
  TEN_ASSERT(key, "Invalid argument.");

  ten_json_t result = TEN_JSON_INIT_VAL(self->ctx);
  bool success = ten_json_object_peek(self, key, &result);
  if (!success) {
    return NULL;
  }

  const char *str = NULL;
  if (yyjson_mut_is_str((yyjson_mut_val *)result.json)) {
    str = yyjson_get_str(result.json);
  }

  return str;
}

static bool ten_json_object_peek_object(ten_json_t *self, const char *key,
                                        ten_json_t *object) {
  TEN_ASSERT(self && ten_json_check_integrity(self), "Invalid argument.");
  TEN_ASSERT(key, "Invalid argument.");
  TEN_ASSERT(object, "Invalid argument.");

  bool success = ten_json_object_peek(self, key, object);
  if (!success) {
    return false;
  }

  if (yyjson_mut_is_obj((yyjson_mut_val *)object->json)) {
    return true;
  }

  return false;
}

static bool ten_json_object_peek_array(ten_json_t *self, const char *key,
                                       ten_json_t *item) {
  TEN_ASSERT(self && ten_json_check_integrity(self), "Invalid argument.");
  TEN_ASSERT(key, "Invalid argument.");
  TEN_ASSERT(item, "Invalid argument.");

  bool success = ten_json_object_peek(self, key, item);
  if (!success) {
    return false;
  }

  if (yyjson_mut_is_arr((yyjson_mut_val *)item->json)) {
    return true;
  }

  return false;
}

static bool ten_json_object_del(ten_json_t *self, const char *key) {
  TEN_ASSERT(self && ten_json_check_integrity(self), "Invalid argument.");
  TEN_ASSERT(key, "Invalid argument.");

  if (ten_json_object_peek(self, key, NULL)) {
    return yyjson_mut_obj_remove_key(self->json, key) == NULL;
  }

  return false;
}

bool ten_json_object_peek_object_forcibly(ten_json_t *self, const char *key,
                                          ten_json_t *object) {
  TEN_ASSERT(self && ten_json_check_integrity(self), "Invalid argument.");
  TEN_ASSERT(key, "Invalid argument.");
  TEN_ASSERT(object, "Invalid argument.");

  bool success = ten_json_object_peek_object(self, key, object);
  if (success) {
    return true;
  }

  if (ten_json_object_peek(self, key, NULL)) {
    ten_json_object_del(self, key);
  }

  yyjson_mut_val *json_obj = yyjson_mut_obj((yyjson_mut_doc *)self->ctx);
  TEN_ASSERT(json_obj, "Failed to allocate memory.");

  yyjson_mut_val *yyjson_key = yyjson_mut_str((yyjson_mut_doc *)self->ctx, key);
  yyjson_mut_obj_add(self->json, yyjson_key, json_obj);

  object->json = json_obj;
  object->owned_json = false;

  return true;
}

bool ten_json_object_set_new(ten_json_t *self, const char *key,
                             ten_json_t *value) {
  TEN_ASSERT(self && ten_json_check_integrity(self), "Invalid argument.");
  TEN_ASSERT(key, "Invalid argument.");
  TEN_ASSERT(value && ten_json_check_integrity(value), "Invalid argument.");

  yyjson_mut_val *yyjson_key = yyjson_mut_str((yyjson_mut_doc *)self->ctx, key);
  bool success = yyjson_mut_obj_add(self->json, yyjson_key, value->json);
  if (!success) {
    return false;
  }

  value->owned_json = false;
  return true;
}

bool ten_json_object_set_int(ten_json_t *self, const char *key, int64_t value) {
  TEN_ASSERT(self && ten_json_check_integrity(self), "Invalid argument.");
  TEN_ASSERT(key, "Invalid argument.");

  yyjson_mut_val *yyjson_key = yyjson_mut_str((yyjson_mut_doc *)self->ctx, key);
  yyjson_mut_val *yyjson_value =
      yyjson_mut_int((yyjson_mut_doc *)self->ctx, value);

  bool success = yyjson_mut_obj_add(self->json, yyjson_key, yyjson_value);
  if (!success) {
    return false;
  }

  return true;
}

bool ten_json_object_set_real(ten_json_t *self, const char *key, double value) {
  TEN_ASSERT(self && ten_json_check_integrity(self), "Invalid argument.");
  TEN_ASSERT(key, "Invalid argument.");

  yyjson_mut_val *yyjson_key = yyjson_mut_str((yyjson_mut_doc *)self->ctx, key);
  yyjson_mut_val *yyjson_value =
      yyjson_mut_real((yyjson_mut_doc *)self->ctx, value);
  bool success = yyjson_mut_obj_add(self->json, yyjson_key, yyjson_value);
  if (!success) {
    return false;
  }

  return true;
}

bool ten_json_object_set_string(ten_json_t *self, const char *key,
                                const char *value) {
  TEN_ASSERT(self && ten_json_check_integrity(self), "Invalid argument.");
  TEN_ASSERT(key, "Invalid argument.");
  TEN_ASSERT(value, "Invalid argument.");

  yyjson_mut_val *yyjson_key = yyjson_mut_str((yyjson_mut_doc *)self->ctx, key);
  yyjson_mut_val *yyjson_value =
      yyjson_mut_str((yyjson_mut_doc *)self->ctx, value);
  bool success = yyjson_mut_obj_add(self->json, yyjson_key, yyjson_value);
  if (!success) {
    return false;
  }

  return true;
}

bool ten_json_object_set_bool(ten_json_t *self, const char *key, bool value) {
  TEN_ASSERT(self && ten_json_check_integrity(self), "Invalid argument.");
  TEN_ASSERT(key, "Invalid argument.");
  TEN_ASSERT(value, "Invalid argument.");

  yyjson_mut_val *yyjson_key = yyjson_mut_str((yyjson_mut_doc *)self->ctx, key);
  yyjson_mut_val *yyjson_value =
      yyjson_mut_bool((yyjson_mut_doc *)self->ctx, value);
  bool success = yyjson_mut_obj_add(self->json, yyjson_key, yyjson_value);
  if (!success) {
    return false;
  }

  return true;
}

bool ten_json_object_peek(ten_json_t *self, const char *key,
                          ten_json_t *value) {
  TEN_ASSERT(self && ten_json_check_integrity(self), "Invalid argument.");
  TEN_ASSERT(key, "Invalid argument.");

  if (!yyjson_mut_is_obj((yyjson_mut_val *)self->json)) {
    return false;
  }

  yyjson_mut_val *json_obj = yyjson_mut_obj_get(self->json, key);
  if (!json_obj) {
    return NULL;
  }

  if (value) {
    value->json = json_obj;
    value->owned_json = false;
  }

  return true;
}

bool ten_json_object_peek_array_forcibly(ten_json_t *self, const char *key,
                                         ten_json_t *array) {
  TEN_ASSERT(self && ten_json_check_integrity(self), "Invalid argument.");
  TEN_ASSERT(key, "Invalid argument.");
  TEN_ASSERT(array, "Invalid argument.");

  bool success = ten_json_object_peek_array(self, key, array);
  if (success) {
    return true;
  }

  if (ten_json_object_peek(self, key, NULL)) {
    ten_json_object_del(self, key);
  }

  yyjson_mut_val *yyjson_key = yyjson_mut_str((yyjson_mut_doc *)self->ctx, key);
  yyjson_mut_val *yyjson_value = yyjson_mut_arr((yyjson_mut_doc *)self->ctx);
  TEN_ASSERT(yyjson_value, "Failed to allocate memory.");

  success = yyjson_mut_obj_add(self->json, yyjson_key, yyjson_value);
  if (!success) {
    return false;
  }

  array->json = yyjson_value;
  array->owned_json = false;

  return true;
}

static_assert(sizeof(yyjson_mut_obj_iter) < sizeof(ten_json_iter_t),
              "ten_json_iter_t is too large.");

bool ten_json_object_iter_init(ten_json_t *self, ten_json_iter_t *iter) {
  TEN_ASSERT(self && ten_json_check_integrity(self), "Invalid argument.");
  TEN_ASSERT(iter, "Invalid argument.");

  bool success =
      yyjson_mut_obj_iter_init(self->json, (yyjson_mut_obj_iter *)iter);
  if (!success) {
    return false;
  }

  return true;
}

const char *ten_json_object_iter_next_key(void *iter, ten_json_t *key) {
  yyjson_mut_val *yyjson_key =
      yyjson_mut_obj_iter_next((yyjson_mut_obj_iter *)iter);
  if (!yyjson_key) {
    key->json = NULL;
    key->owned_json = false;
    return NULL;
  }

  key->json = yyjson_key;
  key->owned_json = false;

  return yyjson_mut_get_str(yyjson_key);
}

bool ten_json_object_iter_peek_value(ten_json_t *key, ten_json_t *value) {
  TEN_ASSERT(key, "Invalid argument.");
  TEN_ASSERT(value, "Invalid argument.");

  yyjson_mut_val *yyjson_value =
      yyjson_mut_obj_iter_get_val((yyjson_mut_val *)key->json);
  if (!yyjson_value) {
    value->json = NULL;
    value->owned_json = false;
    return false;
  }

  value->json = yyjson_value;
  value->owned_json = false;

  return true;
}

bool ten_json_array_append_new(ten_json_t *self, ten_json_t *item) {
  TEN_ASSERT(self && ten_json_check_integrity(self), "Invalid argument.");
  TEN_ASSERT(item && ten_json_check_integrity(item), "Invalid argument.");

  bool success = yyjson_mut_arr_append(self->json, item->json);
  if (!success) {
    return false;
  }

  item->owned_json = false;
  return true;
}

bool ten_json_array_append_object_and_peak(ten_json_t *self,
                                           ten_json_t *object) {
  TEN_ASSERT(self && ten_json_check_integrity(self), "Invalid argument.");
  TEN_ASSERT(object && ten_json_check_integrity(object), "Invalid argument.");

  yyjson_mut_val *obj_json = yyjson_mut_obj((yyjson_mut_doc *)self->ctx);
  TEN_ASSERT(obj_json, "Failed to allocate memory.");

  bool success = yyjson_mut_arr_append(self->json, obj_json);
  if (!success) {
    return false;
  }

  object->json = obj_json;
  object->owned_json = false;

  return true;
}

bool ten_json_array_append_array_and_peak(ten_json_t *self, ten_json_t *array) {
  TEN_ASSERT(self && ten_json_check_integrity(self), "Invalid argument.");
  TEN_ASSERT(array && ten_json_check_integrity(array), "Invalid argument.");

  yyjson_mut_val *arr_json = yyjson_mut_arr((yyjson_mut_doc *)self->ctx);
  TEN_ASSERT(arr_json, "Failed to allocate memory.");

  bool success = yyjson_mut_arr_append(self->json, arr_json);
  if (!success) {
    return false;
  }

  array->json = arr_json;
  array->owned_json = false;

  return true;
}

size_t ten_json_array_get_size(ten_json_t *self) {
  TEN_ASSERT(self && ten_json_check_integrity(self), "Invalid argument.");
  return yyjson_mut_arr_size((yyjson_mut_val *)self->json);
}

bool ten_json_array_peek_item(ten_json_t *self, size_t index,
                              ten_json_t *item) {
  TEN_ASSERT(self && ten_json_check_integrity(self), "Invalid argument.");
  TEN_ASSERT(item, "Invalid argument.");

  yyjson_mut_val *item_json = yyjson_mut_arr_get(self->json, index);
  if (!item_json) {
    return false;
  }

  item->json = item_json;
  item->owned_json = false;

  return true;
}

const char *ten_json_to_string(ten_json_t *self, const char *key,
                               bool *must_free) {
  TEN_ASSERT(self && ten_json_check_integrity(self), "Invalid argument.");
  TEN_ASSERT(must_free, "Invalid argument.");

  *must_free = false;

  if (key) {
    if (yyjson_mut_is_obj((yyjson_mut_val *)self->json) &&
        yyjson_mut_obj_get(self->json, key) != NULL) {
      yyjson_mut_val *value = yyjson_mut_obj_get(self->json, key);
      switch (yyjson_mut_get_type(value)) {
      case YYJSON_TYPE_STR:
        return yyjson_mut_get_str(value);
      case YYJSON_TYPE_OBJ:
      case YYJSON_TYPE_ARR:
        *must_free = true;
        return yyjson_mut_val_write(value, YYJSON_WRITE_PRETTY, NULL);

      default:
        TEN_ASSERT(0, "Handle more types: %d", yyjson_mut_get_type(value));
        break;
      }
    }
  } else {
    *must_free = true;
    return yyjson_mut_val_write((yyjson_mut_val *)self->json,
                                YYJSON_WRITE_PRETTY, NULL);
  }

  return NULL;
}

ten_json_t *ten_json_from_string(const char *value, ten_error_t *err) {
  TEN_ASSERT(value, "Invalid argument.");
  if (!value || !*value) {
    return NULL;
  }

  yyjson_doc *doc = yyjson_read(value, strlen(value), 0);
  yyjson_val *root = yyjson_doc_get_root(doc);

  yyjson_mut_doc *mut_doc = yyjson_mut_doc_new(NULL);
  yyjson_mut_val *mut_root = yyjson_val_mut_copy(mut_doc, root);
  yyjson_mut_doc_set_root(mut_doc, mut_root);

  yyjson_doc_free(doc);

  if (!mut_doc) {
    if (err) {
      ten_error_set(err, TEN_ERROR_CODE_INVALID_JSON, "%s", value);
    }
    TEN_LOGE("Failed to parse %s", value);
    return NULL;
  }

  ten_json_t *result = ten_json_create(mut_doc);
  TEN_ASSERT(result, "Failed to allocate memory.");

  result->json = mut_root;
  result->owned_json = true;

  result->owned_ctx = true;

  return result;
}

bool ten_json_is_object(ten_json_t *self) {
  TEN_ASSERT(self && ten_json_check_integrity(self), "Invalid argument.");
  return yyjson_mut_is_obj((yyjson_mut_val *)self->json);
}

bool ten_json_is_array(ten_json_t *self) {
  TEN_ASSERT(self && ten_json_check_integrity(self), "Invalid argument.");
  return yyjson_mut_is_arr((yyjson_mut_val *)self->json);
}

bool ten_json_is_string(ten_json_t *self) {
  TEN_ASSERT(self && ten_json_check_integrity(self), "Invalid argument.");
  return yyjson_mut_is_str((yyjson_mut_val *)self->json);
}

bool ten_json_is_integer(ten_json_t *self) {
  TEN_ASSERT(self && ten_json_check_integrity(self), "Invalid argument.");
  return yyjson_mut_is_int((yyjson_mut_val *)self->json);
}

bool ten_json_is_boolean(ten_json_t *self) {
  TEN_ASSERT(self && ten_json_check_integrity(self), "Invalid argument.");
  return yyjson_mut_is_bool((yyjson_mut_val *)self->json);
}

bool ten_json_is_real(ten_json_t *self) {
  TEN_ASSERT(self && ten_json_check_integrity(self), "Invalid argument.");
  return yyjson_mut_is_real((yyjson_mut_val *)self->json);
}

bool ten_json_is_null(ten_json_t *self) {
  TEN_ASSERT(self && ten_json_check_integrity(self), "Invalid argument.");
  return yyjson_mut_is_null((yyjson_mut_val *)self->json);
}

bool ten_json_is_true(ten_json_t *self) {
  TEN_ASSERT(self && ten_json_check_integrity(self), "Invalid argument.");
  return yyjson_mut_is_true((yyjson_mut_val *)self->json);
}

const char *ten_json_peek_string_value(ten_json_t *self) {
  TEN_ASSERT(self && ten_json_check_integrity(self), "Invalid argument.");
  return yyjson_mut_get_str((yyjson_mut_val *)self->json);
}

int64_t ten_json_get_integer_value(ten_json_t *self) {
  TEN_ASSERT(self && ten_json_check_integrity(self), "Invalid argument.");
  return yyjson_mut_get_int((yyjson_mut_val *)self->json);
}

bool ten_json_get_boolean_value(ten_json_t *self) {
  TEN_ASSERT(self && ten_json_check_integrity(self), "Invalid argument.");
  return yyjson_mut_get_bool((yyjson_mut_val *)self->json);
}

double ten_json_get_real_value(ten_json_t *self) {
  TEN_ASSERT(self && ten_json_check_integrity(self), "Invalid argument.");
  return yyjson_mut_get_real((yyjson_mut_val *)self->json);
}

double ten_json_get_number_value(ten_json_t *self) {
  TEN_ASSERT(self && ten_json_check_integrity(self), "Invalid argument.");
  return yyjson_mut_get_real((yyjson_mut_val *)self->json);
}

bool ten_json_set_object(ten_json_t *self) {
  TEN_ASSERT(self && ten_json_check_integrity(self), "Invalid argument.");

  ten_json_deinit(self);

  self->json = yyjson_mut_obj((yyjson_mut_doc *)self->ctx);
  TEN_ASSERT(self->json, "Failed to allocate memory.");

  self->owned_json = true;

  return true;
}

ten_json_t *ten_json_create_root_object(void) {
  ten_json_t *self = ten_json_create(ten_json_create_new_ctx());
  TEN_ASSERT(self, "Failed to allocate memory.");

  bool success = ten_json_set_object(self);
  TEN_ASSERT(success, "Failed to create the root object.");
  if (!success) {
    ten_json_destroy(self);
    return NULL;
  }

  return self;
}

bool ten_json_set_array(ten_json_t *self) {
  TEN_ASSERT(self && ten_json_check_integrity(self), "Invalid argument.");

  ten_json_deinit(self);

  self->json = yyjson_mut_arr((yyjson_mut_doc *)self->ctx);
  TEN_ASSERT(self->json, "Failed to allocate memory.");

  self->owned_json = true;

  return true;
}

bool ten_json_set_string(ten_json_t *self, const char *value) {
  TEN_ASSERT(self && ten_json_check_integrity(self), "Invalid argument.");

  ten_json_deinit(self);

  self->json = yyjson_mut_str((yyjson_mut_doc *)self->ctx, value);
  TEN_ASSERT(self->json, "Failed to allocate memory.");

  self->owned_json = true;

  return true;
}

bool ten_json_set_integer(ten_json_t *self, int64_t value) {
  TEN_ASSERT(self && ten_json_check_integrity(self), "Invalid argument.");

  ten_json_deinit(self);

  self->json = yyjson_mut_int((yyjson_mut_doc *)self->ctx, value);
  TEN_ASSERT(self->json, "Failed to allocate memory.");

  self->owned_json = true;

  return true;
}

bool ten_json_set_real(ten_json_t *self, double value) {
  TEN_ASSERT(self && ten_json_check_integrity(self), "Invalid argument.");

  ten_json_deinit(self);

  self->json = yyjson_mut_real((yyjson_mut_doc *)self->ctx, value);
  TEN_ASSERT(self->json, "Failed to allocate memory.");

  self->owned_json = true;

  return true;
}

bool ten_json_set_boolean(ten_json_t *self, bool value) {
  TEN_ASSERT(self && ten_json_check_integrity(self), "Invalid argument.");

  ten_json_deinit(self);

  self->json = yyjson_mut_bool((yyjson_mut_doc *)self->ctx, value);
  TEN_ASSERT(self->json, "Failed to allocate memory.");

  self->owned_json = true;

  return true;
}

bool ten_json_set_null(ten_json_t *self) {
  TEN_ASSERT(self && ten_json_check_integrity(self), "Invalid argument.");

  ten_json_deinit(self);

  self->json = yyjson_mut_null((yyjson_mut_doc *)self->ctx);
  TEN_ASSERT(self->json, "Failed to allocate memory.");

  self->owned_json = true;

  return true;
}

void *ten_json_create_new_ctx(void) { return yyjson_mut_doc_new(NULL); }
