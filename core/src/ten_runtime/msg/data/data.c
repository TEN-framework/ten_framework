//
// Copyright © 2024 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#include "ten_runtime/msg/data/data.h"

#include "include_internal/ten_runtime/msg/data/field/field_info.h"
#include "include_internal/ten_runtime/msg/locked_res.h"
#include "include_internal/ten_runtime/msg/msg.h"
#include "include_internal/ten_utils/value/value_path.h"
#include "ten_runtime/common/errno.h"
#include "ten_utils/lib/alloc.h"
#include "ten_utils/lib/buf.h"
#include "ten_utils/lib/json.h"
#include "ten_utils/lib/signature.h"
#include "ten_utils/lib/smart_ptr.h"
#include "ten_utils/macro/check.h"
#include "ten_utils/macro/mark.h"
#include "ten_utils/value/value_get.h"

bool ten_raw_data_check_integrity(ten_data_t *self) {
  TEN_ASSERT(self, "Should not happen.");

  if (ten_signature_get(&self->signature) !=
      (ten_signature_t)TEN_DATA_SIGNATURE) {
    return false;
  }

  if (self->msg_hdr.type != TEN_MSG_TYPE_DATA) {
    return false;
  }

  return true;
}

static ten_data_t *ten_raw_data_create(void) {
  ten_data_t *self = (ten_data_t *)TEN_MALLOC(sizeof(ten_data_t));
  TEN_ASSERT(self, "Failed to allocate memory.");

  ten_raw_msg_init(&self->msg_hdr, TEN_MSG_TYPE_DATA);
  ten_signature_set(&self->signature, TEN_DATA_SIGNATURE);
  ten_value_init_buf(&self->buf, 0);

  return self;
}

void ten_raw_data_destroy(ten_data_t *self) {
  TEN_ASSERT(self && ten_raw_data_check_integrity(self), "Should not happen.");

  ten_raw_msg_deinit(&self->msg_hdr);
  ten_value_deinit(&self->buf);

  TEN_FREE(self);
}

ten_shared_ptr_t *ten_data_create(void) {
  ten_data_t *self = ten_raw_data_create();
  return ten_shared_ptr_create(self, ten_raw_data_destroy);
}

static void ten_raw_data_set_buf_with_move(ten_data_t *self, ten_buf_t *buf) {
  TEN_ASSERT(self && ten_raw_data_check_integrity(self), "Should not happen.");
  ten_buf_move(ten_value_peek_buf(&self->buf), buf);
}

static ten_buf_t *ten_raw_data_peek_buf(ten_data_t *self) {
  TEN_ASSERT(self && ten_raw_data_check_integrity(self), "Should not happen.");
  return ten_value_peek_buf(&self->buf);
}

ten_buf_t *ten_data_peek_buf(ten_shared_ptr_t *self_) {
  TEN_ASSERT(self_, "Should not happen.");
  ten_data_t *self = ten_shared_ptr_get_data(self_);
  return ten_raw_data_peek_buf(self);
}

void ten_data_set_buf_with_move(ten_shared_ptr_t *self_, ten_buf_t *buf) {
  TEN_ASSERT(self_, "Should not happen.");

  ten_data_t *self = ten_shared_ptr_get_data(self_);
  ten_raw_data_set_buf_with_move(self, buf);
}

void ten_raw_data_buf_copy(ten_msg_t *self, ten_msg_t *src,
                           TEN_UNUSED ten_list_t *excluded_field_ids) {
  TEN_ASSERT(self, "Invalid argument.");
  TEN_ASSERT(src, "Invalid argument.");

  ten_data_t *src_data = (ten_data_t *)src;
  ten_data_t *self_data = (ten_data_t *)self;

  if (ten_buf_get_size(ten_value_peek_buf(&src_data->buf))) {
    ten_buf_init_with_copying_data(ten_value_peek_buf(&self_data->buf),
                                   ten_value_peek_buf(&src_data->buf)->data,
                                   ten_value_peek_buf(&src_data->buf)->size);
  }
}

