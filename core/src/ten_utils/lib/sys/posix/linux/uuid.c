//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
// This file is modified from https://github.com/gpakosz/uuid4/
//
#include "ten_utils/lib/uuid.h"

#include <sys/syscall.h>
#include <time.h>
#include <unistd.h>

#include "ten_utils/macro/check.h"

void ten_uuid4_seed(ten_uuid4_state_t *seed) {
  static uint64_t state0 = 0;

  struct timespec time;
  bool ok = clock_gettime(CLOCK_MONOTONIC_RAW, &time) == 0;
  TEN_ASSERT(ok, "Should not happen.");

  *seed = state0++ + ((uintptr_t)&time ^
                      (uint64_t)(time.tv_sec * 1000000000 + time.tv_nsec));

  uint32_t pid = (uint32_t)getpid();
  uint32_t tid = (uint32_t)syscall(SYS_gettid);
  *seed = *seed * 6364136223846793005U +
          ((uint64_t)(ten_uuid4_mix(ten_uuid4_hash(pid), ten_uuid4_hash(tid)))
           << 32);
  *seed = *seed * 6364136223846793005U + (uintptr_t)getpid;
  *seed = *seed * 6364136223846793005U + (uintptr_t)ten_uuid4_gen_from_seed;
}
