//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#include "ten_utils/macro/memory.h"
#include "ten_utils/ten_config.h"

#include "include_internal/ten_utils/log/encryption.h"

void ten_log_encryption_init(ten_log_encryption_t *self) {
  self->encrypt_cb = NULL;
  self->deinit_cb = NULL;
  self->impl = NULL;
}

void ten_log_encrypt_data(ten_log_t *self, uint8_t *data, size_t data_len) {
  if (self->encryption.encrypt_cb) {
    self->encryption.encrypt_cb(data, data_len, self->encryption.impl);
  }
}
