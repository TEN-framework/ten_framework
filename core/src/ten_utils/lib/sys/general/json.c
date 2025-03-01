//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#include "ten_utils/lib/json.h"

#include <stdbool.h>

#include "jansson.h"
#include "ten_runtime/common/error_code.h"
#include "ten_utils/lib/error.h"
#include "ten_utils/lib/signature.h"
#include "ten_utils/log/log.h"
#include "ten_utils/macro/check.h"
#include "ten_utils/macro/memory.h"
#include "ten_utils/value/type.h"

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
  self->ctx = ctx;
  self->owned = false;
}

ten_json_t *ten_json_create(void *ctx) {
  ten_json_t *self = TEN_MALLOC(sizeof(ten_json_t));
  TEN_ASSERT(self, "Failed to allocate memory.");

  ten_json_init(self, ctx);

  return self;
}

void ten_json_deinit(ten_json_t *self) {
  TEN_ASSERT(self && ten_json_check_integrity(self), "Invalid argument.");

  if (self->owned && self->json) {
    json_decref(self->json);
    self->json = NULL;
  }

  self->owned = false;
}

void ten_json_destroy(ten_json_t *self) {
  TEN_ASSERT(self && ten_json_check_integrity(self), "Invalid argument.");

  ten_json_deinit(self);

  TEN_FREE(self);
}

