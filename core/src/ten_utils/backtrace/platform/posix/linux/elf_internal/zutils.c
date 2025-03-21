//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
// This file is modified from
// https://github.com/ianlancetaylor/libbacktrace [BSD license]
//
#include "include_internal/ten_utils/backtrace/platform/posix/linux/elf_internal/zutils.h"
#include "include_internal/ten_utils/backtrace/platform/posix/internal.h"

/**
 * @brief A function useful for setting a breakpoint for an inflation failure
 * when this code is compiled with -g.
 */
void elf_uncompress_failed(void) {}

// *PVAL is the current value being read from the stream, and *PBITS
// is the number of valid bits.  Ensure that *PVAL holds at least 15
// bits by reading additional bits from *PPIN, up to PINEND, as
// needed.  Updates *PPIN, *PVAL and *PBITS.  Returns 1 on success, 0
// on error.
int elf_fetch_bits(const unsigned char **ppin, const unsigned char *pinend,
                   uint64_t *pval, unsigned int *pbits) {
  unsigned int bits = 0;
  const unsigned char *pin = NULL;
  uint64_t val = 0;
  uint32_t next = 0;

  bits = *pbits;
  if (bits >= 15) {
    return 1;
  }
  pin = *ppin;
  val = *pval;

  if (unlikely(pinend - pin < 4)) {
    elf_uncompress_failed();
    return 0;
  }

#if defined(__BYTE_ORDER__) && defined(__ORDER_LITTLE_ENDIAN__) &&             \
    defined(__ORDER_BIG_ENDIAN__) &&                                           \
    (__BYTE_ORDER__ == __ORDER_BIG_ENDIAN__ ||                                 \
     __BYTE_ORDER__ == __ORDER_LITTLE_ENDIAN__)
  // We've ensured that PIN is aligned.
  next = *(const uint32_t *)pin;

#if __BYTE_ORDER__ == __ORDER_BIG_ENDIAN__
  next = __builtin_bswap32(next);
#endif
#else
  next = pin[0] | (pin[1] << 8) | (pin[2] << 16) | (pin[3] << 24);
#endif

  val |= (uint64_t)next << bits;
  bits += 32;
  pin += 4;

  // We will need the next four bytes soon.
  __builtin_prefetch(pin, 0, 0);

  *ppin = pin;
  *pval = val;
  *pbits = bits;

  return 1;
}
