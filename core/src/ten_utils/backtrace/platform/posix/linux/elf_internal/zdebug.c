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

#include <stdlib.h>
#include <string.h>

#include "include_internal/ten_utils/backtrace/backtrace.h"
#include "include_internal/ten_utils/backtrace/platform/posix/linux/elf.h"
#include "include_internal/ten_utils/backtrace/platform/posix/linux/elf_internal/zdebug.h"

// Uncompress the old compressed debug format, the one emitted by
// --compress-debug-sections=zlib-gnu.  The compressed data is in
// COMPRESSED / COMPRESSED_SIZE, and the function writes to
// *UNCOMPRESSED / *UNCOMPRESSED_SIZE.  ZDEBUG_TABLE is work space to
// hold Huffman tables.  Returns 0 on error, 1 on successful
// decompression or if something goes wrong.  In general we try to
// carry on, by returning 1, even if we can't decompress.
int elf_uncompress_zdebug(ten_backtrace_t *self,
                          const unsigned char *compressed,
                          size_t compressed_size, uint16_t *zdebug_table,
                          ten_backtrace_error_func_t error_cb, void *data,
                          unsigned char **uncompressed,
                          size_t *uncompressed_size) {
  size_t sz = 0;
  size_t i = 0;
  unsigned char *po = NULL;

  *uncompressed = NULL;
  *uncompressed_size = 0;

  // The format starts with the four bytes ZLIB, followed by the 8
  // byte length of the uncompressed data in big-endian order,
  // followed by a zlib stream.

  if (compressed_size < 12 || memcmp(compressed, "ZLIB", 4) != 0) {
    return 1;
  }

  sz = 0;
  for (i = 0; i < 8; i++) {
    sz = (sz << 8) | compressed[i + 4];
  }

  if (*uncompressed != NULL && *uncompressed_size >= sz) {
    po = *uncompressed;
  } else {
    po = (unsigned char *)malloc(sz);
    if (po == NULL) {
      return 0;
    }
  }

  if (!elf_zlib_inflate_and_verify(compressed + 12, compressed_size - 12,
                                   zdebug_table, po, sz)) {
    return 1;
  }

  *uncompressed = po;
  *uncompressed_size = sz;

  return 1;
}

// Uncompress the new compressed debug format, the official standard
// ELF approach emitted by --compress-debug-sections=zlib-gabi.  The
// compressed data is in COMPRESSED / COMPRESSED_SIZE, and the
// function writes to *UNCOMPRESSED / *UNCOMPRESSED_SIZE.
// ZDEBUG_TABLE is work space as for elf_uncompress_zdebug.  Returns 0
// on error, 1 on successful decompression or if something goes wrong.
// In general we try to carry on, by returning 1, even if we can't
// decompress.
int elf_uncompress_chdr(ten_backtrace_t *self, const unsigned char *compressed,
                        size_t compressed_size, uint16_t *zdebug_table,
                        ten_backtrace_error_func_t error_cb, void *data,
                        unsigned char **uncompressed,
                        size_t *uncompressed_size) {
  const b_elf_chdr *chdr = NULL;
  char *alc = NULL;
  size_t alc_len = 0;
  unsigned char *po = NULL;

  *uncompressed = NULL;
  *uncompressed_size = 0;

  // The format starts with an ELF compression header.
  if (compressed_size < sizeof(b_elf_chdr)) {
    return 1;
  }

  chdr = (const b_elf_chdr *)compressed;

  alc = NULL;
  alc_len = 0;
  if (*uncompressed != NULL && *uncompressed_size >= chdr->ch_size) {
    po = *uncompressed;
  } else {
    alc_len = chdr->ch_size;
    alc = malloc(alc_len);
    if (alc == NULL) {
      return 0;
    }
    po = (unsigned char *)alc;
  }

  switch (chdr->ch_type) {
  case ELFCOMPRESS_ZLIB:
    if (!elf_zlib_inflate_and_verify(compressed + sizeof(b_elf_chdr),
                                     compressed_size - sizeof(b_elf_chdr),
                                     zdebug_table, po, chdr->ch_size)) {
      goto skip;
    }
    break;

  case ELFCOMPRESS_ZSTD:
    if (!elf_zstd_decompress(compressed + sizeof(b_elf_chdr),
                             compressed_size - sizeof(b_elf_chdr),
                             (unsigned char *)zdebug_table, po,
                             chdr->ch_size)) {
      goto skip;
    }
    break;

  default:
    // Unsupported compression algorithm.
    goto skip;
  }

  *uncompressed = po;
  *uncompressed_size = chdr->ch_size;

  return 1;

skip:
  if (alc != NULL && alc_len > 0) {
    free(alc);
  }
  return 1;
}
