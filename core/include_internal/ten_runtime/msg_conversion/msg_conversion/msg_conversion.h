//
// This file is part of the TEN Framework project.
// See https://github.com/TEN-framework/ten_framework/LICENSE for license
// information.
//
#pragma once

#include "ten_runtime/ten_config.h"

#include "include_internal/ten_runtime/common/loc.h"
#include "ten_utils/container/list.h"
#include "ten_utils/lib/error.h"
#include "ten_utils/lib/signature.h"
#include "ten_utils/lib/smart_ptr.h"

#define TEN_MSG_CONVERSIONS_SIGNATURE 0x00C0F3A0F42BE9E9U

typedef struct ten_extension_info_t ten_extension_info_t;
typedef struct ten_extension_t ten_extension_t;
typedef struct ten_msg_and_result_conversion_operation_t
    ten_msg_and_result_conversion_operation_t;

// {
//   "type": "per_property",
//   "rules": [{
//     "path": "...",
//     "conversion_mode": "from_original",
//     "original_path": "..."
//   },{
//     "path": "...",
//     "conversion_mode": "fixed_value",
//     "value": "..."
//   }]
// }
typedef struct ten_msg_conversion_t {
  ten_signature_t signature;

  ten_loc_t src_loc;
  ten_string_t msg_name;

  ten_msg_and_result_conversion_operation_t
      *msg_and_result_conversion_operation;
} ten_msg_conversion_t;

TEN_RUNTIME_PRIVATE_API bool ten_msg_conversion_check_integrity(
    ten_msg_conversion_t *self);

TEN_RUNTIME_PRIVATE_API ten_msg_conversion_t *ten_msg_conversion_create(
    const char *msg_name);

TEN_RUNTIME_PRIVATE_API void ten_msg_conversion_destroy(
    ten_msg_conversion_t *self);

TEN_RUNTIME_PRIVATE_API bool ten_msg_conversion_merge(
    ten_list_t *msg_conversions, ten_msg_conversion_t *new_msg_conversion,
    ten_error_t *err);

/**
 * @param result [out] The type of each item is
 * 'ten_msg_and_result_conversion_t'
 */
TEN_RUNTIME_PRIVATE_API bool ten_extension_convert_msg(ten_extension_t *self,
                                                       ten_shared_ptr_t *msg,
                                                       ten_list_t *result,
                                                       ten_error_t *err);

TEN_RUNTIME_PRIVATE_API ten_msg_conversion_t *ten_msg_conversion_from_json(
    ten_json_t *json, ten_extension_info_t *src_extension_info,
    const char *cmd_name, ten_error_t *err);

TEN_RUNTIME_PRIVATE_API ten_msg_conversion_t *ten_msg_conversion_from_value(
    ten_value_t *value, ten_extension_info_t *src_extension_info,
    const char *cmd_name, ten_error_t *err);
