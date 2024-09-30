//
// Copyright © 2024 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#include "ten_runtime/ten_config.h"

#include "ten_utils/lib/signature.h"
#include "ten_utils/lib/smart_ptr.h"

#define TEN_ENV_TESTER_SIGNATURE 0x66C619FBA7DC8BD9U

typedef struct ten_extension_tester_t ten_extension_tester_t;

typedef struct ten_env_tester_t {
  ten_signature_t signature;
  ten_extension_tester_t *tester;
} ten_env_tester_t;

TEN_RUNTIME_PRIVATE_API ten_env_tester_t *ten_env_tester_create(
    ten_extension_tester_t *tester);

TEN_RUNTIME_PRIVATE_API void ten_env_tester_destroy(ten_env_tester_t *self);

typedef void (*ten_env_tester_cmd_result_handler_func_t)(
    ten_env_tester_t *self, ten_shared_ptr_t *cmd_result, void *user_data);

TEN_RUNTIME_API void ten_env_tester_send_cmd(
    ten_env_tester_t *self, ten_shared_ptr_t *cmd,
    ten_env_tester_cmd_result_handler_func_t handler, void *user_data);

TEN_RUNTIME_API void ten_env_tester_stop_test(ten_env_tester_t *self);
