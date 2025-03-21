//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#include "ten_utils/ten_config.h"

#include <stdlib.h>

#include "include_internal/ten_utils/backtrace/platform/posix/dwarf_internal/unit.h"

// Compare a unit offset against a unit for bsearch.
static int units_search(const void *vkey, const void *ventry) {
  const size_t *key = (const size_t *)vkey;
  const unit *entry = *((const unit *const *)ventry);
  size_t offset = *key;
  if (offset < entry->low_offset) {
    return -1;
  } else if (offset >= entry->high_offset) {
    return 1;
  } else {
    return 0;
  }
}

// Find a unit in PU containing OFFSET.
unit *find_unit(unit **pu, size_t units_count, size_t offset) {
  unit **u = NULL;
  u = bsearch(&offset, (const void *)pu, units_count, sizeof(unit *),
              units_search);
  return u == NULL ? NULL : *u;
}