ten_msg_t *ten_raw_data_as_msg_clone(ten_msg_t *self,
                                     ten_list_t *excluded_field_ids) {
  TEN_ASSERT(self && ten_raw_msg_check_integrity(self) &&
                 ten_raw_msg_get_type(self) == TEN_MSG_TYPE_DATA,
             "Should not happen.");

  ten_data_t *new_data = ten_raw_data_create();
  TEN_ASSERT(new_data, "Failed to allocate memory.");

  for (size_t i = 0; i < ten_data_fields_info_size; ++i) {
    if (excluded_field_ids) {
      bool skip = false;

      ten_list_foreach (excluded_field_ids, iter) {
        if (ten_data_fields_info[i].field_id ==
            ten_int32_listnode_get(iter.node)) {
          skip = true;
          break;
        }
      }

      if (skip) {
        continue;
      }
    }

    ten_msg_copy_field_func_t copy_field = ten_data_fields_info[i].copy_field;
    if (copy_field) {
      copy_field((ten_msg_t *)new_data, self, excluded_field_ids);
    }
  }

  return (ten_msg_t *)new_data;
}

static bool ten_raw_data_init_from_json(ten_data_t *self, ten_json_t *json,
                                        ten_error_t *err) {
  TEN_ASSERT(self && ten_raw_data_check_integrity(self), "Should not happen.");
  TEN_ASSERT(json && ten_json_check_integrity(json), "Should not happen.");

  for (size_t i = 0; i < ten_data_fields_info_size; ++i) {
    ten_msg_get_field_from_json_func_t get_field_from_json =
        ten_data_fields_info[i].get_field_from_json;
    if (get_field_from_json) {
      if (!get_field_from_json((ten_msg_t *)self, json, err)) {
        return false;
      }
    }
  }

  return true;
}

static ten_data_t *ten_raw_data_create_from_json(ten_json_t *json,
                                                 ten_error_t *err) {
  TEN_ASSERT(json, "Should not happen.");

  ten_data_t *data = ten_raw_data_create();
  TEN_ASSERT(data && ten_raw_data_check_integrity((ten_data_t *)data),
             "Should not happen.");

  if (!ten_raw_data_init_from_json(data, json, err)) {
    ten_raw_data_destroy(data);
    return NULL;
  }

  return data;
}

ten_msg_t *ten_raw_data_as_msg_create_from_json(ten_json_t *json,
                                                ten_error_t *err) {
  TEN_ASSERT(json, "Should not happen.");

  return (ten_msg_t *)ten_raw_data_create_from_json(json, err);
}

bool ten_raw_data_as_msg_init_from_json(ten_msg_t *self, ten_json_t *json,
                                        ten_error_t *err) {
  TEN_ASSERT(self && ten_raw_data_check_integrity((ten_data_t *)self),
             "Should not happen.");
  TEN_ASSERT(json && ten_json_check_integrity(json), "Should not happen.");

  return ten_raw_data_init_from_json((ten_data_t *)self, json, err);
}

ten_json_t *ten_raw_data_as_msg_to_json(ten_msg_t *self, ten_error_t *err) {
  TEN_ASSERT(self && ten_raw_msg_check_integrity(self) &&
                 ten_raw_msg_get_type(self) == TEN_MSG_TYPE_DATA,
             "Should not happen.");

  ten_json_t *json = ten_json_create_object();
  TEN_ASSERT(json, "Should not happen.");

  if (!ten_raw_msg_put_field_to_json(self, json, err)) {
    ten_json_destroy(json);
    return NULL;
  }

  return json;
}

bool ten_raw_data_loop_all_fields(ten_msg_t *self,
                                  ten_raw_msg_process_one_field_func_t cb,
                                  void *user_data, ten_error_t *err) {
  // TODO(Wei): Implement the fields traversal algorithm here.
  return cb(self, NULL, user_data, err);
}

