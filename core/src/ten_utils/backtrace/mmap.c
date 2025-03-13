//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#include "ten_utils/ten_config.h"

#include "include_internal/ten_utils/backtrace/mmap.h"

#include <assert.h>
#include <errno.h>
#include <stdint.h>
#include <stdio.h>
#include <sys/mman.h>
#include <sys/types.h>
#include <unistd.h>

/**
 * @file This file implements file views and memory allocation when mmap is
 * available.
 */

bool ten_mmap_init(ten_mmap_t *self, int descriptor, off_t offset,
                   uint64_t size) {
  assert(self && "Invalid argument.");
  assert((uint64_t)(size_t)size == size && "file size too large.");

  int pagesize = getpagesize();
  long in_page_offset = offset % pagesize;
  off_t offset_in_page_cnt = offset - in_page_offset;

  // Enlarge the size to the page boundary at the beginning and at the end.
  size += in_page_offset;
  size = (size + (pagesize - 1)) & ~(pagesize - 1);

  void *map =
      mmap(NULL, size, PROT_READ, MAP_PRIVATE, descriptor, offset_in_page_cnt);
  if (map == MAP_FAILED) {
    (void)fprintf(stderr, "Failed to mmap: %d\n", errno);
    assert(0 && "Failed to mmap.");
    return false;
  }

  self->data = (char *)map + in_page_offset;
  self->base = map;
  self->len = size;

  return true;
}

void ten_mmap_deinit(ten_mmap_t *self) {
  assert(self && "Invalid argument.");

  union {
    const void *cv;
    void *v;
  } const_cast;

  const_cast.cv = self->base;
  if (munmap(const_cast.v, self->len) < 0) {
    (void)fprintf(stderr, "Failed to munmap: %d\n", errno);
    assert(0 && "Failed to munmap.");
  }
}
