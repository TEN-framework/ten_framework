//
// This file is part of the TEN Framework project.
// See https://github.com/TEN-framework/ten_framework/LICENSE for license
// information.
//
#include "ten_utils/backtrace/backtrace.h"

#include <assert.h>
#include <execinfo.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "ten_utils/backtrace/common.h"
#include "ten_utils/lib/alloc.h"
#include "ten_utils/log/log.h"

/**
 * @note On Mac, we are currently using a simple method instead of a complicated
 * posix method to dump backtrace. So we only need a field of
 * 'ten_backtrace_common_t'.
 */
typedef struct ten_backtrace_mac_t {
  ten_backtrace_common_t common;
} ten_backtrace_mac_t;

ten_backtrace_t *ten_backtrace_create(void) {
  ten_backtrace_mac_t *self =
      ten_malloc_without_backtrace(sizeof(ten_backtrace_mac_t));
  assert(self && "Failed to allocate memory.");

  ten_backtrace_common_init(&self->common, ten_backtrace_default_dump_cb,
                            ten_backtrace_default_error_cb);

  return (ten_backtrace_t *)self;
}

void ten_backtrace_destroy(ten_backtrace_t *self) {
  assert(self && "Invalid argument.");

  ten_backtrace_common_deinit(self);

  ten_free_without_backtrace(self);
}

void ten_backtrace_dump(ten_backtrace_t *self, size_t skip) {
  assert(self && "Invalid argument.");

  // NOTE: Currently, the only way to get backtrace via
  // 'ten_backtrace_dump_posix' is to create .dsym for each executable and
  // libraries. Otherwise, it will show
  //
  // "no debug info in Mach-O executable".
  //
  // Therefore, we use the glibc builtin method (backtrace_symbols) for now.

  void *call_stack[128];
  int frames = backtrace(call_stack, 128);
  char **strs = backtrace_symbols(call_stack, frames);
  for (size_t i = skip; i < frames; ++i) {
    TEN_LOGE_AUX(((ten_backtrace_common_t *)self)->log, "%s", strs[i]);
  }
  free(strs);
}
