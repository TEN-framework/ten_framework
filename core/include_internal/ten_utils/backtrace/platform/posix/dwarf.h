//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#pragma once

#include "ten_utils/ten_config.h"

#include <stddef.h>
#include <stdio.h>

#include "include_internal/ten_utils/backtrace/backtrace.h"
#include "include_internal/ten_utils/backtrace/platform/posix/dwarf_internal/abbreviation.h"  // IWYU pragma: keep
#include "include_internal/ten_utils/backtrace/platform/posix/dwarf_internal/attribute.h"  // IWYU pragma: keep
#include "include_internal/ten_utils/backtrace/platform/posix/dwarf_internal/buf.h"  // IWYU pragma: keep
#include "include_internal/ten_utils/backtrace/platform/posix/dwarf_internal/form.h"  // IWYU pragma: keep
#include "include_internal/ten_utils/backtrace/platform/posix/dwarf_internal/function.h"  // IWYU pragma: keep
#include "include_internal/ten_utils/backtrace/platform/posix/dwarf_internal/line.h"  // IWYU pragma: keep
#include "include_internal/ten_utils/backtrace/platform/posix/dwarf_internal/range_list.h"  // IWYU pragma: keep
#include "include_internal/ten_utils/backtrace/platform/posix/dwarf_internal/section.h"  // IWYU pragma: keep
#include "include_internal/ten_utils/backtrace/platform/posix/dwarf_internal/tag.h"  // IWYU pragma: keep
#include "include_internal/ten_utils/backtrace/platform/posix/dwarf_internal/unit.h"  // IWYU pragma: keep

#define IS_DIR_SEPARATOR(c) ((c) == '/')
#define IS_ABSOLUTE_PATH(f) (IS_DIR_SEPARATOR((f)[0]))

TEN_UTILS_PRIVATE_API uint64_t read_address(ten_backtrace_t *self,
                                            struct dwarf_buf *buf,
                                            int addrsize);

TEN_UTILS_PRIVATE_API int advance(ten_backtrace_t *self, struct dwarf_buf *buf,
                                  size_t count);

TEN_UTILS_PRIVATE_API uint16_t read_uint16(ten_backtrace_t *self,
                                           struct dwarf_buf *buf);

TEN_UTILS_PRIVATE_API uint32_t read_uint24(ten_backtrace_t *self,
                                           struct dwarf_buf *buf);

TEN_UTILS_PRIVATE_API uint32_t read_uint32(ten_backtrace_t *self,
                                           struct dwarf_buf *buf);

TEN_UTILS_PRIVATE_API uint64_t read_uint64(ten_backtrace_t *self,
                                           struct dwarf_buf *buf);

TEN_UTILS_PRIVATE_API uint64_t read_offset(ten_backtrace_t *self,
                                           struct dwarf_buf *buf,
                                           int is_dwarf64);

TEN_UTILS_PRIVATE_API uint64_t read_uleb128(ten_backtrace_t *self,
                                            struct dwarf_buf *buf);

TEN_UTILS_PRIVATE_API int64_t read_sleb128(ten_backtrace_t *self,
                                           struct dwarf_buf *buf);

TEN_UTILS_PRIVATE_API const char *read_string(ten_backtrace_t *self,
                                              struct dwarf_buf *buf);

TEN_UTILS_PRIVATE_API unsigned char read_byte(ten_backtrace_t *self,
                                              struct dwarf_buf *buf);

TEN_UTILS_PRIVATE_API signed char read_sbyte(ten_backtrace_t *self,
                                             struct dwarf_buf *buf);

TEN_UTILS_PRIVATE_API void dwarf_buf_error(ten_backtrace_t *self,
                                           struct dwarf_buf *buf,
                                           const char *msg, int errnum);

TEN_UTILS_PRIVATE_API int resolve_string(
    ten_backtrace_t *self, const dwarf_sections *dwarf_sections, int is_dwarf64,
    int is_bigendian, uint64_t str_offsets_base, const attr_val *val,
    ten_backtrace_error_func_t error_cb, void *data, const char **string);

TEN_UTILS_PRIVATE_API uint64_t read_initial_length(ten_backtrace_t *self,
                                                   struct dwarf_buf *buf,
                                                   int *is_dwarf64);