TEN_TYPE ten_json_get_type(ten_json_t *self) {
  TEN_ASSERT(self && ten_json_check_integrity(self), "Invalid argument.");

  switch (json_typeof((struct json_t *)self->json)) {
  case JSON_OBJECT:
    return TEN_TYPE_OBJECT;
  case JSON_ARRAY:
    return TEN_TYPE_ARRAY;
  case JSON_STRING:
    return TEN_TYPE_STRING;
  case JSON_INTEGER:
    if (ten_json_get_integer_value(self) >= 0) {
      return TEN_TYPE_UINT64;
    } else {
      return TEN_TYPE_INT64;
    }
  case JSON_REAL:
    return TEN_TYPE_FLOAT64;
  case JSON_TRUE:
  case JSON_FALSE:
    return TEN_TYPE_BOOL;
  case JSON_NULL:
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
  if (json_is_string((struct json_t *)result.json)) {
    str = json_string_value(result.json);
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

  if (json_is_object((struct json_t *)object->json)) {
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

  if (json_is_array((struct json_t *)item->json)) {
    return true;
  }

  return false;
}

static bool ten_json_object_del(ten_json_t *self, const char *key) {
  TEN_ASSERT(self && ten_json_check_integrity(self), "Invalid argument.");
  TEN_ASSERT(key, "Invalid argument.");

  if (ten_json_object_peek(self, key, NULL)) {
    // 0: success, -1: error
    return json_object_del(self->json, key) == 0;
  }

  return false;
}

bool ten_json_object_peek_object_forcibly(ten_json_t *json, const char *key,
                                          ten_json_t *object) {
  TEN_ASSERT(json && ten_json_check_integrity(json), "Invalid argument.");
  TEN_ASSERT(key, "Invalid argument.");
  TEN_ASSERT(object, "Invalid argument.");

  bool success = ten_json_object_peek_object(json, key, object);
  if (success) {
    return true;
  }

  if (ten_json_object_peek(json, key, NULL)) {
    ten_json_object_del(json, key);
  }

  struct json_t *json_obj = json_object();
  TEN_ASSERT(json_obj, "Failed to allocate memory.");

  json_object_set_new(json->json, key, json_obj);

  object->json = json_obj;
  object->owned = false;

  return true;
}

bool ten_json_object_set_new(ten_json_t *self, const char *key,
                             ten_json_t *value) {
  TEN_ASSERT(self && ten_json_check_integrity(self), "Invalid argument.");
  TEN_ASSERT(key, "Invalid argument.");
  TEN_ASSERT(value && ten_json_check_integrity(value), "Invalid argument.");

  bool success = (json_object_set_new(self->json, key, value->json) != -1);
  if (!success) {
    return false;
  }

  value->owned = false;
  return true;
}

bool ten_json_object_set_int(ten_json_t *self, const char *key, int64_t value) {
  TEN_ASSERT(self && ten_json_check_integrity(self), "Invalid argument.");
  TEN_ASSERT(key, "Invalid argument.");

  bool success =
      (json_object_set_new(self->json, key, json_integer(value)) != -1);
  if (!success) {
    return false;
  }

  return true;
}

bool ten_json_object_set_real(ten_json_t *self, const char *key, double value) {
  TEN_ASSERT(self && ten_json_check_integrity(self), "Invalid argument.");
  TEN_ASSERT(key, "Invalid argument.");

  bool success = (json_object_set_new(self->json, key, json_real(value)) != -1);
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

  bool success =
      (json_object_set_new(self->json, key, json_string(value)) != -1);
  if (!success) {
    return false;
  }

  return true;
}

bool ten_json_object_set_bool(ten_json_t *self, const char *key, bool value) {
  TEN_ASSERT(self && ten_json_check_integrity(self), "Invalid argument.");
  TEN_ASSERT(key, "Invalid argument.");
  TEN_ASSERT(value, "Invalid argument.");

  bool success =
      (json_object_set_new(self->json, key, json_boolean(value)) != -1);
  if (!success) {
    return false;
  }

  return true;
}

bool ten_json_object_peek(ten_json_t *self, const char *key,
                          ten_json_t *value) {
  TEN_ASSERT(self && ten_json_check_integrity(self), "Invalid argument.");
  TEN_ASSERT(key, "Invalid argument.");

  if (!json_is_object((struct json_t *)self->json)) {
    return false;
  }

  struct json_t *json_obj = json_object_get(self->json, key);
  if (!json_obj) {
    return NULL;
  }

  if (value) {
    value->json = json_obj;
    value->owned = false;
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

  struct json_t *arr_json = json_array();
  TEN_ASSERT(arr_json, "Failed to allocate memory.");

  json_object_set_new(self->json, key, arr_json);

  array->json = arr_json;
  array->owned = false;

  return true;
}

const char *ten_json_object_iter_key(ten_json_object_iter_t *iter) {
  return json_object_iter_key((void *)iter);
}

bool ten_json_object_iter_peek_value(ten_json_object_iter_t *iter,
                                     ten_json_t *value) {
  TEN_ASSERT(iter, "Invalid argument.");

  struct json_t *value_json = json_object_iter_value((void *)iter);
  if (!value_json) {
    return false;
  }

  value->json = value_json;
  value->owned = false;

  return true;
}

ten_json_object_iter_t *ten_json_object_iter(ten_json_t *self) {
  TEN_ASSERT(self && ten_json_check_integrity(self), "Invalid argument.");
  return (ten_json_object_iter_t *)json_object_iter(self->json);
}

ten_json_object_iter_t *ten_json_object_key_to_iter(const char *key) {
  TEN_ASSERT(key, "Invalid argument.");
  return (ten_json_object_iter_t *)json_object_key_to_iter(key);
}

ten_json_object_iter_t *
ten_json_object_iter_next(ten_json_t *self, ten_json_object_iter_t *iter) {
  TEN_ASSERT(self && ten_json_check_integrity(self), "Invalid argument.");
  TEN_ASSERT(iter, "Invalid argument.");
  return (ten_json_object_iter_t *)json_object_iter_next(self->json,
                                                         (void *)iter);
}

bool ten_json_array_append_new(ten_json_t *self, ten_json_t *item) {
  TEN_ASSERT(self && ten_json_check_integrity(self), "Invalid argument.");
  TEN_ASSERT(item && ten_json_check_integrity(item), "Invalid argument.");

  bool success = (json_array_append_new(self->json, item->json) != -1);
  if (!success) {
    return false;
  }

  item->owned = false;
  return true;
}

bool ten_json_array_append_object_and_peak(ten_json_t *self,
                                           ten_json_t *object) {
  TEN_ASSERT(self && ten_json_check_integrity(self), "Invalid argument.");
  TEN_ASSERT(object && ten_json_check_integrity(object), "Invalid argument.");

  struct json_t *obj_json = json_object();
  TEN_ASSERT(obj_json, "Failed to allocate memory.");

  bool success = (json_array_append_new(self->json, obj_json) != -1);
  if (!success) {
    return false;
  }

  object->json = obj_json;
  object->owned = false;

  return true;
}

bool ten_json_array_append_array_and_peak(ten_json_t *self, ten_json_t *array) {
  TEN_ASSERT(self && ten_json_check_integrity(self), "Invalid argument.");
  TEN_ASSERT(array && ten_json_check_integrity(array), "Invalid argument.");

  struct json_t *arr_json = json_array();
  TEN_ASSERT(arr_json, "Failed to allocate memory.");

  bool success = json_array_append_new(self->json, arr_json);
  if (!success) {
    return false;
  }

  array->json = arr_json;
  array->owned = false;

  return true;
}

size_t ten_json_array_get_size(ten_json_t *self) {
  TEN_ASSERT(self && ten_json_check_integrity(self), "Invalid argument.");
  return json_array_size(self->json);
}

bool ten_json_array_peek_item(ten_json_t *self, size_t index,
                              ten_json_t *item) {
  TEN_ASSERT(self && ten_json_check_integrity(self), "Invalid argument.");
  TEN_ASSERT(item, "Invalid argument.");

  struct json_t *item_json = json_array_get(self->json, index);
  if (!item_json) {
    return false;
  }

  item->json = item_json;
  item->owned = false;

  return true;
}

const char *ten_json_to_string(ten_json_t *self, const char *key,
                               bool *must_free) {
  TEN_ASSERT(self && ten_json_check_integrity(self), "Invalid argument.");
  TEN_ASSERT(must_free, "Invalid argument.");

  *must_free = false;

  if (key) {
    if (json_is_object((struct json_t *)self->json) &&
        json_object_get(self->json, key) != NULL) {
      struct json_t *value = json_object_get(self->json, key);
      switch (value->type) {
      case JSON_STRING:
        return json_string_value(value);
      case JSON_OBJECT:
      case JSON_ARRAY:
        *must_free = true; // json_dumps will invoke malloc
        return json_dumps(value, JSON_ENCODE_ANY);

      default:
        TEN_ASSERT(0, "Handle more types: %d", value->type);
        break;
      }
    }
  } else {
    *must_free = true; // json_dumps will invoke malloc
    return json_dumps(self->json, JSON_ENCODE_ANY);
  }

  return NULL;
}

ten_json_t *ten_json_from_string(const char *value, ten_error_t *err) {
  TEN_ASSERT(value, "Invalid argument.");
  if (!value || !*value) {
    return NULL;
  }

  json_error_t error;
  struct json_t *ret = json_loads(value, JSON_DECODE_ANY, &error);
  if (!ret) {
    if (err) {
      ten_error_set(err, TEN_ERROR_CODE_INVALID_JSON, "%s: %s", value,
                    error.text);
    }
    TEN_LOGE("Failed to parse %s: %s", value, error.text);
    return NULL;
  }

  ten_json_t *result = ten_json_create(ten_json_create_new_ctx());
  TEN_ASSERT(result, "Failed to allocate memory.");

  result->json = ret;
  result->owned = true;

  return result;
}

bool ten_json_is_object(ten_json_t *self) {
  TEN_ASSERT(self && ten_json_check_integrity(self), "Invalid argument.");
  return json_is_object((struct json_t *)self->json);
}

bool ten_json_is_array(ten_json_t *self) {
  TEN_ASSERT(self && ten_json_check_integrity(self), "Invalid argument.");
  return json_is_array((struct json_t *)self->json);
}

bool ten_json_is_string(ten_json_t *self) {
  TEN_ASSERT(self && ten_json_check_integrity(self), "Invalid argument.");
  return json_is_string((struct json_t *)self->json);
}

bool ten_json_is_integer(ten_json_t *self) {
  TEN_ASSERT(self && ten_json_check_integrity(self), "Invalid argument.");
  return json_is_integer((struct json_t *)self->json);
}

bool ten_json_is_boolean(ten_json_t *self) {
  TEN_ASSERT(self && ten_json_check_integrity(self), "Invalid argument.");
  return json_is_boolean((struct json_t *)self->json);
}

bool ten_json_is_real(ten_json_t *self) {
  TEN_ASSERT(self && ten_json_check_integrity(self), "Invalid argument.");
  return json_is_real((struct json_t *)self->json);
}

bool ten_json_is_null(ten_json_t *self) {
  TEN_ASSERT(self && ten_json_check_integrity(self), "Invalid argument.");
  return json_is_null((struct json_t *)self->json);
}

bool ten_json_is_true(ten_json_t *self) {
  TEN_ASSERT(self && ten_json_check_integrity(self), "Invalid argument.");
  return json_is_true((struct json_t *)self->json);
}

const char *ten_json_peek_string_value(ten_json_t *self) {
  TEN_ASSERT(self && ten_json_check_integrity(self), "Invalid argument.");
  return json_string_value((struct json_t *)self->json);
}

int64_t ten_json_get_integer_value(ten_json_t *self) {
  TEN_ASSERT(self && ten_json_check_integrity(self), "Invalid argument.");
  return json_integer_value((struct json_t *)self->json);
}

bool ten_json_get_boolean_value(ten_json_t *self) {
  TEN_ASSERT(self && ten_json_check_integrity(self), "Invalid argument.");
  return json_boolean_value((struct json_t *)self->json);
}

double ten_json_get_real_value(ten_json_t *self) {
  TEN_ASSERT(self && ten_json_check_integrity(self), "Invalid argument.");
  return json_real_value((struct json_t *)self->json);
}

double ten_json_get_number_value(ten_json_t *self) {
  TEN_ASSERT(self && ten_json_check_integrity(self), "Invalid argument.");
  return json_number_value((struct json_t *)self->json);
}

bool ten_json_set_object(ten_json_t *self) {
  TEN_ASSERT(self && ten_json_check_integrity(self), "Invalid argument.");

  ten_json_deinit(self);

  self->json = json_object();
  TEN_ASSERT(self->json, "Failed to allocate memory.");

  self->owned = true;

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

  self->json = json_array();
  TEN_ASSERT(self->json, "Failed to allocate memory.");

  self->owned = true;

  return true;
}

bool ten_json_set_string(ten_json_t *self, const char *value) {
  TEN_ASSERT(self && ten_json_check_integrity(self), "Invalid argument.");

  ten_json_deinit(self);

  self->json = json_string(value);
  TEN_ASSERT(self->json, "Failed to allocate memory.");

  self->owned = true;

  return true;
}

bool ten_json_set_integer(ten_json_t *self, int64_t value) {
  TEN_ASSERT(self && ten_json_check_integrity(self), "Invalid argument.");

  ten_json_deinit(self);

  self->json = json_integer(value);
  TEN_ASSERT(self->json, "Failed to allocate memory.");

  self->owned = true;

  return true;
}

bool ten_json_set_real(ten_json_t *self, double value) {
  TEN_ASSERT(self && ten_json_check_integrity(self), "Invalid argument.");

  ten_json_deinit(self);

  self->json = json_real(value);
  TEN_ASSERT(self->json, "Failed to allocate memory.");

  self->owned = true;

  return true;
}

bool ten_json_set_boolean(ten_json_t *self, bool value) {
  TEN_ASSERT(self && ten_json_check_integrity(self), "Invalid argument.");

  ten_json_deinit(self);

  self->json = json_boolean(value);
  TEN_ASSERT(self->json, "Failed to allocate memory.");

  self->owned = true;

  return true;
}

bool ten_json_set_null(ten_json_t *self) {
  TEN_ASSERT(self && ten_json_check_integrity(self), "Invalid argument.");

  ten_json_deinit(self);

  self->json = json_null();
  TEN_ASSERT(self->json, "Failed to allocate memory.");

  self->owned = true;

  return true;
}

void *ten_json_create_new_ctx(void) { return NULL; }
