//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#pragma once

#include "ten_utils/ten_config.h"

#include "ten_utils/log/log.h"

TEN_UTILS_PRIVATE_API void ten_log_encryption_init(ten_log_encryption_t *self);

TEN_UTILS_PRIVATE_API void ten_log_encrypt_data(ten_log_t *self, uint8_t *data,
                                                size_t data_len);
