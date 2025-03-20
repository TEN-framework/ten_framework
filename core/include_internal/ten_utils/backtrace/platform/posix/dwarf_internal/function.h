//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#pragma once

#include "ten_utils/ten_config.h"

#include <stddef.h>

#include "include_internal/ten_utils/backtrace/vector.h"

/**
 * @brief Represents a function described in the DWARF debug information.
 *
 * This structure contains information about a function, including its name,
 * source location for inlined functions, and address ranges. For inlined
 * functions, it tracks the call site information to help with stack trace
 * generation.
 */
typedef struct function {
  // The name of the function.
  const char *name;

  // For inlined functions, the filename containing the call site.
  // This is NULL for non-inlined functions.
  const char *caller_filename;

  // For inlined functions, the line number of the call site.
  // This is 0 for non-inlined functions.
  int caller_lineno;

  // Array of address ranges associated with this function.
  // For inlined functions, these represent the specific ranges where
  // the function was inlined.
  struct function_addrs *function_addrs;

  // Number of entries in the function_addrs array.
  size_t function_addrs_count;
} function;

/**
 * @brief An address range for a function.
 *
 * This structure maps a range of program counter (PC) values to a specific
 * function. The range is inclusive at the low end and exclusive at the high
 * end, meaning a PC value matches if: low <= PC < high.
 */
typedef struct function_addrs {
  // Lower bound of the address range (inclusive).
  uintptr_t low;

  // Upper bound of the address range (exclusive).
  uintptr_t high;

  // Pointer to the function this address range belongs to.
  struct function *function;
} function_addrs;

/**
 * @brief A growable vector of function address ranges.
 *
 * This structure is used to collect and manage function address ranges
 * during DWARF parsing. It uses the ten_vector_t implementation for
 * dynamic memory management.
 */
typedef struct function_vector {
  // Underlying vector storage containing struct function_addrs elements.
  // Memory is managed by the ten_vector_t implementation.
  ten_vector_t vec;

  // Number of function address ranges currently stored in the vector.
  size_t count;
} function_vector;
