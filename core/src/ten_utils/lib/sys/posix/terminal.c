//
// Copyright © 2024 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to https://github.com/TEN-framework/ten_framework/LICENSE for more
// information.
//
#include "ten_utils/lib/terminal.h"

#include <stdio.h>
#include <sys/ioctl.h>
#include <unistd.h>

size_t ten_terminal_get_width_in_char(void) {
  struct winsize w;
  ioctl(fileno(stdout), TIOCGWINSZ, &w);
  return w.ws_col;
}

int ten_terminal_is_terminal(int fd) { return isatty(fd); }
