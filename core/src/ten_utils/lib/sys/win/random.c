//
// Copyright © 2024 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to https://github.com/TEN-framework/ten_framework/LICENSE for more
// information.
//
#include "ten_utils/lib/random.h"

#include <Windows.h>
#include <bcrypt.h>

int ten_random(void *buf, size_t size) {
  return (int)BCryptGenRandom(NULL, buf, size, BCRYPT_USE_SYSTEM_PREFERRED_RNG);
}
