//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#pragma once

#include "ten_utils/ten_config.h"

#include <stddef.h>

#include "include_internal/ten_utils/backtrace/platform/posix/dwarf_internal/tag.h"

/**
 * @brief Represents a single DWARF abbreviation entry.
 *
 * DWARF abbreviations define the structure of Debugging Information Entries
 * (DIEs) in the .debug_info section. Each abbreviation specifies a tag type and
 * a set of attributes that describe program elements like functions, variables,
 * and types.
 *
 * Abbreviations are stored in the .debug_abbrev section and are referenced by
 * DIEs using the abbreviation code.
 */
typedef struct abbrev {
  // The abbreviation code - a unique identifier used by DIEs to reference
  // this abbreviation. Typically assigned sequentially starting from 1.
  uint64_t code;

  // The entry tag that identifies what kind of program construct this
  // abbreviation describes (e.g., DW_TAG_subprogram for functions).
  enum dwarf_tag tag;

  // Flag indicating whether DIEs using this abbreviation have child entries.
  // Non-zero if the DIE has children, zero otherwise.
  int has_children;

  // The number of attributes defined in this abbreviation.
  size_t num_attrs;

  // Pointer to an array of attribute definitions.
  // Each attribute specifies a name and form for the DIE's attributes.
  struct attr *attrs;
} abbrev;

/**
 * @brief Stores DWARF abbreviations for a compilation unit.
 *
 * This structure exists only during the parsing of a compilation unit.
 * While most DWARF readers use a hash table to map abbreviation IDs to
 * abbreviation entries, we optimize for GCC's behavior where abbreviation
 * IDs are issued in numerical order starting at 1. This allows us to use
 * a simple sorted vector for lookups instead of a more complex hash table.
 *
 * Abbreviations define the structure of Debugging Information Entries (DIEs)
 * in the .debug_info section, specifying what attributes each DIE contains.
 */
typedef struct abbrevs {
  // The number of abbreviation entries in the vector.
  size_t num_abbrevs;

  // Array of abbreviation entries, sorted by the code field for efficient
  // lookup. Each entry contains information about a specific DIE format.
  struct abbrev *abbrevs;
} abbrevs;
