//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
// This file is modified from
// https://github.com/ianlancetaylor/libbacktrace [BSD license]
//
#include "ten_utils/ten_config.h"

#include <assert.h>
#include <fcntl.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <unistd.h>

#include "include_internal/ten_utils/backtrace/backtrace.h"
#include "include_internal/ten_utils/backtrace/platform/posix/config.h" // IWYU pragma: keep
#include "include_internal/ten_utils/backtrace/platform/posix/file.h"
#include "include_internal/ten_utils/backtrace/platform/posix/internal.h"
#include "ten_utils/lib/atomic_ptr.h"
#include "ten_utils/macro/mark.h"

#if defined(HAVE_MACH_O_DYLD_H)
#include <mach-o/dyld.h>

static char *macho_get_executable_path(ten_backtrace_t *self,
                                       ten_backtrace_error_func_t error_cb,
                                       void *data) {
  uint32_t len = 0;
  if (_NSGetExecutablePath(NULL, &len) == 0) {
    return NULL;
  }

  char *name = (char *)malloc(len);
  assert(name && "Failed to allocate memory.");

  if (_NSGetExecutablePath(name, &len) != 0) {
    free(name);
    return NULL;
  }

  return name;
}

#else /* !defined (HAVE_MACH_O_DYLD_H) */

#define macho_get_executable_path(self, error_cb, data) NULL

#endif /* !defined (HAVE_MACH_O_DYLD_H) */

/**
 * @brief Initialize the file_line information from the executable.
 *
 * @return 1 on success, 0 on failure.
 */
static int initialize_file_line_mechanism(ten_backtrace_t *self,
                                          ten_backtrace_error_func_t error_cb,
                                          void *data) {
  ten_backtrace_posix_t *posix_self = (ten_backtrace_posix_t *)self;
  assert(posix_self && "Invalid argument.");

  int64_t failed = ten_atomic_load(&posix_self->file_line_init_failed);
  if (failed) {
    (void)fprintf(stderr, "Failed to read executable information.");
    return 0;
  }

  ten_backtrace_get_file_line_func_t get_file_line_func =
      (ten_backtrace_get_file_line_func_t)ten_atomic_ptr_load(
          (void *)&posix_self->get_file_line_func);
  if (get_file_line_func != NULL) {
    return 1;
  }

  // We have not initialized the information. Do it now.

  int descriptor = -1;
  const char *filename = NULL;
  bool called_error_callback = false;

  for (size_t pass = 0; pass < 4; ++pass) {
    switch (pass) {
    case 0:
      filename = "/proc/self/exe";
      break;

    case 1:
      filename = "/proc/curproc/file";
      break;

    case 2: {
      char buf[64] = {0};

      int written =
          snprintf(buf, sizeof(buf), "/proc/%ld/object/a.out", (long)getpid());
      assert(written > 0);

      filename = buf;
      break;
    }

    case 3:
      filename = macho_get_executable_path(self, error_cb, data);
      break;

    default:
      abort();
      break;
    }

    if (filename == NULL) {
      // Failed to get the executable filename, try next way.
      continue;
    }

    bool does_not_exist = false;
    descriptor = ten_backtrace_open_file(filename, &does_not_exist);
    if (descriptor < 0 && !does_not_exist) {
      called_error_callback = true;
      break;
    }

    if (descriptor >= 0) {
      // Successfully open the executable file.
      break;
    }

    // Failed to open the executable filename, try next way.
  }

  if (descriptor < 0) {
    if (!called_error_callback) {
      error_cb(data, "Failed to find executable to open.", 0, NULL);
    }
    failed = 1;
  }

  if (!failed) {
    if (!ten_backtrace_init_posix(self, filename, descriptor, error_cb, data,
                                  &get_file_line_func)) {
      failed = 1;
    }
  }

  if (failed) {
    ten_atomic_store(&posix_self->file_line_init_failed, 1);
    return 0;
  }

  // TODO(Wei): Note that if two threads initialize at once, one of the data
  // sets may be leaked. Might need to consider a new way to avoid this.
  ten_atomic_ptr_store((void *)&posix_self->get_file_line_func,
                       get_file_line_func);

  return 1;
}

/**
 * @brief Given a PC, find the file name, line number, and function name.
 */
int ten_backtrace_get_file_line_info(ten_backtrace_t *self, uintptr_t pc,
                                     ten_backtrace_dump_file_line_func_t cb,
                                     ten_backtrace_error_func_t error_cb,
                                     void *data) {
  ten_backtrace_posix_t *posix_self = (ten_backtrace_posix_t *)self;
  assert(posix_self && "Invalid argument.");

  if (!initialize_file_line_mechanism(self, error_cb, data)) {
    return 0;
  }

  if (ten_atomic_load(&posix_self->file_line_init_failed)) {
    return 0;
  }

  return (posix_self->get_file_line_func)(self, pc, cb, error_cb, data);
}

/**
 * @brief Given a PC, find the symbol for it, and its value.
 */
int ten_backtrace_get_syminfo(ten_backtrace_t *self, uintptr_t pc,
                              ten_backtrace_dump_syminfo_func_t cb,
                              ten_backtrace_error_func_t error_cb, void *data) {
  ten_backtrace_posix_t *posix_self = (ten_backtrace_posix_t *)self;
  assert(posix_self && "Invalid argument.");

  if (!initialize_file_line_mechanism(self, error_cb, data)) {
    return 0;
  }

  if (ten_atomic_load(&posix_self->file_line_init_failed)) {
    return 0;
  }

  posix_self->get_syminfo_func(self, pc, cb, error_cb, data);

  return 1;
}

/**
 * @brief A ten_backtrace_dump_syminfo_func_t that can call into a
 * ten_backtrace_dump_file_line_func_t, used when we have a symbol table but no
 * debug info.
 */
void backtrace_dump_syminfo_to_dump_file_line_cb(
    ten_backtrace_t *self, uintptr_t pc, const char *symname,
    TEN_UNUSED uintptr_t sym_val, TEN_UNUSED uintptr_t sym_size, void *data) {
  struct backtrace_call_full *bt_data = (struct backtrace_call_full *)data;
  assert(bt_data && "Invalid argument.");

  bt_data->ret =
      bt_data->dump_file_line_cb(self, pc, NULL, 0, symname, bt_data->data);
}

/**
 * @brief An error callback that corresponds to
 * backtrace_syminfo_to_full_callback.
 */
void backtrace_dump_syminfo_to_dump_file_line_error_cb(ten_backtrace_t *self,
                                                       const char *msg,
                                                       int errnum, void *data) {
  struct backtrace_call_full *bt_data = (struct backtrace_call_full *)data;
  assert(bt_data && "Invalid argument.");

  bt_data->error_cb(self, msg, errnum, bt_data->data);
}
