//
// Copyright © 2024 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to https://github.com/TEN-framework/ten_framework/LICENSE for more
// information.
//
#pragma once

#include "ten_utils/ten_config.h"

#include <stddef.h>

TEN_UTILS_API char *ten_sanitizer_memory_strndup(const char *str, size_t size,
                                                 const char *file_name,
                                                 uint32_t lineno,
                                                 const char *func_name);
