//
// This file is part of the TEN Framework project.
// See https://github.com/TEN-framework/ten_framework/LICENSE for license
// information.
//
#pragma once

#include "ten_utils/ten_config.h"

#include <stdbool.h>

#include "ten_utils/lib/signature.h"
#include "ten_utils/lib/string.h"

#define TEN_LISTNODE_SIGNATURE 0x00D7B10E642B105CU

typedef struct ten_listnode_t ten_listnode_t;
typedef struct ten_list_t ten_list_t;

struct ten_listnode_t {
  ten_signature_t signature;
  ten_listnode_t *next, *prev;
  void (*destroy)(ten_listnode_t *);
};

#include "ten_utils/container/list_node_int32.h"      // IWYU pragma: keep
#include "ten_utils/container/list_node_ptr.h"        // IWYU pragma: keep
#include "ten_utils/container/list_node_smart_ptr.h"  // IWYU pragma: keep
#include "ten_utils/container/list_node_str.h"        // IWYU pragma: keep

TEN_UTILS_PRIVATE_API bool ten_listnode_check_integrity(ten_listnode_t *self);

TEN_UTILS_PRIVATE_API void ten_listnode_init(ten_listnode_t *self,
                                             void *destroy);

TEN_UTILS_API void ten_listnode_destroy(ten_listnode_t *self);
TEN_UTILS_API void ten_listnode_destroy_only(ten_listnode_t *self);
