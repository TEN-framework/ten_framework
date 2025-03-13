//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#include "ten_utils/ten_config.h"

#include <assert.h>
#include <fcntl.h>
#include <stdbool.h>
#include <stdio.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <unistd.h>

// Mac OS X 10.6 does not support O_CLOEXEC.
#ifndef O_CLOEXEC
#define O_CLOEXEC 0
#endif

// Mac OS does not support FD_CLOEXEC.
#ifndef FD_CLOEXEC
#define FD_CLOEXEC 1
#endif

int ten_backtrace_open_file(const char *filename, bool *does_not_exist) {
  if (does_not_exist != NULL) {
    *does_not_exist = false;
  }

  int fd = open(filename, (O_RDONLY | O_CLOEXEC));
  if (fd < 0) {
    if (does_not_exist != NULL) {
      *does_not_exist = true;
    } else {
      (void)fprintf(stderr, "Failed to open %s\n", filename);
    }

    return -1;
  }

  // Set FD_CLOEXEC just in case the kernel does not support O_CLOEXEC. It
  // doesn't matter if this fails for some reason.
  fcntl(fd, F_SETFD, FD_CLOEXEC);

  return fd;
}

bool ten_backtrace_close_file(int fd) {
  if (close(fd) < 0) {
    (void)fprintf(stderr, "Failed to close %d\n", fd);
    return false;
  }
  return true;
}
