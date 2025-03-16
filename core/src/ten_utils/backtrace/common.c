//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#include "include_internal/ten_utils/backtrace/common.h"

#include <assert.h>
#include <errno.h>
#include <inttypes.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "include_internal/ten_utils/backtrace/backtrace.h"
#include "include_internal/ten_utils/backtrace/file.h"
#include "ten_utils/backtrace/backtrace.h"
#include "ten_utils/macro/mark.h"

// Initialize global backtrace pointer to NULL.
ten_backtrace_t *g_ten_backtrace = NULL;

#if defined(OS_WINDOWS)
// There is no 'strerror_r in Windows, use strerror_s instead. Note that the
// parameter order of strerror_s is different from strerror_r.
#define strerror_r(errno, buf, len) strerror_s(buf, len, errno)
#endif

/**
 * @brief Call strerror_r and get the full error message. Allocate memory for
 * the entire string with malloc. Return string. Caller must free string.
 * If malloc fails, return NULL.
 */
static char *ten_strerror(int errnum) {
  if (errnum <= 0) {
    // No valid error number provided.
    assert(0 && "Invalid error number.");
    return NULL;
  }

  size_t size = 1024;
  char *buf = malloc(size);
  if (!buf) {
    assert(0 && "Failed to allocate memory.");
    return NULL;
  }

  // Try to get error string.
  while (strerror_r(errnum, buf, size) == -1) {
    // Check if error is not just due to buffer size.
    if (errno != ERANGE) {
      assert(0 && "Failed to get error string.");
      free(buf);
      return NULL;
    }

    size_t new_size = size * 2;
    if (new_size < size) { // Check for overflow.
      assert(0 && "Overflow.");
      free(buf);
      return NULL;
    }

    char *new_buf = realloc(buf, new_size);
    if (!new_buf) {
      assert(0 && "Failed to reallocate memory.");
      free(buf);
      return NULL;
    }

    buf = new_buf;
    size = new_size;
  }

  return buf;
}

int ten_backtrace_default_dump_cb(ten_backtrace_t *self_, uintptr_t pc,
                                  const char *filename, int lineno,
                                  const char *function, TEN_UNUSED void *data) {
  ten_backtrace_common_t *self = (ten_backtrace_common_t *)self_;
  if (!self) {
    assert(0 && "Invalid argument.");
    return -1;
  }

  // Ensure we have valid strings to print.
  const char *safe_filename = filename ? filename : "<unknown file>";
  const char *safe_function = function ? function : "<unknown function>";

  // Normalize the filename to remove ".." path components
  char normalized_path[4096] = {0};
  if (filename && ten_backtrace_normalize_path(safe_filename, normalized_path,
                                               sizeof(normalized_path))) {
    // Use the normalized path if successful
    safe_filename = normalized_path;
  }

#if defined(OS_WINDOWS)
  // On Windows, ensure we're using consistent path separators in output.
  // This is only needed if normalization didn't happen or failed.
  if (safe_filename != normalized_path && filename) {
    char windows_path[4096] = {0};
    strncpy(windows_path, safe_filename, sizeof(windows_path) - 1);

    // Replace forward slashes with backslashes for display
    for (char *p = windows_path; *p != '\0'; p++) {
      if (*p == '/') {
        *p = '\\';
      }
    }
    safe_filename = windows_path;
  }
#endif

  int result = fprintf(stderr, "%s@%s:%d (0x%0" PRIxPTR ")\n", safe_function,
                       safe_filename, lineno, pc);

  // Check if fprintf failed.
  if (result < 0) {
    assert(0 && "Failed to fprintf(stderr).");
    return -1;
  }

  return 0;
}

void ten_backtrace_default_error_cb(ten_backtrace_t *self_, const char *msg,
                                    int errnum, TEN_UNUSED void *data) {
  ten_backtrace_common_t *self = (ten_backtrace_common_t *)self_;
  if (!self) {
    assert(0 && "Invalid argument.");
    return;
  }

  // Ensure we have a valid message.
  const char *safe_msg = msg ? msg : "<unknown error>";

  // Print the error message
  if (fprintf(stderr, "%s", safe_msg) < 0) {
    assert(0 && "Failed to fprintf(stderr).");
    return; // Error writing to stderr, just return.
  }

  // Print error details if available.
  if (errnum > 0) {
    char *buf = ten_strerror(errnum);
    if (buf) {
      if (fprintf(stderr, ": %s", buf) < 0) {
        assert(0 && "Failed to fprintf(stderr).");
      }
      free(buf);
    } else {
      // If ten_strerror failed, print the raw error number.
      if (fprintf(stderr, ": error %d", errnum) < 0) {
        assert(0 && "Failed to fprintf(stderr).");
      }
    }
  }

  // Add newline for better formatting.
  if (fprintf(stderr, "\n") < 0) {
    assert(0 && "Failed to fprintf(stderr).");
  }
}

void ten_backtrace_common_init(ten_backtrace_common_t *self,
                               ten_backtrace_dump_file_line_func_t dump_cb,
                               ten_backtrace_error_func_t error_cb) {
  if (!self) {
    assert(0 && "Invalid argument.");
    return;
  }

  // Use provided callbacks or default ones if NULL.
  self->dump_cb = dump_cb ? dump_cb : ten_backtrace_default_dump_cb;
  self->error_cb = error_cb ? error_cb : ten_backtrace_default_error_cb;
}

void ten_backtrace_common_deinit(ten_backtrace_t *self) {
  if (!self) {
    assert(0 && "Invalid argument.");
    return;
  }

  ten_backtrace_common_t *common_self = (ten_backtrace_common_t *)self;

  // Release any resources that might be held. For now, there's nothing specific
  // to clean up.
}

void ten_backtrace_create_global(void) {
  // Only create if not already created.
  if (!g_ten_backtrace) {
    g_ten_backtrace = ten_backtrace_create();
    if (!g_ten_backtrace) {
      assert(0 && "Failed to create global backtrace.");
    }
  }
}

void ten_backtrace_destroy_global(void) {
  if (g_ten_backtrace) {
    ten_backtrace_destroy(g_ten_backtrace);
    g_ten_backtrace = NULL; // Clear the pointer after destruction.
  }
}

void ten_backtrace_dump_global(size_t skip) {
  // Check if global backtrace is available.
  if (!g_ten_backtrace) {
    if (fprintf(stderr, "Error: Global backtrace object not initialized.\n") <
        0) {
      assert(0 && "Failed to fprintf(stderr).");
    }
    return;
  }

  const char *enable_backtrace_dump = getenv("TEN_ENABLE_BACKTRACE_DUMP");
  // getenv might return NULL, so check that first.
  if (enable_backtrace_dump && !strcmp(enable_backtrace_dump, "true")) {
    ten_backtrace_dump(g_ten_backtrace, skip);
  } else {
    if (fprintf(stderr,
                "Backtrace dump is disabled by TEN_ENABLE_BACKTRACE_DUMP.\n") <
        0) {
      assert(0 && "Failed to fprintf(stderr).");
    }
  }
}