static uint8_t *ten_raw_data_alloc_buf(ten_data_t *self, size_t size) {
  uint8_t *data = ten_value_peek_buf(&self->buf)->data;
  if (data) {
    TEN_ASSERT(0, "Should not happen.");
    return NULL;
  }

  if (!ten_value_init_buf(&self->buf, size)) {
    return NULL;
  }

  return ten_value_peek_buf(&self->buf)->data;
}

uint8_t *ten_data_alloc_buf(ten_shared_ptr_t *self, size_t size) {
  TEN_ASSERT(self, "Should not happen.");
  return ten_raw_data_alloc_buf(ten_shared_ptr_get_data(self), size);
}

bool ten_raw_data_like_set_ten_property(ten_msg_t *self, ten_list_t *paths,
                                        ten_value_t *value, ten_error_t *err) {
  TEN_ASSERT(self && ten_raw_msg_check_integrity(self), "Invalid argument.");
  TEN_ASSERT(paths && ten_list_check_integrity(paths),
             "path should not be empty.");
  TEN_ASSERT(value && ten_value_check_integrity(value), "Invalid argument.");

  bool success = true;

  ten_error_t tmp_err;
  bool use_tmp_err = false;
  if (!err) {
    use_tmp_err = true;
    ten_error_init(&tmp_err);
    err = &tmp_err;
  }

  ten_list_foreach (paths, item_iter) {
    ten_value_path_item_t *item = ten_ptr_listnode_get(item_iter.node);
    TEN_ASSERT(item, "Invalid argument.");

    switch (item->type) {
      case TEN_VALUE_PATH_ITEM_TYPE_OBJECT_ITEM: {
        if (!strcmp(TEN_STR_NAME,
                    ten_string_get_raw_str(&item->obj_item_str))) {
          if (ten_value_is_string(value)) {
            ten_string_init_from_c_str(&self->name,
                                       ten_value_peek_string(value),
                                       strlen(ten_value_peek_string(value)));
            success = true;
          } else {
            success = false;
          }
        }
        break;
      }

      default:
        break;
    }
  }

  if (use_tmp_err) {
    ten_error_deinit(&tmp_err);
  }

  return success;
}

static ten_data_t *ten_raw_data_create_from_json_string(const char *json_str,
                                                        ten_error_t *err) {
  ten_json_t *json = ten_json_from_string(json_str, err);
  if (json == NULL) {
    return NULL;
  }

  ten_data_t *data = ten_raw_data_create_from_json(json, err);

  ten_json_destroy(json);

  return data;
}

ten_shared_ptr_t *ten_data_create_from_json_string(const char *json_str,
                                                   ten_error_t *err) {
  ten_data_t *data = ten_raw_data_create_from_json_string(json_str, err);
  return ten_shared_ptr_create(data, ten_raw_data_destroy);
}

bool ten_raw_data_check_type_and_name(ten_msg_t *self, const char *type_str,
                                      const char *name_str, ten_error_t *err) {
  TEN_ASSERT(self && ten_raw_msg_check_integrity(self), "Invalid argument.");

  if (type_str) {
    if (strcmp(type_str, TEN_STR_DATA) != 0) {
      if (err) {
        ten_error_set(err, TEN_ERRNO_GENERIC,
                      "Incorrect message type for data: %s", type_str);
      }
      return false;
    }
  }

  if (name_str) {
    if (strncmp(name_str, TEN_STR_MSG_NAME_TEN_NAMESPACE_PREFIX,
                strlen(TEN_STR_MSG_NAME_TEN_NAMESPACE_PREFIX)) == 0) {
      if (err) {
        ten_error_set(err, TEN_ERRNO_GENERIC,
                      "Incorrect message name for data: %s", name_str);
      }
      return false;
    }
  }

  return true;
}
