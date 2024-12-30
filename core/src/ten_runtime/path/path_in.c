//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#include "include_internal/ten_runtime/path/path_in.h"

#include "include_internal/ten_runtime/path/path.h"
#include "include_internal/ten_runtime/path/path_table.h"
#include "ten_utils/lib/alloc.h"
#include "ten_utils/macro/check.h"

ten_path_in_t *ten_path_in_create(ten_path_table_t *table, const char *cmd_name,
                                  const char *parent_cmd_id, const char *cmd_id,
                                  ten_loc_t *src_loc, ten_loc_t *dest_loc,
                                  ten_msg_conversion_t *result_conversion) {
  ten_path_in_t *self = (ten_path_in_t *)TEN_MALLOC(sizeof(ten_path_in_t));
  TEN_ASSERT(self, "Failed to allocate memory.");

  ten_path_init((ten_path_t *)self, table, TEN_PATH_IN, cmd_name, parent_cmd_id,
                cmd_id, src_loc, dest_loc);
  self->base.result_conversion = result_conversion;

  return self;
}

void ten_path_in_destroy(ten_path_in_t *self) {
  TEN_ASSERT(self, "Should not happen.");

  ten_path_deinit((ten_path_t *)self);

  TEN_FREE(self);
}
