//
// Copyright © 2024 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to https://github.com/TEN-framework/ten_framework/LICENSE for more
// information.
//
#include "ten_utils/lib/atomic.h"
#include "ten_utils/lib/thread.h"

void ten_thread_pause_cpu(void) {
  /* Set hardware multi-threading low priority */
  asm volatile("or 1,1,1");
  /* Set hardware multi-threading medium priority */
  asm volatile("or 2,2,2");
  ten_compiler_barrier();
}