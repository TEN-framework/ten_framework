//
// This file is part of the TEN Framework project.
// See https://github.com/TEN-framework/ten_framework/LICENSE for license
// information.
//
#include "include_internal/ten_runtime/common/preserved_metadata.h"

static char metadata[] = "version=0.2.0";

void ten_preserved_metadata(void) {
  ((char volatile *)metadata)[0] = metadata[0];
}