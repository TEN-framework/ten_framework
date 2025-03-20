//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#include "include_internal/ten_utils/backtrace/buffer.h"

#include <assert.h>
#include <inttypes.h>
#include <stdio.h>
#include <string.h>

#include "include_internal/ten_utils/backtrace/file.h"

#define BUFFER_LINE_MAX_LEN 1024

void ten_backtrace_buffer_init(ten_backtrace_buffer_t *self, char *data,
                               size_t capacity) {
  assert(self && "Invalid argument.");
  assert(data && "Invalid argument.");
  assert(capacity > 0 && "Invalid argument.");

  if (!self || !data || capacity == 0) {
    return;
  }

  self->data = data;
  self->capacity = capacity;
  self->length = 0;
  self->overflow = 0;

  // Ensure the buffer starts with a null character.
  self->data[0] = '\0';
}

int ten_backtrace_buffer_dump_cb(ten_backtrace_t *self, uintptr_t pc,
                                 const char *filename, int lineno,
                                 const char *function, void *data) {
  assert(self && "Invalid argument.");
  assert(data && "Invalid argument.");

  ten_backtrace_buffer_t *buffer = (ten_backtrace_buffer_t *)data;
  if (!buffer) {
    return -1;
  }

  // If overflow, do not continue writing.
  if (buffer->overflow) {
    // Return 0 to continue processing the next frame.
    return 0;
  }

  // Ensure we have valid strings to print.
  const char *safe_filename = filename ? filename : "<unknown file>";
  const char *safe_function = function ? function : "<unknown function>";

  // Normalize the filename to remove ".." path components
  char normalized_path[NORMALIZE_PATH_BUF_SIZE] = {0};
  if (filename && ten_backtrace_normalize_path(safe_filename, normalized_path,
                                               sizeof(normalized_path))) {
    // Use the normalized path if successful
    safe_filename = normalized_path;
  }

  // Format the output line.
  char line[BUFFER_LINE_MAX_LEN];
  int result = snprintf(line, sizeof(line), "%s@%s:%d (0x%0" PRIxPTR ")\n",
                        safe_function, safe_filename, lineno, pc);
  if (result < 0) {
    assert(0 && "Failed to format output.");
    return -1;
  }

  // Calculate the new length.
  size_t line_len = (size_t)result;
  size_t new_length = buffer->length + line_len;

  // Check if it will cause buffer overflow.
  if (new_length >= buffer->capacity) {
    buffer->overflow = 1;

    // Return 0 to continue processing the next frame.
    return 0;
  }

  // Append the line to the buffer.
  memcpy(buffer->data + buffer->length, line, line_len);

  buffer->length = new_length;

  // Ensure the buffer ends with a null character.
  buffer->data[buffer->length] = '\0';

  // Return 0 to continue processing the next frame.
  return 0;
}
