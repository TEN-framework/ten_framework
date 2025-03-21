//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#include "ten_utils/ten_config.h"

#include <string.h>

#include "include_internal/ten_utils/backtrace/backtrace.h"
#include "include_internal/ten_utils/backtrace/platform/posix/dwarf.h"
#include "include_internal/ten_utils/backtrace/platform/posix/dwarf_internal/buf.h"

// Read one zero-terminated string from BUF and advance past the string.
const char *read_string(ten_backtrace_t *self, dwarf_buf *buf) {
  const char *p = (const char *)buf->buf;
  size_t len = strnlen(p, buf->left);

  // - If len == left, we ran out of buffer before finding the zero terminator.
  //   Generate an error by advancing len + 1.
  // - If len < left, advance by len + 1 to skip past the zero terminator.
  size_t count = len + 1;

  if (!advance(self, buf, count)) {
    return NULL;
  }

  return p;
}

// Read one byte from BUF and advance 1 byte.
unsigned char read_byte(ten_backtrace_t *self, dwarf_buf *buf) {
  const unsigned char *p = buf->buf;

  if (!advance(self, buf, 1)) {
    return 0;
  }
  return p[0];
}

// Read a signed char from BUF and advance 1 byte.
signed char read_sbyte(ten_backtrace_t *self, dwarf_buf *buf) {
  const unsigned char *p = buf->buf;

  if (!advance(self, buf, 1)) {
    return 0;
  }
  return (*p ^ 0x80) - 0x80;
}

// Read a uint16 from BUF and advance 2 bytes.
uint16_t read_uint16(ten_backtrace_t *self, dwarf_buf *buf) {
  const unsigned char *p = buf->buf;

  if (!advance(self, buf, 2)) {
    return 0;
  }

  if (buf->is_bigendian) {
    return ((uint16_t)p[0] << 8) | (uint16_t)p[1];
  } else {
    return ((uint16_t)p[1] << 8) | (uint16_t)p[0];
  }
}

// Read a 24 bit value from BUF and advance 3 bytes.
uint32_t read_uint24(ten_backtrace_t *self, dwarf_buf *buf) {
  const unsigned char *p = buf->buf;

  if (!advance(self, buf, 3)) {
    return 0;
  }

  if (buf->is_bigendian) {
    return (((uint32_t)p[0] << 16) | ((uint32_t)p[1] << 8) | (uint32_t)p[2]);
  } else {
    return (((uint32_t)p[2] << 16) | ((uint32_t)p[1] << 8) | (uint32_t)p[0]);
  }
}

// Read a uint32 from BUF and advance 4 bytes.
uint32_t read_uint32(ten_backtrace_t *self, dwarf_buf *buf) {
  const unsigned char *p = buf->buf;

  if (!advance(self, buf, 4)) {
    return 0;
  }

  if (buf->is_bigendian) {
    return (((uint32_t)p[0] << 24) | ((uint32_t)p[1] << 16) |
            ((uint32_t)p[2] << 8) | (uint32_t)p[3]);
  } else {
    return (((uint32_t)p[3] << 24) | ((uint32_t)p[2] << 16) |
            ((uint32_t)p[1] << 8) | (uint32_t)p[0]);
  }
}

// Read a uint64 from BUF and advance 8 bytes.
uint64_t read_uint64(ten_backtrace_t *self, dwarf_buf *buf) {
  const unsigned char *p = buf->buf;

  if (!advance(self, buf, 8)) {
    return 0;
  }

  if (buf->is_bigendian) {
    return (((uint64_t)p[0] << 56) | ((uint64_t)p[1] << 48) |
            ((uint64_t)p[2] << 40) | ((uint64_t)p[3] << 32) |
            ((uint64_t)p[4] << 24) | ((uint64_t)p[5] << 16) |
            ((uint64_t)p[6] << 8) | (uint64_t)p[7]);
  } else {
    return (((uint64_t)p[7] << 56) | ((uint64_t)p[6] << 48) |
            ((uint64_t)p[5] << 40) | ((uint64_t)p[4] << 32) |
            ((uint64_t)p[3] << 24) | ((uint64_t)p[2] << 16) |
            ((uint64_t)p[1] << 8) | (uint64_t)p[0]);
  }
}

// Read an offset from BUF and advance the appropriate number of
// bytes.
uint64_t read_offset(ten_backtrace_t *self, dwarf_buf *buf, int is_dwarf64) {
  if (is_dwarf64) {
    return read_uint64(self, buf);
  } else {
    return read_uint32(self, buf);
  }
}

// Read an address from BUF and advance the appropriate number of
// bytes.
uint64_t read_address(ten_backtrace_t *self, dwarf_buf *buf, int addrsize) {
  switch (addrsize) {
  case 1:
    return read_byte(self, buf);
  case 2:
    return read_uint16(self, buf);
  case 4:
    return read_uint32(self, buf);
  case 8:
    return read_uint64(self, buf);
  default:
    dwarf_buf_error(self, buf, "unrecognized address size", 0);
    return 0;
  }
}

// Read an unsigned LEB128 number.
uint64_t read_uleb128(ten_backtrace_t *self, dwarf_buf *buf) {
  unsigned char b = 0;

  uint64_t ret = 0;
  unsigned int shift = 0;
  int overflow = 0;
  do {
    const unsigned char *p = buf->buf;
    if (!advance(self, buf, 1)) {
      return 0;
    }
    b = *p;
    if (shift < 64) {
      ret |= ((uint64_t)(b & 0x7f)) << shift;
    } else if (!overflow) {
      dwarf_buf_error(self, buf, "LEB128 overflows uint64_t", 0);
      overflow = 1;
    }
    shift += 7;
  } while ((b & 0x80) != 0);

  return ret;
}

// Read a signed LEB128 number.
int64_t read_sleb128(ten_backtrace_t *self, dwarf_buf *buf) {
  unsigned char b = 0;

  uint64_t val = 0;
  unsigned int shift = 0;
  int overflow = 0;
  do {
    const unsigned char *p = buf->buf;
    if (!advance(self, buf, 1)) {
      return 0;
    }

    b = *p;
    if (shift < 64) {
      val |= ((uint64_t)(b & 0x7f)) << shift;
    } else if (!overflow) {
      dwarf_buf_error(self, buf, "signed LEB128 overflows uint64_t", 0);
      overflow = 1;
    }
    shift += 7;
  } while ((b & 0x80) != 0);

  if ((b & 0x40) != 0 && shift < 64) {
    val |= ((uint64_t)-1) << shift;
  }

  return (int64_t)val;
}

// Read initial_length from BUF and advance the appropriate number of bytes.
uint64_t read_initial_length(ten_backtrace_t *self, dwarf_buf *buf,
                             int *is_dwarf64) {
  uint64_t len = read_uint32(self, buf);
  if (len == 0xffffffff) {
    len = read_uint64(self, buf);
    *is_dwarf64 = 1;
  } else {
    *is_dwarf64 = 0;
  }

  return len;
}
