//
// Copyright © 2024 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to https://github.com/TEN-framework/ten_framework/LICENSE for more
// information.
//
#include "ten_utils/lib/thread_once.h"

#include <pthread.h>

#include "ten_utils/macro/check.h"

int ten_thread_once(ten_thread_once_t *once_control,
                    void (*init_routine)(void)) {
  TEN_ASSERT(once_control && init_routine, "Invalid argument.");

  int rc = pthread_once(once_control, init_routine);
  if (rc != 0) {
    TEN_LOGE("Failed to pthread_once(): %d", rc);
  }

  return rc;
}
