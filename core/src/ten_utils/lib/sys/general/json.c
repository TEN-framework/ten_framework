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

void ten_json_init(ten_json_t *self) {
  TEN_ASSERT(self, "Invalid argument.");

  ten_signature_set(&self->signature, TEN_JSON_SIGNATURE);
  self->json = NULL;
  self->owned = true;
}

ten_json_t *ten_json_create(void) {
  ten_json_t *self = TEN_MALLOC(sizeof(ten_json_t));
  TEN_ASSERT(self, "Failed to allocate memory.");

  ten_json_init(self);

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

TEN_TYPE ten_json_get_type(ten_json_t *json) {
  TEN_ASSERT(json && ten_json_check_integrity(json), "Invalid argument.");

  switch (json_typeof(json->json)) {
  case JSON_OBJECT:
    return TEN_TYPE_OBJECT;
  case JSON_ARRAY:
    return TEN_TYPE_ARRAY;
  case JSON_STRING:
    return TEN_TYPE_STRING;
  case JSON_INTEGER:
    if (ten_json_get_integer_value(json) >= 0) {
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

const char *ten_json_object_peek_string(ten_json_t *json, const char *field) {
  TEN_ASSERT(json && ten_json_check_integrity(json), "Invalid argument.");
  TEN_ASSERT(field, "Invalid argument.");

  ten_json_t result = TEN_JSON_INIT_VAL;
  bool success = ten_json_object_peek(json, field, &result);
  if (!success) {
    return NULL;
  }

  const char *str = NULL;
  if (json_is_string(result.json)) {
    str = json_string_value(result.json);
  }

  ten_json_deinit(&result);
  return str;
}

bool ten_json_object_peek_object(ten_json_t *json, const char *field,
                                 ten_json_t *result) {
  TEN_ASSERT(json && ten_json_check_integrity(json), "Invalid argument.");
  TEN_ASSERT(field, "Invalid argument.");
  TEN_ASSERT(result, "Invalid argument.");

  bool success = ten_json_object_peek(json, field, result);
  if (!success) {
    return false;
  }

  if (json_is_object(result->json)) {
    return true;
  }

  return false;
}

bool ten_json_object_peek_array(ten_json_t *json, const char *field,
                                ten_json_t *result) {
  TEN_ASSERT(json && ten_json_check_integrity(json), "Invalid argument.");
  TEN_ASSERT(field, "Invalid argument.");
  TEN_ASSERT(result, "Invalid argument.");

  bool success = ten_json_object_peek(json, field, result);
  if (!success) {
    return false;
  }

  if (json_is_array(result->json)) {
    return true;
  }

  return false;
}

static bool ten_json_object_del(ten_json_t *json, const char *field) {
  TEN_ASSERT(json && ten_json_check_integrity(json), "Invalid argument.");
  TEN_ASSERT(field, "Invalid argument.");

  if (ten_json_object_peek(json, field, NULL)) {
    // 0: success, -1: error
    return json_object_del(json->json, field) == 0;
  }

  return false;
}

bool ten_json_object_peek_object_forcibly(ten_json_t *json, const char *field,
                                          ten_json_t *result) {
  TEN_ASSERT(json && ten_json_check_integrity(json), "Invalid argument.");
  TEN_ASSERT(field, "Invalid argument.");
  TEN_ASSERT(result, "Invalid argument.");

  bool success = ten_json_object_peek_object(json, field, result);
  if (success) {
    return true;
  }

  if (ten_json_object_peek(json, field, NULL)) {
    ten_json_object_del(json, field);
  }

  json_t *json_obj = json_object();
  TEN_ASSERT(json_obj, "Failed to allocate memory.");

  json_object_set_new(json->json, field, json_obj);

  result->json = json_obj;
  result->owned = false;

  return true;
}

bool ten_json_object_set_new(ten_json_t *json, const char *field,
                             ten_json_t *value) {
  TEN_ASSERT(json && ten_json_check_integrity(json), "Invalid argument.");
  TEN_ASSERT(field, "Invalid argument.");
  TEN_ASSERT(value && ten_json_check_integrity(value), "Invalid argument.");

  bool success = json_object_set_new(json->json, field, value->json) != -1;
  if (!success) {
    return false;
  }

  value->owned = false;
  return true;
}

void ten_json_object_set_int(ten_json_t *json, const char *field,
                             int64_t value) {
  TEN_ASSERT(json && ten_json_check_integrity(json), "Invalid argument.");
  TEN_ASSERT(field, "Invalid argument.");

  json_object_set_new(json->json, field, json_integer(value));
}

void ten_json_object_set_real(ten_json_t *json, const char *field,
                              double value) {
  TEN_ASSERT(json && ten_json_check_integrity(json), "Invalid argument.");
  TEN_ASSERT(field, "Invalid argument.");

  json_object_set_new(json->json, field, json_real(value));
}

void ten_json_object_set_string(ten_json_t *json, const char *field,
                                const char *value) {
  TEN_ASSERT(json && ten_json_check_integrity(json), "Invalid argument.");
  TEN_ASSERT(field, "Invalid argument.");
  TEN_ASSERT(value, "Invalid argument.");

  json_object_set_new(json->json, field, json_string(value));
}

void ten_json_object_set_bool(ten_json_t *json, const char *field, bool value) {
  TEN_ASSERT(json && ten_json_check_integrity(json), "Invalid argument.");
  TEN_ASSERT(field, "Invalid argument.");
  TEN_ASSERT(value, "Invalid argument.");

  json_object_set_new(json->json, field, json_boolean(value));
}

void ten_json_array_append_new(ten_json_t *json, ten_json_t *value) {
  TEN_ASSERT(json && ten_json_check_integrity(json), "Invalid argument.");
  TEN_ASSERT(value && ten_json_check_integrity(value), "Invalid argument.");

  json_array_append_new(json->json, value->json);

  value->owned = false;
}

void ten_json_array_append_object_and_peak(ten_json_t *json,
                                           ten_json_t *result) {
  TEN_ASSERT(json && ten_json_check_integrity(json), "Invalid argument.");
  TEN_ASSERT(result && ten_json_check_integrity(result), "Invalid argument.");

  json_t *obj_json = json_object();
  TEN_ASSERT(obj_json, "Failed to allocate memory.");

  json_array_append_new(json->json, obj_json);

  result->json = obj_json;
  result->owned = false;
}

void ten_json_array_append_array_and_peak(ten_json_t *json,
                                          ten_json_t *result) {
  TEN_ASSERT(json && ten_json_check_integrity(json), "Invalid argument.");
  TEN_ASSERT(result && ten_json_check_integrity(result), "Invalid argument.");

  json_t *arr_json = json_array();
  TEN_ASSERT(arr_json, "Failed to allocate memory.");

  json_array_append_new(json->json, arr_json);

  result->json = arr_json;
  result->owned = false;
}

bool ten_json_object_peek(ten_json_t *json, const char *field,
                          ten_json_t *result) {
  TEN_ASSERT(json && ten_json_check_integrity(json), "Invalid argument.");
  TEN_ASSERT(field, "Invalid argument.");

  if (!json_is_object(json->json)) {
    return false;
  }

  json_t *json_obj = json_object_get(json->json, field);
  if (!json_obj) {
    return NULL;
  }

  if (result) {
    result->json = json_obj;
    result->owned = false;
  }

  return true;
}

bool ten_json_object_peek_array_forcibly(ten_json_t *json, const char *field,
                                         ten_json_t *result) {
  TEN_ASSERT(json && ten_json_check_integrity(json), "Invalid argument.");
  TEN_ASSERT(field, "Invalid argument.");
  TEN_ASSERT(result, "Invalid argument.");

  bool success = ten_json_object_peek_array(json, field, result);
  if (success) {
    return true;
  }

  if (ten_json_object_peek(json, field, NULL)) {
    ten_json_object_del(json, field);
  }

  json_t *arr_json = json_array();
  TEN_ASSERT(arr_json, "Failed to allocate memory.");

  json_object_set_new(json->json, field, arr_json);

  result->json = arr_json;
  result->owned = false;

  return true;
}

const char *ten_json_to_string(ten_json_t *json, const char *field,
                               bool *must_free) {
  TEN_ASSERT(json && ten_json_check_integrity(json), "Invalid argument.");
  TEN_ASSERT(must_free, "Invalid argument.");

  *must_free = false;

  if (field) {
    if (json_is_object(json->json) &&
        json_object_get(json->json, field) != NULL) {
      json_t *value = json_object_get(json->json, field);
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
    return json_dumps(json->json, JSON_ENCODE_ANY);
  }

  return NULL;
}

ten_json_t *ten_json_from_string(const char *msg, ten_error_t *err) {
  TEN_ASSERT(msg, "Invalid argument.");
  if (!msg || !*msg) {
    return NULL;
  }

  json_error_t error;
  json_t *ret = json_loads(msg, JSON_DECODE_ANY, &error);
  if (!ret) {
    if (err) {
      ten_error_set(err, TEN_ERROR_CODE_INVALID_JSON, "%s: %s", msg,
                    error.text);
    }
    TEN_LOGE("Failed to parse %s: %s", msg, error.text);
    return NULL;
  }

  ten_json_t *result = ten_json_create();
  TEN_ASSERT(result, "Failed to allocate memory.");

  result->json = ret;
  result->owned = true;

  return result;
}

bool ten_json_is_object(ten_json_t *json) {
  TEN_ASSERT(json && ten_json_check_integrity(json), "Invalid argument.");
  return json_is_object(json->json);
}

bool ten_json_is_array(ten_json_t *json) {
  TEN_ASSERT(json && ten_json_check_integrity(json), "Invalid argument.");
  return json_is_array(json->json);
}

bool ten_json_is_string(ten_json_t *json) {
  TEN_ASSERT(json && ten_json_check_integrity(json), "Invalid argument.");
  return json_is_string(json->json);
}

bool ten_json_is_integer(ten_json_t *json) {
  TEN_ASSERT(json && ten_json_check_integrity(json), "Invalid argument.");
  return json_is_integer(json->json);
}

bool ten_json_is_boolean(ten_json_t *json) {
  TEN_ASSERT(json && ten_json_check_integrity(json), "Invalid argument.");
  return json_is_boolean(json->json);
}

bool ten_json_is_real(ten_json_t *json) {
  TEN_ASSERT(json && ten_json_check_integrity(json), "Invalid argument.");
  return json_is_real(json->json);
}

bool ten_json_is_null(ten_json_t *json) {
  TEN_ASSERT(json && ten_json_check_integrity(json), "Invalid argument.");
  return json_is_null(json->json);
}

bool ten_json_is_true(ten_json_t *json) {
  TEN_ASSERT(json && ten_json_check_integrity(json), "Invalid argument.");
  return json_is_true(json->json);
}

const char *ten_json_peek_string_value(ten_json_t *json) {
  TEN_ASSERT(json && ten_json_check_integrity(json), "Invalid argument.");
  return json_string_value(json->json);
}

int64_t ten_json_get_integer_value(ten_json_t *json) {
  TEN_ASSERT(json && ten_json_check_integrity(json), "Invalid argument.");
  return json_integer_value(json->json);
}

bool ten_json_get_boolean_value(ten_json_t *json) {
  TEN_ASSERT(json && ten_json_check_integrity(json), "Invalid argument.");
  return json_boolean_value(json->json);
}

double ten_json_get_real_value(ten_json_t *json) {
  TEN_ASSERT(json && ten_json_check_integrity(json), "Invalid argument.");
  return json_real_value(json->json);
}

double ten_json_get_number_value(ten_json_t *json) {
  TEN_ASSERT(json && ten_json_check_integrity(json), "Invalid argument.");
  return json_number_value(json->json);
}

bool ten_json_set_object(ten_json_t *self) {
  TEN_ASSERT(self && ten_json_check_integrity(self), "Invalid argument.");

  ten_json_deinit(self);

  self->json = json_object();
  TEN_ASSERT(self->json, "Failed to allocate memory.");

  self->owned = true;

  return true;
}

ten_json_t *ten_json_create_object(void) {
  ten_json_t *self = ten_json_create();
  TEN_ASSERT(self, "Failed to allocate memory.");

  ten_json_set_object(self);

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

ten_json_t *ten_json_create_array(void) {
  ten_json_t *self = ten_json_create();
  TEN_ASSERT(self, "Failed to allocate memory.");

  ten_json_set_array(self);

  return self;
}

bool ten_json_set_string(ten_json_t *self, const char *value) {
  TEN_ASSERT(self && ten_json_check_integrity(self), "Invalid argument.");

  ten_json_deinit(self);

  self->json = json_string(value);
  TEN_ASSERT(self->json, "Failed to allocate memory.");

  self->owned = true;

  return true;
}

ten_json_t *ten_json_create_string(const char *str) {
  TEN_ASSERT(str, "Invalid argument.");

  ten_json_t *self = ten_json_create();
  TEN_ASSERT(self, "Failed to allocate memory.");

  ten_json_set_string(self, str);

  return self;
}

bool ten_json_set_integer(ten_json_t *self, int64_t value) {
  TEN_ASSERT(self && ten_json_check_integrity(self), "Invalid argument.");

  ten_json_deinit(self);

  self->json = json_integer(value);
  TEN_ASSERT(self->json, "Failed to allocate memory.");

  self->owned = true;

  return true;
}

ten_json_t *ten_json_create_integer(int64_t value) {
  ten_json_t *self = ten_json_create();
  TEN_ASSERT(self, "Failed to allocate memory.");

  ten_json_set_integer(self, value);

  return self;
}

bool ten_json_set_real(ten_json_t *self, double value) {
  TEN_ASSERT(self && ten_json_check_integrity(self), "Invalid argument.");

  ten_json_deinit(self);

  self->json = json_real(value);
  TEN_ASSERT(self->json, "Failed to allocate memory.");

  self->owned = true;

  return true;
}

ten_json_t *ten_json_create_real(double value) {
  ten_json_t *self = ten_json_create();
  TEN_ASSERT(self, "Failed to allocate memory.");

  ten_json_set_real(self, value);

  return self;
}

bool ten_json_set_boolean(ten_json_t *self, bool value) {
  TEN_ASSERT(self && ten_json_check_integrity(self), "Invalid argument.");

  ten_json_deinit(self);

  self->json = json_boolean(value);
  TEN_ASSERT(self->json, "Failed to allocate memory.");

  self->owned = true;

  return true;
}

ten_json_t *ten_json_create_boolean(bool value) {
  ten_json_t *self = ten_json_create();
  TEN_ASSERT(self, "Failed to allocate memory.");

  ten_json_set_boolean(self, value);

  return self;
}

ten_json_t *ten_json_create_true(void) {
  ten_json_t *self = ten_json_create();
  TEN_ASSERT(self, "Failed to allocate memory.");

  self->json = json_true();
  TEN_ASSERT(self->json, "Failed to allocate memory.");

  self->owned = true;

  return self;
}

ten_json_t *ten_json_create_false(void) {
  ten_json_t *self = ten_json_create();
  TEN_ASSERT(self, "Failed to allocate memory.");

  self->json = json_false();
  TEN_ASSERT(self->json, "Failed to allocate memory.");

  self->owned = true;

  return self;
}

bool ten_json_set_null(ten_json_t *self) {
  TEN_ASSERT(self && ten_json_check_integrity(self), "Invalid argument.");

  ten_json_deinit(self);

  self->json = json_null();
  TEN_ASSERT(self->json, "Failed to allocate memory.");

  self->owned = true;

  return true;
}

ten_json_t *ten_json_create_null(void) {
  ten_json_t *self = ten_json_create();
  TEN_ASSERT(self, "Failed to allocate memory.");

  ten_json_set_null(self);

  return self;
}

size_t ten_json_array_get_size(ten_json_t *json) {
  TEN_ASSERT(json && ten_json_check_integrity(json), "Invalid argument.");
  return json_array_size(json->json);
}

bool ten_json_array_peek_item(ten_json_t *json, size_t index,
                              ten_json_t *result) {
  TEN_ASSERT(json && ten_json_check_integrity(json), "Invalid argument.");
  TEN_ASSERT(result, "Invalid argument.");

  json_t *item = json_array_get(json->json, index);
  if (!item) {
    return false;
  }

  result->json = item;
  result->owned = false;

  return true;
}

const char *ten_json_object_iter_key(ten_json_object_iter_t *iter) {
  return json_object_iter_key((void *)iter);
}

bool ten_json_object_iter_peek_value(ten_json_object_iter_t *iter,
                                     ten_json_t *result) {
  TEN_ASSERT(iter, "Invalid argument.");

  json_t *value = json_object_iter_value((void *)iter);
  if (!value) {
    return false;
  }

  result->json = value;
  result->owned = false;

  return true;
}

ten_json_object_iter_t *ten_json_object_iter(ten_json_t *json) {
  TEN_ASSERT(json && ten_json_check_integrity(json), "Invalid argument.");
  return (ten_json_object_iter_t *)json_object_iter(json->json);
}

ten_json_object_iter_t *ten_json_object_key_to_iter(const char *key) {
  TEN_ASSERT(key, "Invalid argument.");
  return (ten_json_object_iter_t *)json_object_key_to_iter(key);
}

ten_json_object_iter_t *
ten_json_object_iter_next(ten_json_t *json, ten_json_object_iter_t *iter) {
  TEN_ASSERT(json && ten_json_check_integrity(json), "Invalid argument.");
  TEN_ASSERT(iter, "Invalid argument.");
  return (ten_json_object_iter_t *)json_object_iter_next(json->json,
                                                         (void *)iter);
}
