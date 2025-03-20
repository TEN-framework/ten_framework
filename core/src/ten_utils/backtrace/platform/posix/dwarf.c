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
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/types.h>

#include "include_internal/ten_utils/backtrace/backtrace.h"
#include "include_internal/ten_utils/backtrace/platform/posix/dwarf.h"
#include "include_internal/ten_utils/backtrace/platform/posix/internal.h"
#include "include_internal/ten_utils/backtrace/sort.h"
#include "include_internal/ten_utils/backtrace/vector.h"
#include "ten_utils/lib/atomic.h"
#include "ten_utils/lib/atomic_ptr.h"

// Report an error for a DWARF buffer.
static void dwarf_buf_error(ten_backtrace_t *self, struct dwarf_buf *buf,
                            const char *msg, int errnum) {
  char b[200];

  int written = snprintf(b, sizeof b, "%s in %s at %d", msg, buf->name,
                         (int)(buf->buf - buf->start));
  assert(written > 0);

  buf->error_cb(self, b, errnum, NULL);
}

/* Require at least COUNT bytes in BUF.  Return 1 if all is well, 0 on
   error.  */
static int require(ten_backtrace_t *self, struct dwarf_buf *buf, size_t count) {
  if (buf->left >= count) {
    return 1;
  }

  if (!buf->reported_underflow) {
    dwarf_buf_error(self, buf, "DWARF underflow", 0);
    buf->reported_underflow = 1;
  }

  return 0;
}

/* Advance COUNT bytes in BUF.  Return 1 if all is well, 0 on
   error.  */
static int advance(ten_backtrace_t *self, struct dwarf_buf *buf, size_t count) {
  if (!require(self, buf, count)) {
    return 0;
  }
  buf->buf += count;
  buf->left -= count;
  return 1;
}

/* Read one zero-terminated string from BUF and advance past the string.  */
static const char *read_string(ten_backtrace_t *self, struct dwarf_buf *buf) {
  const char *p = (const char *)buf->buf;
  size_t len = strnlen(p, buf->left);

  /* - If len == left, we ran out of buffer before finding the zero terminator.
       Generate an error by advancing len + 1.
     - If len < left, advance by len + 1 to skip past the zero terminator.  */
  size_t count = len + 1;

  if (!advance(self, buf, count)) {
    return NULL;
  }

  return p;
}

/* Read one byte from BUF and advance 1 byte.  */
static unsigned char read_byte(ten_backtrace_t *self, struct dwarf_buf *buf) {
  const unsigned char *p = buf->buf;

  if (!advance(self, buf, 1)) {
    return 0;
  }
  return p[0];
}

/* Read a signed char from BUF and advance 1 byte.  */
static signed char read_sbyte(ten_backtrace_t *self, struct dwarf_buf *buf) {
  const unsigned char *p = buf->buf;

  if (!advance(self, buf, 1)) {
    return 0;
  }
  return (*p ^ 0x80) - 0x80;
}

