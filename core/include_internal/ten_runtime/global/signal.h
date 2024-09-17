//
// Copyright © 2024 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to https://github.com/TEN-framework/ten_framework/LICENSE for more
// information.
//
#include "ten_runtime/ten_config.h"

// TODO(Wei): Note that this function has no effect on windows now.
TEN_RUNTIME_API void ten_global_setup_signal_stuff(void);

TEN_RUNTIME_PRIVATE_API void ten_global_signal_alt_stack_create(void);

TEN_RUNTIME_PRIVATE_API void ten_global_signal_alt_stack_destroy(void);
