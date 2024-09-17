//
// Copyright © 2024 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to https://github.com/TEN-framework/ten_framework/LICENSE for more
// information.
//
#include "include_internal/ten_runtime/common/preserved_metadata.h"

static char metadata[] = "version=0.2.0";

void ten_preserved_metadata(void) {
  ((char volatile *)metadata)[0] = metadata[0];
}