/* Read a uint16 from BUF and advance 2 bytes.  */
static uint16_t read_uint16(ten_backtrace_t *self, struct dwarf_buf *buf) {
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

/* Read a 24 bit value from BUF and advance 3 bytes.  */
static uint32_t read_uint24(ten_backtrace_t *self, struct dwarf_buf *buf) {
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

/* Read a uint32 from BUF and advance 4 bytes.  */

static uint32_t read_uint32(ten_backtrace_t *self, struct dwarf_buf *buf) {
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

/* Read a uint64 from BUF and advance 8 bytes.  */
static uint64_t read_uint64(ten_backtrace_t *self, struct dwarf_buf *buf) {
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

/* Read an offset from BUF and advance the appropriate number of
   bytes.  */
static uint64_t read_offset(ten_backtrace_t *self, struct dwarf_buf *buf,
                            int is_dwarf64) {
  if (is_dwarf64) {
    return read_uint64(self, buf);
  } else {
    return read_uint32(self, buf);
  }
}

/* Read an address from BUF and advance the appropriate number of
   bytes.  */
static uint64_t read_address(ten_backtrace_t *self, struct dwarf_buf *buf,
                             int addrsize) {
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

/* Return whether a value is the highest possible address, given the
   address size.  */
static int is_highest_address(uint64_t address, int addrsize) {
  switch (addrsize) {
  case 1:
    return address == (unsigned char)-1;
  case 2:
    return address == (uint16_t)-1;
  case 4:
    return address == (uint32_t)-1;
  case 8:
    return address == (uint64_t)-1;
  default:
    return 0;
  }
}

/* Read an unsigned LEB128 number.  */
static uint64_t read_uleb128(ten_backtrace_t *self, struct dwarf_buf *buf) {
  uint64_t ret;
  unsigned int shift;
  int overflow;
  unsigned char b;

  ret = 0;
  shift = 0;
  overflow = 0;
  do {
    const unsigned char *p;

    p = buf->buf;
    if (!advance(self, buf, 1)) {
      return 0;
    }
    b = *p;
    if (shift < 64)
      ret |= ((uint64_t)(b & 0x7f)) << shift;
    else if (!overflow) {
      dwarf_buf_error(self, buf, "LEB128 overflows uint64_t", 0);
      overflow = 1;
    }
    shift += 7;
  } while ((b & 0x80) != 0);

  return ret;
}

/* Read a signed LEB128 number.  */
static int64_t read_sleb128(ten_backtrace_t *self, struct dwarf_buf *buf) {
  uint64_t val;
  unsigned int shift;
  int overflow;
  unsigned char b;

  val = 0;
  shift = 0;
  overflow = 0;
  do {
    const unsigned char *p;

    p = buf->buf;
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

/* Return the length of an LEB128 number.  */
static size_t leb128_len(const unsigned char *p) {
  size_t ret = 1;
  while ((*p & 0x80) != 0) {
    ++p;
    ++ret;
  }
  return ret;
}

/* Read initial_length from BUF and advance the appropriate number of bytes.  */

static uint64_t read_initial_length(ten_backtrace_t *self,
                                    struct dwarf_buf *buf, int *is_dwarf64) {
  uint64_t len;

  len = read_uint32(self, buf);
  if (len == 0xffffffff) {
    len = read_uint64(self, buf);
    *is_dwarf64 = 1;
  } else
    *is_dwarf64 = 0;

  return len;
}

// Free an abbreviations structure.
static void free_abbrevs(ten_backtrace_t *self, struct abbrevs *abbrevs,
                         ten_backtrace_error_func_t error_cb, void *data) {
  size_t i;

  for (i = 0; i < abbrevs->num_abbrevs; ++i) {
    free(abbrevs->abbrevs[i].attrs);
  }
  free(abbrevs->abbrevs);
  abbrevs->num_abbrevs = 0;
  abbrevs->abbrevs = NULL;
}

/* Read an attribute value.  Returns 1 on success, 0 on failure.  If
   the value can be represented as a uint64_t, sets *VAL and sets
   *IS_VALID to 1.  We don't try to store the value of other attribute
   forms, because we don't care about them.  */

static int read_attribute(ten_backtrace_t *self, enum dwarf_form form,
                          uint64_t implicit_val, struct dwarf_buf *buf,
                          int is_dwarf64, int version, int addrsize,
                          const struct dwarf_sections *dwarf_sections,
                          struct dwarf_data *altlink, struct attr_val *val) {
  /* Avoid warnings about val.u.FIELD may be used uninitialized if
     this function is inlined.  The warnings aren't valid but can
     occur because the different fields are set and used
     conditionally.  */
  memset(val, 0, sizeof *val);

  switch (form) {
  case DW_FORM_addr:
    val->encoding = ATTR_VAL_ADDRESS;
    val->u.uint = read_address(self, buf, addrsize);
    return 1;
  case DW_FORM_block2:
    val->encoding = ATTR_VAL_BLOCK;
    return advance(self, buf, read_uint16(self, buf));
  case DW_FORM_block4:
    val->encoding = ATTR_VAL_BLOCK;
    return advance(self, buf, read_uint32(self, buf));
  case DW_FORM_data2:
    val->encoding = ATTR_VAL_UINT;
    val->u.uint = read_uint16(self, buf);
    return 1;
  case DW_FORM_data4:
    val->encoding = ATTR_VAL_UINT;
    val->u.uint = read_uint32(self, buf);
    return 1;
  case DW_FORM_data8:
    val->encoding = ATTR_VAL_UINT;
    val->u.uint = read_uint64(self, buf);
    return 1;
  case DW_FORM_data16:
    val->encoding = ATTR_VAL_BLOCK;
    return advance(self, buf, 16);
  case DW_FORM_string:
    val->encoding = ATTR_VAL_STRING;
    val->u.string = read_string(self, buf);
    return val->u.string == NULL ? 0 : 1;
  case DW_FORM_block:
    val->encoding = ATTR_VAL_BLOCK;
    return advance(self, buf, read_uleb128(self, buf));
  case DW_FORM_block1:
    val->encoding = ATTR_VAL_BLOCK;
    return advance(self, buf, read_byte(self, buf));
  case DW_FORM_data1:
    val->encoding = ATTR_VAL_UINT;
    val->u.uint = read_byte(self, buf);
    return 1;
  case DW_FORM_flag:
    val->encoding = ATTR_VAL_UINT;
    val->u.uint = read_byte(self, buf);
    return 1;
  case DW_FORM_sdata:
    val->encoding = ATTR_VAL_SINT;
    val->u.sint = read_sleb128(self, buf);
    return 1;
  case DW_FORM_strp: {
    uint64_t offset;

    offset = read_offset(self, buf, is_dwarf64);
    if (offset >= dwarf_sections->size[DEBUG_STR]) {
      dwarf_buf_error(self, buf, "DW_FORM_strp out of range", 0);
      return 0;
    }
    val->encoding = ATTR_VAL_STRING;
    val->u.string = (const char *)dwarf_sections->data[DEBUG_STR] + offset;
    return 1;
  }
  case DW_FORM_line_strp: {
    uint64_t offset;

    offset = read_offset(self, buf, is_dwarf64);
    if (offset >= dwarf_sections->size[DEBUG_LINE_STR]) {
      dwarf_buf_error(self, buf, "DW_FORM_line_strp out of range", 0);
      return 0;
    }
    val->encoding = ATTR_VAL_STRING;
    val->u.string = (const char *)dwarf_sections->data[DEBUG_LINE_STR] + offset;
    return 1;
  }
  case DW_FORM_udata:
    val->encoding = ATTR_VAL_UINT;
    val->u.uint = read_uleb128(self, buf);
    return 1;
  case DW_FORM_ref_addr:
    val->encoding = ATTR_VAL_REF_INFO;
    if (version == 2)
      val->u.uint = read_address(self, buf, addrsize);
    else
      val->u.uint = read_offset(self, buf, is_dwarf64);
    return 1;
  case DW_FORM_ref1:
    val->encoding = ATTR_VAL_REF_UNIT;
    val->u.uint = read_byte(self, buf);
    return 1;
  case DW_FORM_ref2:
    val->encoding = ATTR_VAL_REF_UNIT;
    val->u.uint = read_uint16(self, buf);
    return 1;
  case DW_FORM_ref4:
    val->encoding = ATTR_VAL_REF_UNIT;
    val->u.uint = read_uint32(self, buf);
    return 1;
  case DW_FORM_ref8:
    val->encoding = ATTR_VAL_REF_UNIT;
    val->u.uint = read_uint64(self, buf);
    return 1;
  case DW_FORM_ref_udata:
    val->encoding = ATTR_VAL_REF_UNIT;
    val->u.uint = read_uleb128(self, buf);
    return 1;
  case DW_FORM_indirect: {
    uint64_t form;

    form = read_uleb128(self, buf);
    if (form == DW_FORM_implicit_const) {
      dwarf_buf_error(self, buf, "DW_FORM_indirect to DW_FORM_implicit_const",
                      0);
      return 0;
    }
    return read_attribute(self, (enum dwarf_form)form, 0, buf, is_dwarf64,
                          version, addrsize, dwarf_sections, altlink, val);
  }
  case DW_FORM_sec_offset:
    val->encoding = ATTR_VAL_REF_SECTION;
    val->u.uint = read_offset(self, buf, is_dwarf64);
    return 1;
  case DW_FORM_exprloc:
    val->encoding = ATTR_VAL_EXPR;
    return advance(self, buf, read_uleb128(self, buf));
  case DW_FORM_flag_present:
    val->encoding = ATTR_VAL_UINT;
    val->u.uint = 1;
    return 1;
  case DW_FORM_ref_sig8:
    val->encoding = ATTR_VAL_REF_TYPE;
    val->u.uint = read_uint64(self, buf);
    return 1;
  case DW_FORM_strx:
  case DW_FORM_strx1:
  case DW_FORM_strx2:
  case DW_FORM_strx3:
  case DW_FORM_strx4: {
    uint64_t offset;

    switch (form) {
    case DW_FORM_strx:
      offset = read_uleb128(self, buf);
      break;
    case DW_FORM_strx1:
      offset = read_byte(self, buf);
      break;
    case DW_FORM_strx2:
      offset = read_uint16(self, buf);
      break;
    case DW_FORM_strx3:
      offset = read_uint24(self, buf);
      break;
    case DW_FORM_strx4:
      offset = read_uint32(self, buf);
      break;
    default:
      /* This case can't happen.  */
      return 0;
    }
    val->encoding = ATTR_VAL_STRING_INDEX;
    val->u.uint = offset;
    return 1;
  }
  case DW_FORM_addrx:
  case DW_FORM_addrx1:
  case DW_FORM_addrx2:
  case DW_FORM_addrx3:
  case DW_FORM_addrx4: {
    uint64_t offset;

    switch (form) {
    case DW_FORM_addrx:
      offset = read_uleb128(self, buf);
      break;
    case DW_FORM_addrx1:
      offset = read_byte(self, buf);
      break;
    case DW_FORM_addrx2:
      offset = read_uint16(self, buf);
      break;
    case DW_FORM_addrx3:
      offset = read_uint24(self, buf);
      break;
    case DW_FORM_addrx4:
      offset = read_uint32(self, buf);
      break;
    default:
      /* This case can't happen.  */
      return 0;
    }
    val->encoding = ATTR_VAL_ADDRESS_INDEX;
    val->u.uint = offset;
    return 1;
  }
  case DW_FORM_ref_sup4:
    val->encoding = ATTR_VAL_REF_SECTION;
    val->u.uint = read_uint32(self, buf);
    return 1;
  case DW_FORM_ref_sup8:
    val->encoding = ATTR_VAL_REF_SECTION;
    val->u.uint = read_uint64(self, buf);
    return 1;
  case DW_FORM_implicit_const:
    val->encoding = ATTR_VAL_UINT;
    val->u.uint = implicit_val;
    return 1;
  case DW_FORM_loclistx:
    /* We don't distinguish this from DW_FORM_sec_offset.  It
     * shouldn't matter since we don't care about loclists.  */
    val->encoding = ATTR_VAL_REF_SECTION;
    val->u.uint = read_uleb128(self, buf);
    return 1;
  case DW_FORM_rnglistx:
    val->encoding = ATTR_VAL_RNGLISTS_INDEX;
    val->u.uint = read_uleb128(self, buf);
    return 1;
  case DW_FORM_GNU_addr_index:
    val->encoding = ATTR_VAL_REF_SECTION;
    val->u.uint = read_uleb128(self, buf);
    return 1;
  case DW_FORM_GNU_str_index:
    val->encoding = ATTR_VAL_REF_SECTION;
    val->u.uint = read_uleb128(self, buf);
    return 1;
  case DW_FORM_GNU_ref_alt:
    val->u.uint = read_offset(self, buf, is_dwarf64);
    if (altlink == NULL) {
      val->encoding = ATTR_VAL_NONE;
      return 1;
    }
    val->encoding = ATTR_VAL_REF_ALT_INFO;
    return 1;
  case DW_FORM_strp_sup:
  case DW_FORM_GNU_strp_alt: {
    uint64_t offset;

    offset = read_offset(self, buf, is_dwarf64);
    if (altlink == NULL) {
      val->encoding = ATTR_VAL_NONE;
      return 1;
    }
    if (offset >= altlink->dwarf_sections.size[DEBUG_STR]) {
      dwarf_buf_error(self, buf, "DW_FORM_strp_sup out of range", 0);
      return 0;
    }
    val->encoding = ATTR_VAL_STRING;
    val->u.string =
        (const char *)altlink->dwarf_sections.data[DEBUG_STR] + offset;
    return 1;
  }
  default:
    dwarf_buf_error(self, buf, "unrecognized DWARF form", -1);
    return 0;
  }
}

/* If we can determine the value of a string attribute, set *STRING to
   point to the string.  Return 1 on success, 0 on error.  If we don't
   know the value, we consider that a success, and we don't change
   *STRING.  An error is only reported for some sort of out of range
   offset.  */

static int resolve_string(ten_backtrace_t *self,
                          const struct dwarf_sections *dwarf_sections,
                          int is_dwarf64, int is_bigendian,
                          uint64_t str_offsets_base, const struct attr_val *val,
                          ten_backtrace_error_func_t error_cb, void *data,
                          const char **string) {
  switch (val->encoding) {
  case ATTR_VAL_STRING:
    *string = val->u.string;
    return 1;

  case ATTR_VAL_STRING_INDEX: {
    uint64_t offset;
    struct dwarf_buf offset_buf;

    offset = val->u.uint * (is_dwarf64 ? 8 : 4) + str_offsets_base;
    if (offset + (is_dwarf64 ? 8 : 4) >
        dwarf_sections->size[DEBUG_STR_OFFSETS]) {
      error_cb(self, "DW_FORM_strx value out of range", 0, data);
      return 0;
    }

    offset_buf.name = ".debug_str_offsets";
    offset_buf.start = dwarf_sections->data[DEBUG_STR_OFFSETS];
    offset_buf.buf = dwarf_sections->data[DEBUG_STR_OFFSETS] + offset;
    offset_buf.left = dwarf_sections->size[DEBUG_STR_OFFSETS] - offset;
    offset_buf.is_bigendian = is_bigendian;
    offset_buf.error_cb = error_cb;
    offset_buf.data = data;
    offset_buf.reported_underflow = 0;

    offset = read_offset(self, &offset_buf, is_dwarf64);
    if (offset >= dwarf_sections->size[DEBUG_STR]) {
      dwarf_buf_error(self, &offset_buf, "DW_FORM_strx offset out of range", 0);
      return 0;
    }
    *string = (const char *)dwarf_sections->data[DEBUG_STR] + offset;
    return 1;
  }

  default:
    return 1;
  }
}

/* Set *ADDRESS to the real address for a ATTR_VAL_ADDRESS_INDEX.
   Return 1 on success, 0 on error.  */

static int resolve_addr_index(ten_backtrace_t *self,
                              const struct dwarf_sections *dwarf_sections,
                              uint64_t addr_base, int addrsize,
                              int is_bigendian, uint64_t addr_index,
                              ten_backtrace_error_func_t error_cb, void *data,
                              uintptr_t *address) {
  uint64_t offset;
  struct dwarf_buf addr_buf;

  offset = addr_index * addrsize + addr_base;
  if (offset + addrsize > dwarf_sections->size[DEBUG_ADDR]) {
    error_cb(self, "DW_FORM_addrx value out of range", 0, data);
    return 0;
  }

  addr_buf.name = ".debug_addr";
  addr_buf.start = dwarf_sections->data[DEBUG_ADDR];
  addr_buf.buf = dwarf_sections->data[DEBUG_ADDR] + offset;
  addr_buf.left = dwarf_sections->size[DEBUG_ADDR] - offset;
  addr_buf.is_bigendian = is_bigendian;
  addr_buf.error_cb = error_cb;
  addr_buf.data = data;
  addr_buf.reported_underflow = 0;

  *address = (uintptr_t)read_address(self, &addr_buf, addrsize);
  return 1;
}

/* Compare a unit offset against a unit for bsearch.  */

static int units_search(const void *vkey, const void *ventry) {
  const size_t *key = (const size_t *)vkey;
  const struct unit *entry = *((const struct unit *const *)ventry);
  size_t offset;

  offset = *key;
  if (offset < entry->low_offset) {
    return -1;
  } else if (offset >= entry->high_offset) {
    return 1;
  } else {
    return 0;
  }
}

/* Find a unit in PU containing OFFSET.  */

static struct unit *find_unit(struct unit **pu, size_t units_count,
                              size_t offset) {
  struct unit **u;
  u = bsearch(&offset, pu, units_count, sizeof(struct unit *), units_search);
  return u == NULL ? NULL : *u;
}

/* Compare function_addrs for qsort.  When ranges are nested, make the
   smallest one sort last.  */

static int function_addrs_compare(const void *v1, const void *v2) {
  const struct function_addrs *a1 = (const struct function_addrs *)v1;
  const struct function_addrs *a2 = (const struct function_addrs *)v2;

  if (a1->low < a2->low) {
    return -1;
  }
  if (a1->low > a2->low) {
    return 1;
  }
  if (a1->high < a2->high) {
    return 1;
  }
  if (a1->high > a2->high) {
    return -1;
  }

  return strcmp(a1->function->name, a2->function->name);
}

/* Compare a PC against a function_addrs for bsearch.  We always
   allocate an entra entry at the end of the vector, so that this
   routine can safely look at the next entry.  Note that if there are
   multiple ranges containing PC, which one will be returned is
   unpredictable.  We compensate for that in dwarf_fileline.  */

static int function_addrs_search(const void *vkey, const void *ventry) {
  const uintptr_t *key = (const uintptr_t *)vkey;
  const struct function_addrs *entry = (const struct function_addrs *)ventry;
  uintptr_t pc;

  pc = *key;
  if (pc < entry->low) {
    return -1;
  } else if (pc > (entry + 1)->low) {
    return 1;
  } else {
    return 0;
  }
}

/* Add a new compilation unit address range to a vector.  This is
   called via add_ranges.  Returns 1 on success, 0 on failure.  */

static int add_unit_addr(ten_backtrace_t *self, void *rdata, uintptr_t lowpc,
                         uintptr_t highpc, ten_backtrace_error_func_t error_cb,
                         void *data, void *pvec) {
  struct unit *u = (struct unit *)rdata;
  struct unit_addrs_vector *vec = (struct unit_addrs_vector *)pvec;
  struct unit_addrs *p;

  /* Try to merge with the last entry.  */
  if (vec->count > 0) {
    p = (struct unit_addrs *)vec->vec.data + (vec->count - 1);
    if ((lowpc == p->high || lowpc == p->high + 1) && u == p->u) {
      if (highpc > p->high) {
        p->high = highpc;
      }
      return 1;
    }
  }

  p = (unit_addrs *)ten_vector_grow(&vec->vec, sizeof(unit_addrs));
  if (p == NULL) {
    return 0;
  }

  p->low = lowpc;
  p->high = highpc;
  p->u = u;

  ++vec->count;

  return 1;
}

/* Compare unit_addrs for qsort.  When ranges are nested, make the
   smallest one sort last.  */

static int unit_addrs_compare(const void *v1, const void *v2) {
  const struct unit_addrs *a1 = (const struct unit_addrs *)v1;
  const struct unit_addrs *a2 = (const struct unit_addrs *)v2;

  if (a1->low < a2->low) {
    return -1;
  }
  if (a1->low > a2->low) {
    return 1;
  }
  if (a1->high < a2->high) {
    return 1;
  }
  if (a1->high > a2->high) {
    return -1;
  }
  if (a1->u->lineoff < a2->u->lineoff) {
    return -1;
  }
  if (a1->u->lineoff > a2->u->lineoff) {
    return 1;
  }
  return 0;
}

/* Compare a PC against a unit_addrs for bsearch.  We always allocate
   an entry entry at the end of the vector, so that this routine can
   safely look at the next entry.  Note that if there are multiple
   ranges containing PC, which one will be returned is unpredictable.
   We compensate for that in dwarf_fileline.  */

static int unit_addrs_search(const void *vkey, const void *ventry) {
  const uintptr_t *key = (const uintptr_t *)vkey;
  const struct unit_addrs *entry = (const struct unit_addrs *)ventry;
  uintptr_t pc;

  pc = *key;
  if (pc < entry->low)
    return -1;
  else if (pc > (entry + 1)->low)
    return 1;
  else
    return 0;
}

/* Sort the line vector by PC.  We want a stable sort here to maintain
   the order of lines for the same PC values.  Since the sequence is
   being sorted in place, their addresses cannot be relied on to
   maintain stability.  That is the purpose of the index member.  */

static int line_compare(const void *v1, const void *v2) {
  const struct line *ln1 = (const struct line *)v1;
  const struct line *ln2 = (const struct line *)v2;

  if (ln1->pc < ln2->pc)
    return -1;
  else if (ln1->pc > ln2->pc)
    return 1;
  else if (ln1->idx < ln2->idx)
    return -1;
  else if (ln1->idx > ln2->idx)
    return 1;
  else
    return 0;
}

/* Find a PC in a line vector.  We always allocate an extra entry at
   the end of the lines vector, so that this routine can safely look
   at the next entry.  Note that when there are multiple mappings for
   the same PC value, this will return the last one.  */

static int line_search(const void *vkey, const void *ventry) {
  const uintptr_t *key = (const uintptr_t *)vkey;
  const struct line *entry = (const struct line *)ventry;
  uintptr_t pc;

  pc = *key;
  if (pc < entry->pc)
    return -1;
  else if (pc >= (entry + 1)->pc)
    return 1;
  else
    return 0;
}

/* Sort the abbrevs by the abbrev code.  This function is passed to
   both qsort and bsearch.  */

static int abbrev_compare(const void *v1, const void *v2) {
  const struct abbrev *a1 = (const struct abbrev *)v1;
  const struct abbrev *a2 = (const struct abbrev *)v2;

  if (a1->code < a2->code)
    return -1;
  else if (a1->code > a2->code)
    return 1;
  else {
    /* This really shouldn't happen.  It means there are two
       different abbrevs with the same code, and that means we don't
       know which one lookup_abbrev should return.  */
    return 0;
  }
}

/* Read the abbreviation table for a compilation unit.  Returns 1 on
   success, 0 on failure.  */

static int read_abbrevs(ten_backtrace_t *self, uint64_t abbrev_offset,
                        const unsigned char *dwarf_abbrev,
                        size_t dwarf_abbrev_size, int is_bigendian,
                        ten_backtrace_error_func_t error_cb, void *data,
                        struct abbrevs *abbrevs) {
  struct dwarf_buf abbrev_buf;
  struct dwarf_buf count_buf;
  size_t num_abbrevs;

  abbrevs->num_abbrevs = 0;
  abbrevs->abbrevs = NULL;

  if (abbrev_offset >= dwarf_abbrev_size) {
    error_cb(self, "abbrev offset out of range", 0, data);
    return 0;
  }

  abbrev_buf.name = ".debug_abbrev";
  abbrev_buf.start = dwarf_abbrev;
  abbrev_buf.buf = dwarf_abbrev + abbrev_offset;
  abbrev_buf.left = dwarf_abbrev_size - abbrev_offset;
  abbrev_buf.is_bigendian = is_bigendian;
  abbrev_buf.error_cb = error_cb;
  abbrev_buf.data = data;
  abbrev_buf.reported_underflow = 0;

  /* Count the number of abbrevs in this list.  */

  count_buf = abbrev_buf;
  num_abbrevs = 0;
  while (read_uleb128(self, &count_buf) != 0) {
    if (count_buf.reported_underflow)
      return 0;
    ++num_abbrevs;
    // Skip tag.
    read_uleb128(self, &count_buf);
    // Skip has_children.
    read_byte(self, &count_buf);
    // Skip attributes.
    while (read_uleb128(self, &count_buf) != 0) {
      uint64_t form;

      form = read_uleb128(self, &count_buf);
      if ((enum dwarf_form)form == DW_FORM_implicit_const)
        read_sleb128(self, &count_buf);
    }
    // Skip form of last attribute.
    read_uleb128(self, &count_buf);
  }

  if (count_buf.reported_underflow)
    return 0;

  if (num_abbrevs == 0)
    return 1;

  abbrevs->abbrevs =
      (struct abbrev *)malloc(num_abbrevs * sizeof(struct abbrev));
  if (abbrevs->abbrevs == NULL) {
    return 0;
  }

  abbrevs->num_abbrevs = num_abbrevs;
  memset(abbrevs->abbrevs, 0, num_abbrevs * sizeof(struct abbrev));

  num_abbrevs = 0;
  while (1) {
    uint64_t code;
    struct abbrev a;
    size_t num_attrs;
    struct attr *attrs;

    if (abbrev_buf.reported_underflow)
      goto fail;

    code = read_uleb128(self, &abbrev_buf);
    if (code == 0)
      break;

    a.code = code;
    a.tag = (enum dwarf_tag)read_uleb128(self, &abbrev_buf);
    a.has_children = read_byte(self, &abbrev_buf);

    count_buf = abbrev_buf;
    num_attrs = 0;
    while (read_uleb128(self, &count_buf) != 0) {
      uint64_t form;

      ++num_attrs;
      form = read_uleb128(self, &count_buf);
      if ((enum dwarf_form)form == DW_FORM_implicit_const)
        read_sleb128(self, &count_buf);
    }

    if (num_attrs == 0) {
      attrs = NULL;
      read_uleb128(self, &abbrev_buf);
      read_uleb128(self, &abbrev_buf);
    } else {
      attrs = (struct attr *)malloc(num_attrs * sizeof *attrs);
      if (attrs == NULL) {
        goto fail;
      }

      num_attrs = 0;
      while (1) {
        uint64_t name;
        uint64_t form;

        name = read_uleb128(self, &abbrev_buf);
        form = read_uleb128(self, &abbrev_buf);
        if (name == 0) {
          break;
        }

        attrs[num_attrs].name = (enum dwarf_attribute)name;
        attrs[num_attrs].form = (enum dwarf_form)form;
        if ((enum dwarf_form)form == DW_FORM_implicit_const) {
          attrs[num_attrs].val = read_sleb128(self, &abbrev_buf);
        } else {
          attrs[num_attrs].val = 0;
        }

        ++num_attrs;
      }
    }

    a.num_attrs = num_attrs;
    a.attrs = attrs;

    abbrevs->abbrevs[num_abbrevs] = a;
    ++num_abbrevs;
  }

  backtrace_sort(abbrevs->abbrevs, abbrevs->num_abbrevs, sizeof(struct abbrev),
                 abbrev_compare);

  return 1;

fail:
  free_abbrevs(self, abbrevs, error_cb, data);
  return 0;
}

// Return the abbrev information for an abbrev code.
static const struct abbrev *
lookup_abbrev(ten_backtrace_t *self, struct abbrevs *abbrevs, uint64_t code,
              ten_backtrace_error_func_t error_cb, void *data) {
  struct abbrev key;
  void *p;

  /* With GCC, where abbrevs are simply numbered in order, we should
     be able to just look up the entry.  */
  if (code - 1 < abbrevs->num_abbrevs &&
      abbrevs->abbrevs[code - 1].code == code) {
    return &abbrevs->abbrevs[code - 1];
  }

  /* Otherwise we have to search.  */
  memset(&key, 0, sizeof key);
  key.code = code;
  p = bsearch(&key, abbrevs->abbrevs, abbrevs->num_abbrevs,
              sizeof(struct abbrev), abbrev_compare);
  if (p == NULL) {
    error_cb(self, "Invalid abbreviation code", 0, data);
    return NULL;
  }
  return (const struct abbrev *)p;
}

/* This struct is used to gather address range information while
   reading attributes.  We use this while building a mapping from
   address ranges to compilation units and then again while mapping
   from address ranges to function entries.  Normally either
   lowpc/highpc is set or ranges is set.  */

struct pcrange {
  uintptr_t lowpc;          /* The low PC value.  */
  int have_lowpc;           /* Whether a low PC value was found.  */
  int lowpc_is_addr_index;  /* Whether lowpc is in .debug_addr.  */
  uintptr_t highpc;         /* The high PC value.  */
  int have_highpc;          /* Whether a high PC value was found.  */
  int highpc_is_relative;   /* Whether highpc is relative to lowpc.  */
  int highpc_is_addr_index; /* Whether highpc is in .debug_addr.  */
  uint64_t ranges;          /* Offset in ranges section.  */
  int have_ranges;          /* Whether ranges is valid.  */
  int ranges_is_index;      /* Whether ranges is DW_FORM_rnglistx.  */
};

/* Update PCRANGE from an attribute value.  */

static void update_pcrange(const struct attr *attr, const struct attr_val *val,
                           struct pcrange *pcrange) {
  switch (attr->name) {
  case DW_AT_low_pc:
    if (val->encoding == ATTR_VAL_ADDRESS) {
      pcrange->lowpc = (uintptr_t)val->u.uint;
      pcrange->have_lowpc = 1;
    } else if (val->encoding == ATTR_VAL_ADDRESS_INDEX) {
      pcrange->lowpc = (uintptr_t)val->u.uint;
      pcrange->have_lowpc = 1;
      pcrange->lowpc_is_addr_index = 1;
    }
    break;

  case DW_AT_high_pc:
    if (val->encoding == ATTR_VAL_ADDRESS) {
      pcrange->highpc = (uintptr_t)val->u.uint;
      pcrange->have_highpc = 1;
    } else if (val->encoding == ATTR_VAL_UINT) {
      pcrange->highpc = (uintptr_t)val->u.uint;
      pcrange->have_highpc = 1;
      pcrange->highpc_is_relative = 1;
    } else if (val->encoding == ATTR_VAL_ADDRESS_INDEX) {
      pcrange->highpc = (uintptr_t)val->u.uint;
      pcrange->have_highpc = 1;
      pcrange->highpc_is_addr_index = 1;
    }
    break;

  case DW_AT_ranges:
    if (val->encoding == ATTR_VAL_UINT ||
        val->encoding == ATTR_VAL_REF_SECTION) {
      pcrange->ranges = val->u.uint;
      pcrange->have_ranges = 1;
    } else if (val->encoding == ATTR_VAL_RNGLISTS_INDEX) {
      pcrange->ranges = val->u.uint;
      pcrange->have_ranges = 1;
      pcrange->ranges_is_index = 1;
    }
    break;

  default:
    break;
  }
}

/* Call ADD_RANGE for a low/high PC pair.  Returns 1 on success, 0 on
  error.  */

static int add_low_high_range(
    ten_backtrace_t *self, const struct dwarf_sections *dwarf_sections,
    uintptr_t base_address, int is_bigendian, struct unit *u,
    const struct pcrange *pcrange,
    int (*add_range)(ten_backtrace_t *self, void *rdata, uintptr_t lowpc,
                     uintptr_t highpc, ten_backtrace_error_func_t error_cb,
                     void *data, void *vec),
    void *rdata, ten_backtrace_error_func_t error_cb, void *data, void *vec) {
  uintptr_t lowpc;
  uintptr_t highpc;

  lowpc = pcrange->lowpc;
  if (pcrange->lowpc_is_addr_index) {
    if (!resolve_addr_index(self, dwarf_sections, u->addr_base, u->addrsize,
                            is_bigendian, lowpc, error_cb, data, &lowpc)) {
      return 0;
    }
  }

  highpc = pcrange->highpc;
  if (pcrange->highpc_is_addr_index) {
    if (!resolve_addr_index(self, dwarf_sections, u->addr_base, u->addrsize,
                            is_bigendian, highpc, error_cb, data, &highpc)) {
      return 0;
    }
  }

  if (pcrange->highpc_is_relative) {
    highpc += lowpc;
  }

  /* Add in the base address of the module when recording PC values,
     so that we can look up the PC directly.  */
  lowpc += base_address;
  highpc += base_address;

  return add_range(self, rdata, lowpc, highpc, error_cb, data, vec);
}

/* Call ADD_RANGE for each range read from .debug_ranges, as used in
   DWARF versions 2 through 4.  */

static int add_ranges_from_ranges(
    ten_backtrace_t *self, const struct dwarf_sections *dwarf_sections,
    uintptr_t base_address, int is_bigendian, struct unit *u, uintptr_t base,
    const struct pcrange *pcrange,
    int (*add_range)(ten_backtrace_t *self, void *rdata, uintptr_t lowpc,
                     uintptr_t highpc, ten_backtrace_error_func_t error_cb,
                     void *data, void *vec),
    void *rdata, ten_backtrace_error_func_t error_cb, void *data, void *vec) {
  struct dwarf_buf ranges_buf;

  if (pcrange->ranges >= dwarf_sections->size[DEBUG_RANGES]) {
    error_cb(self, "ranges offset out of range", 0, data);
    return 0;
  }

  ranges_buf.name = ".debug_ranges";
  ranges_buf.start = dwarf_sections->data[DEBUG_RANGES];
  ranges_buf.buf = dwarf_sections->data[DEBUG_RANGES] + pcrange->ranges;
  ranges_buf.left = dwarf_sections->size[DEBUG_RANGES] - pcrange->ranges;
  ranges_buf.is_bigendian = is_bigendian;
  ranges_buf.error_cb = error_cb;
  ranges_buf.data = data;
  ranges_buf.reported_underflow = 0;

  while (1) {
    uint64_t low;
    uint64_t high;

    if (ranges_buf.reported_underflow) {
      return 0;
    }
    low = read_address(self, &ranges_buf, u->addrsize);
    high = read_address(self, &ranges_buf, u->addrsize);

    if (low == 0 && high == 0) {
      break;
    }

    if (is_highest_address(low, u->addrsize)) {
      base = (uintptr_t)high;
    } else {
      if (!add_range(self, rdata, (uintptr_t)low + base + base_address,
                     (uintptr_t)high + base + base_address, error_cb, data,
                     vec)) {
        return 0;
      }
    }
  }

  if (ranges_buf.reported_underflow) {
    return 0;
  }
  return 1;
}

/* Call ADD_RANGE for each range read from .debug_rnglists, as used in
   DWARF version 5.  */

static int add_ranges_from_rnglists(
    ten_backtrace_t *self, const struct dwarf_sections *dwarf_sections,
    uintptr_t base_address, int is_bigendian, struct unit *u, uintptr_t base,
    const struct pcrange *pcrange,
    int (*add_range)(ten_backtrace_t *self, void *rdata, uintptr_t lowpc,
                     uintptr_t highpc, ten_backtrace_error_func_t error_cb,
                     void *data, void *vec),
    void *rdata, ten_backtrace_error_func_t error_cb, void *data, void *vec) {
  uint64_t offset;
  struct dwarf_buf rnglists_buf;

  if (!pcrange->ranges_is_index) {
    offset = pcrange->ranges;
  } else {
    offset = u->rnglists_base + pcrange->ranges * (u->is_dwarf64 ? 8 : 4);
  }

  if (offset >= dwarf_sections->size[DEBUG_RNGLISTS]) {
    error_cb(self, "rnglists offset out of range", 0, data);
    return 0;
  }

  rnglists_buf.name = ".debug_rnglists";
  rnglists_buf.start = dwarf_sections->data[DEBUG_RNGLISTS];
  rnglists_buf.buf = dwarf_sections->data[DEBUG_RNGLISTS] + offset;
  rnglists_buf.left = dwarf_sections->size[DEBUG_RNGLISTS] - offset;
  rnglists_buf.is_bigendian = is_bigendian;
  rnglists_buf.error_cb = error_cb;
  rnglists_buf.data = data;
  rnglists_buf.reported_underflow = 0;

  if (pcrange->ranges_is_index) {
    offset = read_offset(self, &rnglists_buf, u->is_dwarf64);
    offset += u->rnglists_base;
    if (offset >= dwarf_sections->size[DEBUG_RNGLISTS]) {
      error_cb(self, "rnglists index offset out of range", 0, data);
      return 0;
    }
    rnglists_buf.buf = dwarf_sections->data[DEBUG_RNGLISTS] + offset;
    rnglists_buf.left = dwarf_sections->size[DEBUG_RNGLISTS] - offset;
  }

  while (1) {
    unsigned char rle;

    rle = read_byte(self, &rnglists_buf);
    if (rle == DW_RLE_end_of_list)
      break;
    switch (rle) {
    case DW_RLE_base_addressx: {
      uint64_t index;

      index = read_uleb128(self, &rnglists_buf);
      if (!resolve_addr_index(self, dwarf_sections, u->addr_base, u->addrsize,
                              is_bigendian, index, error_cb, data, &base))
        return 0;
    } break;

    case DW_RLE_startx_endx: {
      uint64_t index;
      uintptr_t low;
      uintptr_t high;

      index = read_uleb128(self, &rnglists_buf);
      if (!resolve_addr_index(self, dwarf_sections, u->addr_base, u->addrsize,
                              is_bigendian, index, error_cb, data, &low)) {
        return 0;
      }

      index = read_uleb128(self, &rnglists_buf);
      if (!resolve_addr_index(self, dwarf_sections, u->addr_base, u->addrsize,
                              is_bigendian, index, error_cb, data, &high)) {
        return 0;
      }

      if (!add_range(self, rdata, low + base_address, high + base_address,
                     error_cb, data, vec)) {
        return 0;
      }
    } break;

    case DW_RLE_startx_length: {
      uint64_t index;
      uintptr_t low;
      uintptr_t length;

      index = read_uleb128(self, &rnglists_buf);
      if (!resolve_addr_index(self, dwarf_sections, u->addr_base, u->addrsize,
                              is_bigendian, index, error_cb, data, &low))
        return 0;
      length = read_uleb128(self, &rnglists_buf);
      low += base_address;
      if (!add_range(self, rdata, low, low + length, error_cb, data, vec))
        return 0;
    } break;

    case DW_RLE_offset_pair: {
      uint64_t low;
      uint64_t high;

      low = read_uleb128(self, &rnglists_buf);
      high = read_uleb128(self, &rnglists_buf);
      if (!add_range(self, rdata, low + base + base_address,
                     high + base + base_address, error_cb, data, vec))
        return 0;
    } break;

    case DW_RLE_base_address:
      base = (uintptr_t)read_address(self, &rnglists_buf, u->addrsize);
      break;

    case DW_RLE_start_end: {
      uintptr_t low;
      uintptr_t high;

      low = (uintptr_t)read_address(self, &rnglists_buf, u->addrsize);
      high = (uintptr_t)read_address(self, &rnglists_buf, u->addrsize);
      if (!add_range(self, rdata, low + base_address, high + base_address,
                     error_cb, data, vec)) {
        return 0;
      }
    } break;

    case DW_RLE_start_length: {
      uintptr_t low;
      uintptr_t length;

      low = (uintptr_t)read_address(self, &rnglists_buf, u->addrsize);
      length = (uintptr_t)read_uleb128(self, &rnglists_buf);
      low += base_address;
      if (!add_range(self, rdata, low, low + length, error_cb, data, vec)) {
        return 0;
      }
    } break;

    default:
      dwarf_buf_error(self, &rnglists_buf, "unrecognized DW_RLE value", -1);
      return 0;
    }
  }

  if (rnglists_buf.reported_underflow) {
    return 0;
  }

  return 1;
}

/* Call ADD_RANGE for each lowpc/highpc pair in PCRANGE.  RDATA is
   passed to ADD_RANGE, and is either a struct unit * or a struct
   function *.  VEC is the vector we are adding ranges to, and is
   either a struct unit_addrs_vector * or a struct function_vector *.
   Returns 1 on success, 0 on error.  */

static int add_ranges(ten_backtrace_t *self,
                      const struct dwarf_sections *dwarf_sections,
                      uintptr_t base_address, int is_bigendian, struct unit *u,
                      uintptr_t base, const struct pcrange *pcrange,
                      int (*add_range)(ten_backtrace_t *self, void *rdata,
                                       uintptr_t lowpc, uintptr_t highpc,
                                       ten_backtrace_error_func_t error_cb,
                                       void *data, void *vec),
                      void *rdata, ten_backtrace_error_func_t error_cb,
                      void *data, void *vec) {
  if (pcrange->have_lowpc && pcrange->have_highpc) {
    return add_low_high_range(self, dwarf_sections, base_address, is_bigendian,
                              u, pcrange, add_range, rdata, error_cb, data,
                              vec);
  }

  if (!pcrange->have_ranges) {
    /* Did not find any address ranges to add.  */
    return 1;
  }

  if (u->version < 5) {
    return add_ranges_from_ranges(self, dwarf_sections, base_address,
                                  is_bigendian, u, base, pcrange, add_range,
                                  rdata, error_cb, data, vec);
  } else {
    return add_ranges_from_rnglists(self, dwarf_sections, base_address,
                                    is_bigendian, u, base, pcrange, add_range,
                                    rdata, error_cb, data, vec);
  }
}

/* Find the address range covered by a compilation unit, reading from
   UNIT_BUF and adding values to U.  Returns 1 if all data could be
   read, 0 if there is some error.  */

static int find_address_ranges(ten_backtrace_t *self, uintptr_t base_address,
                               struct dwarf_buf *unit_buf,
                               const struct dwarf_sections *dwarf_sections,
                               int is_bigendian, struct dwarf_data *altlink,
                               ten_backtrace_error_func_t error_cb, void *data,
                               struct unit *u, struct unit_addrs_vector *addrs,
                               enum dwarf_tag *unit_tag) {
  while (unit_buf->left > 0) {
    uint64_t code;
    const struct abbrev *abbrev;
    struct pcrange pcrange;
    struct attr_val name_val;
    int have_name_val;
    struct attr_val comp_dir_val;
    int have_comp_dir_val;
    size_t i;

    code = read_uleb128(self, unit_buf);
    if (code == 0) {
      return 1;
    }

    abbrev = lookup_abbrev(self, &u->abbrevs, code, error_cb, data);
    if (abbrev == NULL) {
      return 0;
    }

    if (unit_tag != NULL) {
      *unit_tag = abbrev->tag;
    }

    memset(&pcrange, 0, sizeof pcrange);
    memset(&name_val, 0, sizeof name_val);
    have_name_val = 0;
    memset(&comp_dir_val, 0, sizeof comp_dir_val);
    have_comp_dir_val = 0;
    for (i = 0; i < abbrev->num_attrs; ++i) {
      struct attr_val val;

      if (!read_attribute(self, abbrev->attrs[i].form, abbrev->attrs[i].val,
                          unit_buf, u->is_dwarf64, u->version, u->addrsize,
                          dwarf_sections, altlink, &val)) {
        return 0;
      }

      switch (abbrev->attrs[i].name) {
      case DW_AT_low_pc:
      case DW_AT_high_pc:
      case DW_AT_ranges:
        update_pcrange(&abbrev->attrs[i], &val, &pcrange);
        break;

      case DW_AT_stmt_list:
        if ((abbrev->tag == DW_TAG_compile_unit ||
             abbrev->tag == DW_TAG_skeleton_unit) &&
            (val.encoding == ATTR_VAL_UINT ||
             val.encoding == ATTR_VAL_REF_SECTION)) {
          u->lineoff = val.u.uint;
        }
        break;

      case DW_AT_name:
        if (abbrev->tag == DW_TAG_compile_unit ||
            abbrev->tag == DW_TAG_skeleton_unit) {
          name_val = val;
          have_name_val = 1;
        }
        break;

      case DW_AT_comp_dir:
        if (abbrev->tag == DW_TAG_compile_unit ||
            abbrev->tag == DW_TAG_skeleton_unit) {
          comp_dir_val = val;
          have_comp_dir_val = 1;
        }
        break;

      case DW_AT_str_offsets_base:
        if ((abbrev->tag == DW_TAG_compile_unit ||
             abbrev->tag == DW_TAG_skeleton_unit) &&
            val.encoding == ATTR_VAL_REF_SECTION) {
          u->str_offsets_base = val.u.uint;
        }
        break;

      case DW_AT_addr_base:
        if ((abbrev->tag == DW_TAG_compile_unit ||
             abbrev->tag == DW_TAG_skeleton_unit) &&
            val.encoding == ATTR_VAL_REF_SECTION) {
          u->addr_base = val.u.uint;
        }
        break;

      case DW_AT_rnglists_base:
        if ((abbrev->tag == DW_TAG_compile_unit ||
             abbrev->tag == DW_TAG_skeleton_unit) &&
            val.encoding == ATTR_VAL_REF_SECTION) {
          u->rnglists_base = val.u.uint;
        }
        break;

      default:
        break;
      }
    }

    // Resolve strings after we're sure that we have seen
    // DW_AT_str_offsets_base.
    if (have_name_val) {
      if (!resolve_string(self, dwarf_sections, u->is_dwarf64, is_bigendian,
                          u->str_offsets_base, &name_val, error_cb, data,
                          &u->filename))
        return 0;
    }
    if (have_comp_dir_val) {
      if (!resolve_string(self, dwarf_sections, u->is_dwarf64, is_bigendian,
                          u->str_offsets_base, &comp_dir_val, error_cb, data,
                          &u->comp_dir))
        return 0;
    }

    if (abbrev->tag == DW_TAG_compile_unit ||
        abbrev->tag == DW_TAG_subprogram ||
        abbrev->tag == DW_TAG_skeleton_unit) {
      if (!add_ranges(self, dwarf_sections, base_address, is_bigendian, u,
                      pcrange.lowpc, &pcrange, add_unit_addr, (void *)u,
                      error_cb, data, (void *)addrs))
        return 0;

      /* If we found the PC range in the DW_TAG_compile_unit or
         DW_TAG_skeleton_unit, we can stop now.  */
      if ((abbrev->tag == DW_TAG_compile_unit ||
           abbrev->tag == DW_TAG_skeleton_unit) &&
          (pcrange.have_ranges || (pcrange.have_lowpc && pcrange.have_highpc)))
        return 1;
    }

    if (abbrev->has_children) {
      if (!find_address_ranges(self, base_address, unit_buf, dwarf_sections,
                               is_bigendian, altlink, error_cb, data, u, addrs,
                               NULL))
        return 0;
    }
  }

  return 1;
}

/* Build a mapping from address ranges to the compilation units where
   the line number information for that range can be found.  Returns 1
   on success, 0 on failure.  */

static int build_address_map(ten_backtrace_t *self, uintptr_t base_address,
                             const struct dwarf_sections *dwarf_sections,
                             int is_bigendian, struct dwarf_data *altlink,
                             ten_backtrace_error_func_t error_cb, void *data,
                             struct unit_addrs_vector *addrs,
                             struct unit_vector *unit_vec) {
  struct dwarf_buf info;
  ten_vector_t units;
  size_t units_count;
  size_t i;
  struct unit **pu;
  size_t unit_offset = 0;
  struct unit_addrs *pa;

  memset(&addrs->vec, 0, sizeof addrs->vec);
  memset(&unit_vec->vec, 0, sizeof unit_vec->vec);
  addrs->count = 0;
  unit_vec->count = 0;

  /* Read through the .debug_info section.  FIXME: Should we use the
     .debug_aranges section?  gdb and addr2line don't use it, but I'm
     not sure why.  */

  info.name = ".debug_info";
  info.start = dwarf_sections->data[DEBUG_INFO];
  info.buf = info.start;
  info.left = dwarf_sections->size[DEBUG_INFO];
  info.is_bigendian = is_bigendian;
  info.error_cb = error_cb;
  info.data = data;
  info.reported_underflow = 0;

  memset(&units, 0, sizeof units);
  units_count = 0;

  while (info.left > 0) {
    const unsigned char *unit_data_start;
    uint64_t len;
    int is_dwarf64;
    struct dwarf_buf unit_buf;
    int version;
    int unit_type;
    uint64_t abbrev_offset;
    int addrsize;
    struct unit *u;
    enum dwarf_tag unit_tag;

    if (info.reported_underflow) {
      goto fail;
    }

    unit_data_start = info.buf;

    len = read_initial_length(self, &info, &is_dwarf64);
    unit_buf = info;
    unit_buf.left = len;

    if (!advance(self, &info, len)) {
      goto fail;
    }

    version = read_uint16(self, &unit_buf);
    if (version < 2 || version > 5) {
      dwarf_buf_error(self, &unit_buf, "unrecognized DWARF version", -1);
      goto fail;
    }

    if (version < 5) {
      unit_type = 0;
    } else {
      unit_type = read_byte(self, &unit_buf);
      if (unit_type == DW_UT_type || unit_type == DW_UT_split_type) {
        /* This unit doesn't have anything we need.  */
        continue;
      }
    }

    pu = (struct unit **)ten_vector_grow(&units, sizeof(struct unit *));
    if (pu == NULL) {
      goto fail;
    }

    u = (struct unit *)malloc(sizeof *u);
    if (u == NULL) {
      goto fail;
    }

    *pu = u;
    ++units_count;

    if (version < 5) {
      addrsize = 0; /* Set below.  */
    } else {
      addrsize = read_byte(self, &unit_buf);
    }

    memset(&u->abbrevs, 0, sizeof u->abbrevs);
    abbrev_offset = read_offset(self, &unit_buf, is_dwarf64);
    if (!read_abbrevs(self, abbrev_offset, dwarf_sections->data[DEBUG_ABBREV],
                      dwarf_sections->size[DEBUG_ABBREV], is_bigendian,
                      error_cb, data, &u->abbrevs)) {
      goto fail;
    }

    if (version < 5) {
      addrsize = read_byte(self, &unit_buf);
    }

    switch (unit_type) {
    case 0:
      break;
    case DW_UT_compile:
    case DW_UT_partial:
      break;
    case DW_UT_skeleton:
    case DW_UT_split_compile:
      read_uint64(self, &unit_buf); /* dwo_id */
      break;
    default:
      break;
    }

    u->low_offset = unit_offset;
    unit_offset += len + (is_dwarf64 ? 12 : 4);
    u->high_offset = unit_offset;
    u->unit_data = unit_buf.buf;
    u->unit_data_len = unit_buf.left;
    u->unit_data_offset = unit_buf.buf - unit_data_start;
    u->version = version;
    u->is_dwarf64 = is_dwarf64;
    u->addrsize = addrsize;
    u->filename = NULL;
    u->comp_dir = NULL;
    u->abs_filename = NULL;
    u->lineoff = 0;
    u->str_offsets_base = 0;
    u->addr_base = 0;
    u->rnglists_base = 0;

    /* The actual line number mappings will be read as needed.  */
    u->lines = NULL;
    u->lines_count = 0;
    u->function_addrs = NULL;
    u->function_addrs_count = 0;

    if (!find_address_ranges(self, base_address, &unit_buf, dwarf_sections,
                             is_bigendian, altlink, error_cb, data, u, addrs,
                             &unit_tag)) {
      goto fail;
    }

    if (unit_buf.reported_underflow) {
      goto fail;
    }
  }

  if (info.reported_underflow) {
    goto fail;
  }

  /* Add a trailing addrs entry, but don't include it in addrs->count.  */
  pa = (unit_addrs *)ten_vector_grow(&addrs->vec, sizeof(unit_addrs));
  if (pa == NULL) {
    goto fail;
  }

  pa->low = 0;
  --pa->low;
  pa->high = pa->low;
  pa->u = NULL;

  unit_vec->vec = units;
  unit_vec->count = units_count;

  return 1;

fail:
  if (units_count > 0) {
    pu = (struct unit **)units.data;
    for (i = 0; i < units_count; i++) {
      free_abbrevs(self, &pu[i]->abbrevs, error_cb, data);
      free(pu[i]);
    }
    ten_vector_deinit(&units);
  }
  if (addrs->count > 0) {
    ten_vector_deinit(&addrs->vec);
    addrs->count = 0;
  }
  return 0;
}

/* Add a new mapping to the vector of line mappings that we are
   building.  Returns 1 on success, 0 on failure.  */

static int add_line(ten_backtrace_t *self, struct dwarf_data *ddata,
                    uintptr_t pc, const char *filename, int lineno,
                    ten_backtrace_error_func_t error_cb, void *data,
                    struct line_vector *vec) {
  struct line *ln;

  /* If we are adding the same mapping, ignore it.  This can happen
     when using discriminators.  */
  if (vec->count > 0) {
    ln = (struct line *)vec->vec.data + (vec->count - 1);
    if (pc == ln->pc && filename == ln->filename && lineno == ln->lineno) {
      return 1;
    }
  }

  ln = (line *)ten_vector_grow(&vec->vec, sizeof(line));
  if (ln == NULL) {
    return 0;
  }

  /* Add in the base address here, so that we can look up the PC
     directly.  */
  ln->pc = pc + ddata->base_address;

  ln->filename = filename;
  ln->lineno = lineno;
  ln->idx = vec->count;

  ++vec->count;

  return 1;
}

/* Free the line header information.  */

static void free_line_header(ten_backtrace_t *self, struct line_header *hdr,
                             ten_backtrace_error_func_t error_cb, void *data) {
  if (hdr->dirs_count != 0) {
    free(hdr->dirs);
  }
  free(hdr->filenames);
}

/* Read the directories and file names for a line header for version
   2, setting fields in HDR.  Return 1 on success, 0 on failure.  */

static int read_v2_paths(ten_backtrace_t *self, struct unit *u,
                         struct dwarf_buf *hdr_buf, struct line_header *hdr) {
  const unsigned char *p;
  const unsigned char *pend;
  size_t i;

  /* Count the number of directory entries.  */
  hdr->dirs_count = 0;
  p = hdr_buf->buf;
  pend = p + hdr_buf->left;
  while (p < pend && *p != '\0') {
    p += strnlen((const char *)p, pend - p) + 1;
    ++hdr->dirs_count;
  }

  /* The index of the first entry in the list of directories is 1.  Index 0 is
     used for the current directory of the compilation.  To simplify index
     handling, we set entry 0 to the compilation unit directory.  */
  ++hdr->dirs_count;
  hdr->dirs = (const char **)malloc(hdr->dirs_count * sizeof(const char *));
  if (hdr->dirs == NULL) {
    return 0;
  }

  hdr->dirs[0] = u->comp_dir;
  i = 1;
  while (*hdr_buf->buf != '\0') {
    if (hdr_buf->reported_underflow)
      return 0;

    hdr->dirs[i] = read_string(self, hdr_buf);
    if (hdr->dirs[i] == NULL)
      return 0;
    ++i;
  }
  if (!advance(self, hdr_buf, 1))
    return 0;

  /* Count the number of file entries.  */
  hdr->filenames_count = 0;
  p = hdr_buf->buf;
  pend = p + hdr_buf->left;
  while (p < pend && *p != '\0') {
    p += strnlen((const char *)p, pend - p) + 1;
    p += leb128_len(p);
    p += leb128_len(p);
    p += leb128_len(p);
    ++hdr->filenames_count;
  }

  /* The index of the first entry in the list of file names is 1.  Index 0 is
     used for the DW_AT_name of the compilation unit.  To simplify index
     handling, we set entry 0 to the compilation unit file name.  */
  ++hdr->filenames_count;
  hdr->filenames = (const char **)malloc(hdr->filenames_count * sizeof(char *));
  if (hdr->filenames == NULL) {
    return 0;
  }

  hdr->filenames[0] = u->filename;
  i = 1;
  while (*hdr_buf->buf != '\0') {
    const char *filename;
    uint64_t dir_index;

    if (hdr_buf->reported_underflow)
      return 0;

    filename = read_string(self, hdr_buf);
    if (filename == NULL)
      return 0;
    dir_index = read_uleb128(self, hdr_buf);
    if (IS_ABSOLUTE_PATH(filename) ||
        (dir_index < hdr->dirs_count && hdr->dirs[dir_index] == NULL))
      hdr->filenames[i] = filename;
    else {
      const char *dir;
      size_t dir_len;
      size_t filename_len;
      char *s;

      if (dir_index < hdr->dirs_count)
        dir = hdr->dirs[dir_index];
      else {
        dwarf_buf_error(self, hdr_buf,
                        ("Invalid directory index in "
                         "line number program header"),
                        0);
        return 0;
      }
      dir_len = strlen(dir);
      filename_len = strlen(filename);
      s = (char *)malloc(dir_len + filename_len + 2);
      if (s == NULL) {
        return 0;
      }

      memcpy(s, dir, dir_len);
      /* FIXME: If we are on a DOS-based file system, and the
         directory or the file name use backslashes, then we
         should use a backslash here.  */
      s[dir_len] = '/';
      memcpy(s + dir_len + 1, filename, filename_len + 1);
      hdr->filenames[i] = s;
    }

    /* Ignore the modification time and size.  */
    read_uleb128(self, hdr_buf);
    read_uleb128(self, hdr_buf);

    ++i;
  }

  return 1;
}

/* Read a single version 5 LNCT entry for a directory or file name in a
   line header.  Sets *STRING to the resulting name, ignoring other
   data.  Return 1 on success, 0 on failure.  */

static int read_lnct(ten_backtrace_t *self, struct dwarf_data *ddata,
                     struct unit *u, struct dwarf_buf *hdr_buf,
                     const struct line_header *hdr, size_t formats_count,
                     const struct line_header_format *formats,
                     const char **string) {
  size_t i;
  const char *dir;
  const char *path;

  dir = NULL;
  path = NULL;
  for (i = 0; i < formats_count; i++) {
    struct attr_val val;

    if (!read_attribute(self, formats[i].form, 0, hdr_buf, u->is_dwarf64,
                        u->version, hdr->addrsize, &ddata->dwarf_sections,
                        ddata->altlink, &val))
      return 0;
    switch (formats[i].lnct) {
    case DW_LNCT_path:
      if (!resolve_string(self, &ddata->dwarf_sections, u->is_dwarf64,
                          ddata->is_bigendian, u->str_offsets_base, &val,
                          hdr_buf->error_cb, hdr_buf->data, &path))
        return 0;
      break;
    case DW_LNCT_directory_index:
      if (val.encoding == ATTR_VAL_UINT) {
        if (val.u.uint >= hdr->dirs_count) {
          dwarf_buf_error(self, hdr_buf,
                          ("Invalid directory index in "
                           "line number program header"),
                          0);
          return 0;
        }
        dir = hdr->dirs[val.u.uint];
      }
      break;
    default:
      /* We don't care about timestamps or sizes or hashes.  */
      break;
    }
  }

  if (path == NULL) {
    dwarf_buf_error(self, hdr_buf,
                    "missing file name in line number program header", 0);
    return 0;
  }

  if (dir == NULL)
    *string = path;
  else {
    size_t dir_len;
    size_t path_len;
    char *s;

    dir_len = strlen(dir);
    path_len = strlen(path);
    s = (char *)malloc(dir_len + path_len + 2);
    if (s == NULL) {
      return 0;
    }

    memcpy(s, dir, dir_len);
    /* FIXME: If we are on a DOS-based file system, and the
       directory or the path name use backslashes, then we should
       use a backslash here.  */
    s[dir_len] = '/';
    memcpy(s + dir_len + 1, path, path_len + 1);
    *string = s;
  }

  return 1;
}

/* Read a set of DWARF 5 line header format entries, setting *PCOUNT
   and *PPATHS.  Return 1 on success, 0 on failure.  */

static int
read_line_header_format_entries(ten_backtrace_t *self, struct dwarf_data *ddata,
                                struct unit *u, struct dwarf_buf *hdr_buf,
                                struct line_header *hdr, size_t *pcount,
                                const char ***ppaths) {
  size_t formats_count;
  struct line_header_format *formats;
  size_t paths_count;
  const char **paths;
  size_t i;
  int ret;

  formats_count = read_byte(self, hdr_buf);
  if (formats_count == 0) {
    formats = NULL;
  } else {
    formats = (struct line_header_format *)malloc(
        (formats_count * sizeof(struct line_header_format)));
    if (formats == NULL) {
      return 0;
    }

    for (i = 0; i < formats_count; i++) {
      formats[i].lnct = (int)read_uleb128(self, hdr_buf);
      formats[i].form = (enum dwarf_form)read_uleb128(self, hdr_buf);
    }
  }

  paths_count = read_uleb128(self, hdr_buf);
  if (paths_count == 0) {
    *pcount = 0;
    *ppaths = NULL;
    ret = 1;
    goto exit;
  }

  paths = (const char **)malloc(paths_count * sizeof(const char *));
  if (paths == NULL) {
    ret = 0;
    goto exit;
  }

  for (i = 0; i < paths_count; i++) {
    if (!read_lnct(self, ddata, u, hdr_buf, hdr, formats_count, formats,
                   &paths[i])) {
      free(paths);
      ret = 0;
      goto exit;
    }
  }

  *pcount = paths_count;
  *ppaths = paths;

  ret = 1;

exit:
  if (formats != NULL) {
    free(formats);
  }

  return ret;
}

/* Read the line header.  Return 1 on success, 0 on failure.  */

static int read_line_header(ten_backtrace_t *self, struct dwarf_data *ddata,
                            struct unit *u, int is_dwarf64,
                            struct dwarf_buf *line_buf,
                            struct line_header *hdr) {
  uint64_t hdrlen;
  struct dwarf_buf hdr_buf;

  hdr->version = read_uint16(self, line_buf);
  if (hdr->version < 2 || hdr->version > 5) {
    dwarf_buf_error(self, line_buf, "unsupported line number version", -1);
    return 0;
  }

  if (hdr->version < 5) {
    hdr->addrsize = u->addrsize;
  } else {
    hdr->addrsize = read_byte(self, line_buf);
    /* We could support a non-zero segment_selector_size but I doubt
       we'll ever see it.  */
    if (read_byte(self, line_buf) != 0) {
      dwarf_buf_error(self, line_buf,
                      "non-zero segment_selector_size not supported", -1);
      return 0;
    }
  }

  hdrlen = read_offset(self, line_buf, is_dwarf64);

  hdr_buf = *line_buf;
  hdr_buf.left = hdrlen;

  if (!advance(self, line_buf, hdrlen)) {
    return 0;
  }

  hdr->min_insn_len = read_byte(self, &hdr_buf);
  if (hdr->version < 4) {
    hdr->max_ops_per_insn = 1;
  } else {
    hdr->max_ops_per_insn = read_byte(self, &hdr_buf);
  }

  /* We don't care about default_is_stmt.  */
  read_byte(self, &hdr_buf);

  hdr->line_base = read_sbyte(self, &hdr_buf);
  hdr->line_range = read_byte(self, &hdr_buf);

  hdr->opcode_base = read_byte(self, &hdr_buf);
  hdr->opcode_lengths = hdr_buf.buf;
  if (!advance(self, &hdr_buf, hdr->opcode_base - 1)) {
    return 0;
  }

  if (hdr->version < 5) {
    if (!read_v2_paths(self, u, &hdr_buf, hdr)) {
      return 0;
    }
  } else {
    if (!read_line_header_format_entries(self, ddata, u, &hdr_buf, hdr,
                                         &hdr->dirs_count, &hdr->dirs)) {
      return 0;
    }

    if (!read_line_header_format_entries(self, ddata, u, &hdr_buf, hdr,
                                         &hdr->filenames_count,
                                         &hdr->filenames)) {
      return 0;
    }
  }

  if (hdr_buf.reported_underflow) {
    return 0;
  }

  return 1;
}

/* Read the line program, adding line mappings to VEC.  Return 1 on
   success, 0 on failure.  */

static int read_line_program(ten_backtrace_t *self, struct dwarf_data *ddata,
                             const struct line_header *hdr,
                             struct dwarf_buf *line_buf,
                             struct line_vector *vec) {
  uint64_t address;
  unsigned int op_index;
  const char *reset_filename;
  const char *filename;
  int lineno;

  address = 0;
  op_index = 0;
  if (hdr->filenames_count > 1) {
    reset_filename = hdr->filenames[1];
  } else {
    reset_filename = "";
  }

  filename = reset_filename;
  lineno = 1;
  while (line_buf->left > 0) {
    unsigned int op;

    op = read_byte(self, line_buf);
    if (op >= hdr->opcode_base) {
      unsigned int advance;

      /* Special opcode.  */
      op -= hdr->opcode_base;
      advance = op / hdr->line_range;
      address +=
          (hdr->min_insn_len * (op_index + advance) / hdr->max_ops_per_insn);
      op_index = (op_index + advance) % hdr->max_ops_per_insn;
      lineno += hdr->line_base + (int)(op % hdr->line_range);
      add_line(self, ddata, address, filename, lineno, line_buf->error_cb,
               line_buf->data, vec);
    } else if (op == DW_LNS_extended_op) {
      uint64_t len;

      len = read_uleb128(self, line_buf);
      op = read_byte(self, line_buf);
      switch (op) {
      case DW_LNE_end_sequence:
        /* FIXME: Should we mark the high PC here?  It seems
           that we already have that information from the
           compilation unit.  */
        address = 0;
        op_index = 0;
        filename = reset_filename;
        lineno = 1;
        break;
      case DW_LNE_set_address:
        address = read_address(self, line_buf, hdr->addrsize);
        break;
      case DW_LNE_define_file: {
        const char *f;
        unsigned int dir_index;

        f = read_string(self, line_buf);
        if (f == NULL)
          return 0;
        dir_index = read_uleb128(self, line_buf);
        /* Ignore that time and length.  */
        read_uleb128(self, line_buf);
        read_uleb128(self, line_buf);
        if (IS_ABSOLUTE_PATH(f))
          filename = f;
        else {
          const char *dir;
          size_t dir_len;
          size_t f_len;
          char *p;

          if (dir_index < hdr->dirs_count)
            dir = hdr->dirs[dir_index];
          else {
            dwarf_buf_error(self, line_buf,
                            ("Invalid directory index "
                             "in line number program"),
                            0);
            return 0;
          }
          dir_len = strlen(dir);
          f_len = strlen(f);
          p = (char *)malloc(dir_len + f_len + 2);
          if (p == NULL) {
            return 0;
          }

          memcpy(p, dir, dir_len);
          /* FIXME: If we are on a DOS-based file system,
             and the directory or the file name use
             backslashes, then we should use a backslash
             here.  */
          p[dir_len] = '/';
          memcpy(p + dir_len + 1, f, f_len + 1);
          filename = p;
        }
      } break;
      case DW_LNE_set_discriminator:
        /* We don't care about discriminators.  */
        read_uleb128(self, line_buf);
        break;
      default:
        if (!advance(self, line_buf, len - 1)) {
          return 0;
        }

        break;
      }
    } else {
      switch (op) {
      case DW_LNS_copy:
        add_line(self, ddata, address, filename, lineno, line_buf->error_cb,
                 line_buf->data, vec);
        break;
      case DW_LNS_advance_pc: {
        uint64_t advance;

        advance = read_uleb128(self, line_buf);
        address +=
            (hdr->min_insn_len * (op_index + advance) / hdr->max_ops_per_insn);
        op_index = (op_index + advance) % hdr->max_ops_per_insn;
      } break;
      case DW_LNS_advance_line:
        lineno += (int)read_sleb128(self, line_buf);
        break;
      case DW_LNS_set_file: {
        uint64_t fileno;

        fileno = read_uleb128(self, line_buf);
        if (fileno >= hdr->filenames_count) {
          dwarf_buf_error(self, line_buf,
                          ("Invalid file number in "
                           "line number program"),
                          0);
          return 0;
        }
        filename = hdr->filenames[fileno];
      } break;
      case DW_LNS_set_column:
        read_uleb128(self, line_buf);
        break;
      case DW_LNS_negate_stmt:
        break;
      case DW_LNS_set_basic_block:
        break;
      case DW_LNS_const_add_pc: {
        unsigned int advance;

        op = 255 - hdr->opcode_base;
        advance = op / hdr->line_range;
        address +=
            (hdr->min_insn_len * (op_index + advance) / hdr->max_ops_per_insn);
        op_index = (op_index + advance) % hdr->max_ops_per_insn;
      } break;
      case DW_LNS_fixed_advance_pc:
        address += read_uint16(self, line_buf);
        op_index = 0;
        break;
      case DW_LNS_set_prologue_end:
        break;
      case DW_LNS_set_epilogue_begin:
        break;
      case DW_LNS_set_isa:
        read_uleb128(self, line_buf);
        break;
      default: {
        unsigned int i;

        for (i = hdr->opcode_lengths[op - 1]; i > 0; --i)
          read_uleb128(self, line_buf);
      } break;
      }
    }
  }

  return 1;
}

/* Read the line number information for a compilation unit.  Returns 1
   on success, 0 on failure.  */

static int read_line_info(ten_backtrace_t *self, struct dwarf_data *ddata,
                          ten_backtrace_error_func_t error_cb, void *data,
                          struct unit *u, struct line_header *hdr,
                          struct line **lines, size_t *lines_count) {
  struct line_vector vec;
  struct dwarf_buf line_buf;
  uint64_t len;
  int is_dwarf64;
  struct line *ln;

  memset(&vec.vec, 0, sizeof vec.vec);
  vec.count = 0;

  memset(hdr, 0, sizeof *hdr);

  if (u->lineoff != (off_t)(size_t)u->lineoff ||
      (size_t)u->lineoff >= ddata->dwarf_sections.size[DEBUG_LINE]) {
    error_cb(self, "unit line offset out of range", 0, data);
    goto fail;
  }

  line_buf.name = ".debug_line";
  line_buf.start = ddata->dwarf_sections.data[DEBUG_LINE];
  line_buf.buf = ddata->dwarf_sections.data[DEBUG_LINE] + u->lineoff;
  line_buf.left = ddata->dwarf_sections.size[DEBUG_LINE] - u->lineoff;
  line_buf.is_bigendian = ddata->is_bigendian;
  line_buf.error_cb = error_cb;
  line_buf.data = data;
  line_buf.reported_underflow = 0;

  len = read_initial_length(self, &line_buf, &is_dwarf64);
  line_buf.left = len;

  if (!read_line_header(self, ddata, u, is_dwarf64, &line_buf, hdr)) {
    goto fail;
  }

  if (!read_line_program(self, ddata, hdr, &line_buf, &vec)) {
    goto fail;
  }

  if (line_buf.reported_underflow) {
    goto fail;
  }

  if (vec.count == 0) {
    /* This is not a failure in the sense of a generating an error,
       but it is a failure in that sense that we have no useful
       information.  */
    goto fail;
  }

  /* Allocate one extra entry at the end.  */
  ln = (line *)ten_vector_grow(&vec.vec, sizeof(line));
  if (ln == NULL) {
    goto fail;
  }

  ln->pc = (uintptr_t)-1;
  ln->filename = NULL;
  ln->lineno = 0;
  ln->idx = 0;

  if (!ten_vector_release_remaining_space(&vec.vec)) {
    goto fail;
  }

  ln = (struct line *)vec.vec.data;
  backtrace_sort(ln, vec.count, sizeof(struct line), line_compare);

  *lines = ln;
  *lines_count = vec.count;

  return 1;

fail:
  ten_vector_deinit(&vec.vec);

  free_line_header(self, hdr, error_cb, data);
  *lines = (struct line *)(uintptr_t)-1;
  *lines_count = 0;
  return 0;
}

static const char *read_referenced_name(ten_backtrace_t *self,
                                        struct dwarf_data *, struct unit *,
                                        uint64_t, ten_backtrace_error_func_t,
                                        void *);

/* Read the name of a function from a DIE referenced by ATTR with VAL.  */

static const char *read_referenced_name_from_attr(
    ten_backtrace_t *self, struct dwarf_data *ddata, struct unit *u,
    struct attr *attr, struct attr_val *val,
    ten_backtrace_error_func_t error_cb, void *data) {
  switch (attr->name) {
  case DW_AT_abstract_origin:
  case DW_AT_specification:
    break;
  default:
    return NULL;
  }

  if (attr->form == DW_FORM_ref_sig8)
    return NULL;

  if (val->encoding == ATTR_VAL_REF_INFO) {
    struct unit *unit =
        find_unit(ddata->units, ddata->units_count, val->u.uint);
    if (unit == NULL)
      return NULL;

    uint64_t offset = val->u.uint - unit->low_offset;
    return read_referenced_name(self, ddata, unit, offset, error_cb, data);
  }

  if (val->encoding == ATTR_VAL_UINT || val->encoding == ATTR_VAL_REF_UNIT)
    return read_referenced_name(self, ddata, u, val->u.uint, error_cb, data);

  if (val->encoding == ATTR_VAL_REF_ALT_INFO) {
    struct unit *alt_unit = find_unit(ddata->altlink->units,
                                      ddata->altlink->units_count, val->u.uint);
    if (alt_unit == NULL)
      return NULL;

    uint64_t offset = val->u.uint - alt_unit->low_offset;
    return read_referenced_name(self, ddata->altlink, alt_unit, offset,
                                error_cb, data);
  }

  return NULL;
}

/* Read the name of a function from a DIE referenced by a
   DW_AT_abstract_origin or DW_AT_specification tag.  OFFSET is within
   the same compilation unit.  */

static const char *read_referenced_name(ten_backtrace_t *self,
                                        struct dwarf_data *ddata,
                                        struct unit *u, uint64_t offset,
                                        ten_backtrace_error_func_t error_cb,
                                        void *data) {
  struct dwarf_buf unit_buf;
  uint64_t code;
  const struct abbrev *abbrev;
  const char *ret;
  size_t i;

  /* OFFSET is from the start of the data for this compilation unit.
     U->unit_data is the data, but it starts U->unit_data_offset bytes
     from the beginning.  */

  if (offset < u->unit_data_offset ||
      offset - u->unit_data_offset >= u->unit_data_len) {
    error_cb(self, "abstract origin or specification out of range", 0, data);
    return NULL;
  }

  offset -= u->unit_data_offset;

  unit_buf.name = ".debug_info";
  unit_buf.start = ddata->dwarf_sections.data[DEBUG_INFO];
  unit_buf.buf = u->unit_data + offset;
  unit_buf.left = u->unit_data_len - offset;
  unit_buf.is_bigendian = ddata->is_bigendian;
  unit_buf.error_cb = error_cb;
  unit_buf.data = data;
  unit_buf.reported_underflow = 0;

  code = read_uleb128(self, &unit_buf);
  if (code == 0) {
    dwarf_buf_error(self, &unit_buf, "Invalid abstract origin or specification",
                    0);
    return NULL;
  }

  abbrev = lookup_abbrev(self, &u->abbrevs, code, error_cb, data);
  if (abbrev == NULL) {
    return NULL;
  }

  ret = NULL;
  for (i = 0; i < abbrev->num_attrs; ++i) {
    struct attr_val val;

    if (!read_attribute(self, abbrev->attrs[i].form, abbrev->attrs[i].val,
                        &unit_buf, u->is_dwarf64, u->version, u->addrsize,
                        &ddata->dwarf_sections, ddata->altlink, &val))
      return NULL;

    switch (abbrev->attrs[i].name) {
    case DW_AT_name:
      /* Third name preference: don't override.  A name we found in some
         other way, will normally be more useful -- e.g., this name is
         normally not mangled.  */
      if (ret != NULL) {
        break;
      }

      if (!resolve_string(self, &ddata->dwarf_sections, u->is_dwarf64,
                          ddata->is_bigendian, u->str_offsets_base, &val,
                          error_cb, data, &ret)) {
        return NULL;
      }
      break;

    case DW_AT_linkage_name:
    case DW_AT_MIPS_linkage_name:
      /* First name preference: override all.  */
      {
        const char *s;

        s = NULL;
        if (!resolve_string(self, &ddata->dwarf_sections, u->is_dwarf64,
                            ddata->is_bigendian, u->str_offsets_base, &val,
                            error_cb, data, &s))
          return NULL;
        if (s != NULL)
          return s;
      }
      break;

    case DW_AT_specification:
      /* Second name preference: override DW_AT_name, don't override
         DW_AT_linkage_name.  */
      {
        const char *name;

        name = read_referenced_name_from_attr(self, ddata, u, &abbrev->attrs[i],
                                              &val, error_cb, data);
        if (name != NULL)
          ret = name;
      }
      break;

    default:
      break;
    }
  }

  return ret;
}

/* Add a range to a unit that maps to a function.  This is called via
   add_ranges.  Returns 1 on success, 0 on error.  */

static int add_function_range(ten_backtrace_t *self, void *rdata,
                              uintptr_t lowpc, uintptr_t highpc,
                              ten_backtrace_error_func_t error_cb, void *data,
                              void *pvec) {
  struct function *function = (struct function *)rdata;
  struct function_vector *vec = (struct function_vector *)pvec;
  struct function_addrs *p;

  if (vec->count > 0) {
    p = (struct function_addrs *)vec->vec.data + (vec->count - 1);
    if ((lowpc == p->high || lowpc == p->high + 1) && function == p->function) {
      if (highpc > p->high)
        p->high = highpc;
      return 1;
    }
  }

  p = (function_addrs *)ten_vector_grow(&vec->vec, sizeof(function_addrs));
  if (p == NULL) {
    return 0;
  }

  p->low = lowpc;
  p->high = highpc;
  p->function = function;

  ++vec->count;

  return 1;
}

/* Read one entry plus all its children.  Add function addresses to
   VEC.  Returns 1 on success, 0 on error.  */

static int read_function_entry(ten_backtrace_t *self, struct dwarf_data *ddata,
                               struct unit *u, uintptr_t base,
                               struct dwarf_buf *unit_buf,
                               const struct line_header *lhdr,
                               ten_backtrace_error_func_t error_cb, void *data,
                               struct function_vector *vec_function,
                               struct function_vector *vec_inlined) {
  while (unit_buf->left > 0) {
    uint64_t code;
    const struct abbrev *abbrev;
    int is_function;
    struct function *function;
    struct function_vector *vec;
    size_t i;
    struct pcrange pcrange;
    int have_linkage_name;

    code = read_uleb128(self, unit_buf);
    if (code == 0)
      return 1;

    abbrev = lookup_abbrev(self, &u->abbrevs, code, error_cb, data);
    if (abbrev == NULL)
      return 0;

    is_function = (abbrev->tag == DW_TAG_subprogram ||
                   abbrev->tag == DW_TAG_entry_point ||
                   abbrev->tag == DW_TAG_inlined_subroutine);

    if (abbrev->tag == DW_TAG_inlined_subroutine)
      vec = vec_inlined;
    else
      vec = vec_function;

    function = NULL;
    if (is_function) {
      function = (struct function *)malloc(sizeof *function);
      if (function == NULL) {
        return 0;
      }

      memset(function, 0, sizeof *function);
    }

    memset(&pcrange, 0, sizeof pcrange);
    have_linkage_name = 0;
    for (i = 0; i < abbrev->num_attrs; ++i) {
      struct attr_val val;

      if (!read_attribute(self, abbrev->attrs[i].form, abbrev->attrs[i].val,
                          unit_buf, u->is_dwarf64, u->version, u->addrsize,
                          &ddata->dwarf_sections, ddata->altlink, &val)) {
        return 0;
      }

      /* The compile unit sets the base address for any address
         ranges in the function entries.  */
      if ((abbrev->tag == DW_TAG_compile_unit ||
           abbrev->tag == DW_TAG_skeleton_unit) &&
          abbrev->attrs[i].name == DW_AT_low_pc) {
        if (val.encoding == ATTR_VAL_ADDRESS)
          base = (uintptr_t)val.u.uint;
        else if (val.encoding == ATTR_VAL_ADDRESS_INDEX) {
          if (!resolve_addr_index(self, &ddata->dwarf_sections, u->addr_base,
                                  u->addrsize, ddata->is_bigendian, val.u.uint,
                                  error_cb, data, &base))
            return 0;
        }
      }

      if (is_function) {
        switch (abbrev->attrs[i].name) {
        case DW_AT_call_file:
          if (val.encoding == ATTR_VAL_UINT) {
            if (val.u.uint >= lhdr->filenames_count) {
              dwarf_buf_error(self, unit_buf,
                              ("Invalid file number in "
                               "DW_AT_call_file attribute"),
                              0);
              return 0;
            }
            function->caller_filename = lhdr->filenames[val.u.uint];
          }
          break;

        case DW_AT_call_line:
          if (val.encoding == ATTR_VAL_UINT)
            function->caller_lineno = val.u.uint;
          break;

        case DW_AT_abstract_origin:
        case DW_AT_specification:
          /* Second name preference: override DW_AT_name, don't override
             DW_AT_linkage_name.  */
          if (have_linkage_name)
            break;
          {
            const char *name;

            name = read_referenced_name_from_attr(
                self, ddata, u, &abbrev->attrs[i], &val, error_cb, data);
            if (name != NULL)
              function->name = name;
          }
          break;

        case DW_AT_name:
          /* Third name preference: don't override.  */
          if (function->name != NULL)
            break;
          if (!resolve_string(self, &ddata->dwarf_sections, u->is_dwarf64,
                              ddata->is_bigendian, u->str_offsets_base, &val,
                              error_cb, data, &function->name))
            return 0;
          break;

        case DW_AT_linkage_name:
        case DW_AT_MIPS_linkage_name:
          /* First name preference: override all.  */
          {
            const char *s;

            s = NULL;
            if (!resolve_string(self, &ddata->dwarf_sections, u->is_dwarf64,
                                ddata->is_bigendian, u->str_offsets_base, &val,
                                error_cb, data, &s))
              return 0;
            if (s != NULL) {
              function->name = s;
              have_linkage_name = 1;
            }
          }
          break;

        case DW_AT_low_pc:
        case DW_AT_high_pc:
        case DW_AT_ranges:
          update_pcrange(&abbrev->attrs[i], &val, &pcrange);
          break;

        default:
          break;
        }
      }
    }

    /* If we couldn't find a name for the function, we have no use
       for it.  */
    if (is_function && function->name == NULL) {
      free(function);
      is_function = 0;
    }

    if (is_function) {
      if (pcrange.have_ranges || (pcrange.have_lowpc && pcrange.have_highpc)) {
        if (!add_ranges(self, &ddata->dwarf_sections, ddata->base_address,
                        ddata->is_bigendian, u, base, &pcrange,
                        add_function_range, (void *)function, error_cb, data,
                        (void *)vec)) {
          return 0;
        }
      } else {
        free(function);
        is_function = 0;
      }
    }

    if (abbrev->has_children) {
      if (!is_function) {
        if (!read_function_entry(self, ddata, u, base, unit_buf, lhdr, error_cb,
                                 data, vec_function, vec_inlined))
          return 0;
      } else {
        struct function_vector fvec;

        /* Gather any information for inlined functions in
           FVEC.  */

        memset(&fvec, 0, sizeof fvec);

        if (!read_function_entry(self, ddata, u, base, unit_buf, lhdr, error_cb,
                                 data, vec_function, &fvec)) {
          return 0;
        }

        if (fvec.count > 0) {
          struct function_addrs *p;
          struct function_addrs *faddrs;

          /* Allocate a trailing entry, but don't include it
             in fvec.count.  */
          p = (function_addrs *)ten_vector_grow(&fvec.vec,
                                                sizeof(function_addrs));
          if (p == NULL) {
            return 0;
          }

          p->low = 0;
          --p->low;
          p->high = p->low;
          p->function = NULL;

          if (!ten_vector_release_remaining_space(&fvec.vec)) {
            return 0;
          }

          faddrs = (struct function_addrs *)fvec.vec.data;
          backtrace_sort(faddrs, fvec.count, sizeof(struct function_addrs),
                         function_addrs_compare);

          function->function_addrs = faddrs;
          function->function_addrs_count = fvec.count;
        }
      }
    }
  }

  return 1;
}

/**
 * @brief Read function name information for a compilation unit. We look through
 * the whole unit looking for function tags.
 */
static void read_function_info(ten_backtrace_t *self, struct dwarf_data *ddata,
                               const struct line_header *lhdr,
                               ten_backtrace_error_func_t error_cb, void *data,
                               struct unit *u, struct function_vector *fvec,
                               struct function_addrs **ret_addrs,
                               size_t *ret_addrs_count) {
  struct function_vector lvec;
  struct function_vector *pfvec = NULL;
  struct dwarf_buf unit_buf;
  struct function_addrs *p = NULL;
  struct function_addrs *addrs = NULL;
  size_t addrs_count = 0;

  // Use FVEC if it is not NULL. Otherwise use our own vector.
  if (fvec != NULL) {
    pfvec = fvec;
  } else {
    memset(&lvec, 0, sizeof lvec);
    pfvec = &lvec;
  }

  unit_buf.name = ".debug_info";
  unit_buf.start = ddata->dwarf_sections.data[DEBUG_INFO];
  unit_buf.buf = u->unit_data;
  unit_buf.left = u->unit_data_len;
  unit_buf.is_bigendian = ddata->is_bigendian;
  unit_buf.error_cb = error_cb;
  unit_buf.data = data;
  unit_buf.reported_underflow = 0;

  while (unit_buf.left > 0) {
    if (!read_function_entry(self, ddata, u, 0, &unit_buf, lhdr, error_cb, data,
                             pfvec, pfvec)) {
      return;
    }
  }

  if (pfvec->count == 0) {
    return;
  }

  // Allocate a trailing entry, but don't include it in pfvec->count.
  p = (function_addrs *)ten_vector_grow(&pfvec->vec, sizeof(function_addrs));
  if (p == NULL) {
    return;
  }

  p->low = 0;
  --p->low;
  p->high = p->low;
  p->function = NULL;

  addrs_count = pfvec->count;

  if (fvec == NULL) {
    if (!ten_vector_release_remaining_space(&lvec.vec)) {
      return;
    }

    addrs = (function_addrs *)pfvec->vec.data;
  } else {
    // Finish this list of addresses, but leave the remaining space in the
    // vector available for the next function unit.
    addrs = (function_addrs *)ten_vector_take_out(&fvec->vec);
    if (addrs == NULL) {
      return;
    }

    fvec->count = 0;
  }

  backtrace_sort(addrs, addrs_count, sizeof(struct function_addrs),
                 function_addrs_compare);

  *ret_addrs = addrs;
  *ret_addrs_count = addrs_count;
}

/* See if PC is inlined in FUNCTION.  If it is, print out the inlined
   information, and update FILENAME and LINENO for the caller.
   Returns whatever CALLBACK returns, or 0 to keep going.  */

static int
report_inlined_functions(ten_backtrace_t *self, uintptr_t pc,
                         struct function *function,
                         ten_backtrace_dump_file_line_func_t dump_file_line_cb,
                         void *data, const char **filename, int *lineno) {
  struct function_addrs *p;
  struct function_addrs *match;
  struct function *inlined;
  int ret;

  if (function->function_addrs_count == 0) {
    return 0;
  }

  /* Our search isn't safe if pc == -1, as that is the sentinel
     value.  */
  if (pc + 1 == 0) {
    return 0;
  }

  p = ((struct function_addrs *)bsearch(
      &pc, function->function_addrs, function->function_addrs_count,
      sizeof(struct function_addrs), function_addrs_search));
  if (p == NULL) {
    return 0;
  }

  /* Here pc >= p->low && pc < (p + 1)->low.  The function_addrs are
     sorted by low, so if pc > p->low we are at the end of a range of
     function_addrs with the same low value.  If pc == p->low walk
     forward to the end of the range with that low value.  Then walk
     backward and use the first range that includes pc.  */
  while (pc == (p + 1)->low) {
    ++p;
  }

  match = NULL;
  while (1) {
    if (pc < p->high) {
      match = p;
      break;
    }
    if (p == function->function_addrs) {
      break;
    }
    if ((p - 1)->low < p->low) {
      break;
    }
    --p;
  }
  if (match == NULL) {
    return 0;
  }

  /* We found an inlined call.  */

  inlined = match->function;

  /* Report any calls inlined into this one.  */
  ret = report_inlined_functions(self, pc, inlined, dump_file_line_cb, data,
                                 filename, lineno);
  if (ret != 0) {
    return ret;
  }

  /* Report this inlined call.  */
  ret = dump_file_line_cb(self, pc, *filename, *lineno, inlined->name, data);
  if (ret != 0) {
    return ret;
  }

  /* Our caller will report the caller of the inlined function; tell
     it the appropriate filename and line number.  */
  *filename = inlined->caller_filename;
  *lineno = inlined->caller_lineno;

  return 0;
}

/**
 * @brief Look for a @a pc in the DWARF mapping for one module. On success, call
 * @a callback and return whatever it returns. On error, call @a error_cb
 * and return 0. Sets @a *found to 1 if the @a pc is found, 0 if not.
 */
static int
dwarf_lookup_pc(ten_backtrace_t *self, struct dwarf_data *ddata, uintptr_t pc,
                ten_backtrace_dump_file_line_func_t dump_file_line_cb,
                ten_backtrace_error_func_t error_cb, void *data, int *found) {
  struct unit_addrs *entry = NULL;
  int found_entry = 0;
  struct unit *u = NULL;
  int new_data = 0;
  struct line *lines = NULL;
  struct line *ln = NULL;
  struct function_addrs *p = NULL;
  struct function_addrs *fmatch = NULL;
  struct function *function = NULL;
  const char *filename = NULL;
  int lineno = 0;
  int ret = 0;

  *found = 1;

  /* Find an address range that includes PC.  Our search isn't safe if
     PC == -1, as we use that as a sentinel value, so skip the search
     in that case.  */
  entry = (ddata->addrs_count == 0 || pc + 1 == 0
               ? NULL
               : bsearch(&pc, ddata->addrs, ddata->addrs_count,
                         sizeof(struct unit_addrs), unit_addrs_search));

  if (entry == NULL) {
    *found = 0;
    return 0;
  }

  /* Here pc >= entry->low && pc < (entry + 1)->low.  The unit_addrs
     are sorted by low, so if pc > p->low we are at the end of a range
     of unit_addrs with the same low value.  If pc == p->low walk
     forward to the end of the range with that low value.  Then walk
     backward and use the first range that includes pc.  */
  while (pc == (entry + 1)->low) {
    ++entry;
  }

  found_entry = 0;

  while (1) {
    if (pc < entry->high) {
      found_entry = 1;
      break;
    }
    if (entry == ddata->addrs) {
      break;
    }

    if ((entry - 1)->low < entry->low) {
      break;
    }
    --entry;
  }
  if (!found_entry) {
    *found = 0;
    return 0;
  }

  /* We need the lines, lines_count, function_addrs,
     function_addrs_count fields of u.  If they are not set, we need
     to set them.  When running in threaded mode, we need to allow for
     the possibility that some other thread is setting them
     simultaneously.  */

  u = entry->u;
  lines = u->lines;

  /* Skip units with no useful line number information by walking
     backward.  Useless line number information is marked by setting
     lines == -1.  */
  while (entry > ddata->addrs && pc >= (entry - 1)->low &&
         pc < (entry - 1)->high) {
    lines = (struct line *)ten_atomic_ptr_load((void *)&u->lines);
    if (lines != (struct line *)(uintptr_t)-1) {
      break;
    }

    --entry;

    u = entry->u;
    lines = u->lines;
  }

  lines = ten_atomic_ptr_load((void *)&u->lines);

  new_data = 0;
  if (lines == NULL) {
    struct function_addrs *function_addrs = NULL;
    size_t function_addrs_count = 0;
    size_t count = 0;
    struct line_header lhdr;

    // We have never read the line information for this unit. Read it now.

    function_addrs = NULL;
    function_addrs_count = 0;
    if (read_line_info(self, ddata, error_cb, data, entry->u, &lhdr, &lines,
                       &count)) {
      struct function_vector *pfvec = NULL;

      read_function_info(self, ddata, &lhdr, error_cb, data, entry->u, pfvec,
                         &function_addrs, &function_addrs_count);
      free_line_header(self, &lhdr, error_cb, data);
      new_data = 1;
    }

    /* Atomically store the information we just read into the unit.
       If another thread is simultaneously writing, it presumably
       read the same information, and we don't care which one we
       wind up with; we just leak the other one.  We do have to
       write the lines field last, so that the acquire-loads above
       ensure that the other fields are set.  */

    ten_atomic_store(&u->lines_count, count);
    ten_atomic_store(&u->function_addrs_count, function_addrs_count);
    ten_atomic_ptr_store((void *)&u->function_addrs, function_addrs);
    ten_atomic_ptr_store((void *)&u->lines, lines);
  }

  // Now all fields of U have been initialized.

  if (lines == (struct line *)(uintptr_t)-1) {
    /* If reading the line number information failed in some way,
       try again to see if there is a better compilation unit for
       this PC.  */
    if (new_data) {
      return dwarf_lookup_pc(self, ddata, pc, dump_file_line_cb, error_cb, data,
                             found);
    }

    return dump_file_line_cb(self, pc, NULL, 0, NULL, data);
  }

  /* Search for PC within this unit.  */

  ln = (struct line *)bsearch(&pc, lines, entry->u->lines_count,
                              sizeof(struct line), line_search);
  if (ln == NULL) {
    /* The PC is between the low_pc and high_pc attributes of the
       compilation unit, but no entry in the line table covers it.
       This implies that the start of the compilation unit has no
       line number information.  */

    if (entry->u->abs_filename == NULL) {
      const char *filename;

      filename = entry->u->filename;
      if (filename != NULL && !IS_ABSOLUTE_PATH(filename) &&
          entry->u->comp_dir != NULL) {
        size_t filename_len;
        const char *dir;
        size_t dir_len;
        char *s;

        filename_len = strlen(filename);
        dir = entry->u->comp_dir;
        dir_len = strlen(dir);
        s = (char *)malloc(dir_len + filename_len + 2);
        if (s == NULL) {
          *found = 0;
          return 0;
        }

        memcpy(s, dir, dir_len);
        // TODO(Wei): Should use backslash if DOS file system.
        s[dir_len] = '/';
        memcpy(s + dir_len + 1, filename, filename_len + 1);
        filename = s;
      }
      entry->u->abs_filename = filename;
    }

    return dump_file_line_cb(self, pc, entry->u->abs_filename, 0, NULL, data);
  }

  /* Search for function name within this unit.  */

  if (entry->u->function_addrs_count == 0) {
    return dump_file_line_cb(self, pc, ln->filename, ln->lineno, NULL, data);
  }

  p = ((struct function_addrs *)bsearch(
      &pc, entry->u->function_addrs, entry->u->function_addrs_count,
      sizeof(struct function_addrs), function_addrs_search));
  if (p == NULL) {
    return dump_file_line_cb(self, pc, ln->filename, ln->lineno, NULL, data);
  }

  /* Here pc >= p->low && pc < (p + 1)->low.  The function_addrs are
     sorted by low, so if pc > p->low we are at the end of a range of
     function_addrs with the same low value.  If pc == p->low walk
     forward to the end of the range with that low value.  Then walk
     backward and use the first range that includes pc.  */
  while (pc == (p + 1)->low) {
    ++p;
  }

  fmatch = NULL;
  while (1) {
    if (pc < p->high) {
      fmatch = p;
      break;
    }

    if (p == entry->u->function_addrs) {
      break;
    }

    if ((p - 1)->low < p->low) {
      break;
    }

    --p;
  }
  if (fmatch == NULL) {
    return dump_file_line_cb(self, pc, ln->filename, ln->lineno, NULL, data);
  }

  function = fmatch->function;

  filename = ln->filename;
  lineno = ln->lineno;

  ret = report_inlined_functions(self, pc, function, dump_file_line_cb, data,
                                 &filename, &lineno);
  if (ret != 0) {
    return ret;
  }

  return dump_file_line_cb(self, pc, filename, lineno, function->name, data);
}

/**
 * @brief Return the file/line information for a PC using the DWARF mapping we
 * built earlier.
 */
static int dwarf_fileline(ten_backtrace_t *self_, uintptr_t pc,
                          ten_backtrace_dump_file_line_func_t dump_file_line_cb,
                          ten_backtrace_error_func_t error_cb, void *data) {
  ten_backtrace_posix_t *self = (ten_backtrace_posix_t *)self_;

  struct dwarf_data **pp = (struct dwarf_data **)&self->get_file_line_user_data;
  while (1) {
    struct dwarf_data *ddata = ten_atomic_ptr_load((void *)pp);
    if (ddata == NULL) {
      break;
    }

    int found = 0;
    int ret = dwarf_lookup_pc(self_, ddata, pc, dump_file_line_cb, error_cb,
                              data, &found);
    if (ret != 0 || found) {
      return ret;
    }

    pp = &ddata->next;
  }

  // TODO(Wei): See if any libraries have been dlopen'ed.

  return dump_file_line_cb(self_, pc, NULL, 0, NULL, data);
}

/* Initialize our data structures from the DWARF debug info for a
   file.  Return NULL on failure.  */

static struct dwarf_data *
build_dwarf_data(ten_backtrace_t *self, uintptr_t base_address,
                 const struct dwarf_sections *dwarf_sections, int is_bigendian,
                 struct dwarf_data *altlink,
                 ten_backtrace_error_func_t error_cb, void *data) {
  struct unit_addrs_vector addrs_vec;
  struct unit_addrs *addrs;
  size_t addrs_count;
  struct unit_vector units_vec;
  struct unit **units;
  size_t units_count;
  struct dwarf_data *fdata;

  if (!build_address_map(self, base_address, dwarf_sections, is_bigendian,
                         altlink, error_cb, data, &addrs_vec, &units_vec)) {
    return NULL;
  }

  if (!ten_vector_release_remaining_space(&addrs_vec.vec)) {
    return NULL;
  }

  if (!ten_vector_release_remaining_space(&units_vec.vec)) {
    return NULL;
  }

  addrs = (struct unit_addrs *)addrs_vec.vec.data;
  units = (struct unit **)units_vec.vec.data;
  addrs_count = addrs_vec.count;
  units_count = units_vec.count;
  backtrace_sort(addrs, addrs_count, sizeof(struct unit_addrs),
                 unit_addrs_compare);
  /* No qsort for units required, already sorted.  */

  fdata = (struct dwarf_data *)malloc(sizeof(struct dwarf_data));
  if (fdata == NULL) {
    return NULL;
  }

  fdata->next = NULL;
  fdata->altlink = altlink;
  fdata->base_address = base_address;
  fdata->addrs = addrs;
  fdata->addrs_count = addrs_count;
  fdata->units = units;
  fdata->units_count = units_count;
  fdata->dwarf_sections = *dwarf_sections;
  fdata->is_bigendian = is_bigendian;
  memset(&fdata->fvec, 0, sizeof fdata->fvec);

  return fdata;
}

/**
 * @brief Build our data structures from the DWARF sections for a module. Set
 * @a fileline_fn and @a state->file_line_data.
 *
 * @return 1 on success, 0 on failure.
 */
int backtrace_dwarf_add(ten_backtrace_t *self_, uintptr_t base_address,
                        const struct dwarf_sections *dwarf_sections,
                        int is_bigendian, struct dwarf_data *fileline_altlink,
                        ten_backtrace_error_func_t error_cb, void *data,
                        ten_backtrace_get_file_line_func_t *fileline_fn,
                        struct dwarf_data **fileline_entry) {
  ten_backtrace_posix_t *self = (ten_backtrace_posix_t *)self_;

  struct dwarf_data *fdata =
      build_dwarf_data(self_, base_address, dwarf_sections, is_bigendian,
                       fileline_altlink, error_cb, data);
  if (fdata == NULL) {
    return 0;
  }

  if (fileline_entry != NULL) {
    *fileline_entry = fdata;
  }

  while (1) {
    struct dwarf_data **pp =
        (struct dwarf_data **)&self->get_file_line_user_data;

    // Find the last element in the 'dwarf_data' list.
    while (1) {
      struct dwarf_data *p = ten_atomic_ptr_load((void *)pp);
      if (p == NULL) {
        break;
      }

      pp = &p->next;
    }

    if (__sync_bool_compare_and_swap(pp, NULL, fdata)) {
      break;
    }
  }

  *fileline_fn = dwarf_fileline;

  return 1;
}
