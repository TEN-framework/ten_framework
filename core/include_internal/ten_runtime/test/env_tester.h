//
// Copyright © 2024 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#include "ten_runtime/ten_config.h"

#include "include_internal/ten_runtime/binding/common.h"
#include "ten_utils/lib/signature.h"

#define TEN_ENV_TESTER_SIGNATURE 0x66C619FBA7DC8BD9U

typedef struct ten_extension_tester_t ten_extension_tester_t;

typedef struct ten_env_tester_t {
  ten_binding_handle_t binding_handle;

  ten_signature_t signature;
  ten_extension_tester_t *tester;
} ten_env_tester_t;

TEN_RUNTIME_PRIVATE_API ten_env_tester_t *ten_env_tester_create(
    ten_extension_tester_t *tester);

TEN_RUNTIME_PRIVATE_API void ten_env_tester_destroy(ten_env_tester_t *self);
