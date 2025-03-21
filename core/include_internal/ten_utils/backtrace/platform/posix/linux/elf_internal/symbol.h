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

#include <stddef.h>

#include "include_internal/ten_utils/backtrace/backtrace.h"
#include "include_internal/ten_utils/backtrace/platform/posix/linux/elf.h"

// Information we keep for an ELF symbol.
struct elf_symbol {
  // The name of the symbol.
  const char *name;
  // The address of the symbol.
  uintptr_t address;
  // The size of the symbol.
  size_t size;
};

// Information to pass to elf_syminfo.
struct elf_syminfo_data {
  // Symbols for the next module.
  struct elf_syminfo_data *next;
  // The ELF symbols, sorted by address.
  struct elf_symbol *symbols;
  // The number of symbols.
  size_t count;
};

// Information about PowerPC64 ELFv1 .opd section.
struct elf_ppc64_opd_data {
  // Address of the .opd section.
  b_elf_addr addr;
  // Section data.
  const char *data;
  // Size of the .opd section.
  size_t size;
  // Corresponding section view.
  struct elf_view view;
};

TEN_UTILS_PRIVATE_API void elf_add_syminfo_data(ten_backtrace_t *self_,
                                                struct elf_syminfo_data *edata);

TEN_UTILS_PRIVATE_API int elf_initialize_syminfo(
    ten_backtrace_t *self, uintptr_t base_address,
    const unsigned char *symtab_data, size_t symtab_size,
    const unsigned char *strtab, size_t strtab_size,
    ten_backtrace_error_func_t error_cb, void *data,
    struct elf_syminfo_data *sdata, struct elf_ppc64_opd_data *opd);
