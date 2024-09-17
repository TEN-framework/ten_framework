//
// Copyright © 2024 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to https://github.com/TEN-framework/ten_framework/LICENSE for more
// information.
//
#pragma once

#include "ten_runtime/ten_config.h"

#include <stdbool.h>

#include "ten_runtime/app/app.h"
#include "src/ten_runtime/binding/go/interface/ten/common.h"
#include "ten_utils/lib/signature.h"

#define TEN_GO_APP_SIGNATURE 0xB97676170237FB01U

typedef struct ten_go_app_t {
  ten_signature_t signature;

  ten_go_bridge_t bridge;

  ten_app_t *c_app;
} ten_go_app_t;

TEN_RUNTIME_PRIVATE_API bool ten_go_app_check_integrity(ten_go_app_t *self);
