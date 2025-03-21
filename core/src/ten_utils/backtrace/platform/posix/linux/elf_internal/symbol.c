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
#include <stdlib.h>

#include "include_internal/ten_utils/backtrace/platform/posix/internal.h"
#include "include_internal/ten_utils/backtrace/platform/posix/linux/elf.h"
#include "include_internal/ten_utils/backtrace/platform/posix/linux/elf_internal/symbol.h"
#include "include_internal/ten_utils/backtrace/sort.h"
#include "ten_utils/lib/atomic_ptr.h"

// Compare struct elf_symbol for qsort.

static int elf_symbol_compare(const void *v1, const void *v2) {
  const struct elf_symbol *e1 = (const struct elf_symbol *)v1;
  const struct elf_symbol *e2 = (const struct elf_symbol *)v2;

  if (e1->address < e2->address) {
    return -1;
  } else if (e1->address > e2->address) {
    return 1;
  } else {
    return 0;
  }
}

// Initialize the symbol table info for elf_syminfo.
int elf_initialize_syminfo(ten_backtrace_t *self, uintptr_t base_address,
                           const unsigned char *symtab_data, size_t symtab_size,
                           const unsigned char *strtab, size_t strtab_size,
                           ten_backtrace_error_func_t error_cb, void *data,
                           struct elf_syminfo_data *sdata,
                           struct elf_ppc64_opd_data *opd) {
  size_t sym_count = 0;
  const b_elf_sym *sym = NULL;
  size_t elf_symbol_count = 0;
  size_t elf_symbol_size = 0;
  struct elf_symbol *elf_symbols = NULL;
  size_t i = 0;
  unsigned int j = 0;

  sym_count = symtab_size / sizeof(b_elf_sym);

  // We only care about function symbols.  Count them.
  sym = (const b_elf_sym *)symtab_data;
  elf_symbol_count = 0;
  for (i = 0; i < sym_count; ++i, ++sym) {
    int info = sym->st_info & 0xf;
    if ((info == STT_FUNC || info == STT_OBJECT) &&
        sym->st_shndx != SHN_UNDEF) {
      ++elf_symbol_count;
    }
  }

  // Some shared library might not have any exported symbols, ex: the RTC
  // plugins.
  if (elf_symbol_count == 0) {
    return 1;
  }

  elf_symbol_size = elf_symbol_count * sizeof(struct elf_symbol);
  elf_symbols = (struct elf_symbol *)malloc(elf_symbol_size);
  assert(elf_symbols && "Failed to allocate memory.");
  if (elf_symbols == NULL) {
    return 0;
  }

  sym = (const b_elf_sym *)symtab_data;
  j = 0;
  for (i = 0; i < sym_count; ++i, ++sym) {
    int info = sym->st_info & 0xf;
    if (info != STT_FUNC && info != STT_OBJECT) {
      continue;
    }

    if (sym->st_shndx == SHN_UNDEF) {
      continue;
    }

    if (sym->st_name >= strtab_size) {
      error_cb(self, "symbol string index out of range", 0, data);
      free(elf_symbols);
      return 0;
    }
    elf_symbols[j].name = (const char *)strtab + sym->st_name;
    // Special case PowerPC64 ELFv1 symbols in .opd section, if the symbol
    // is a function descriptor, read the actual code address from the
    // descriptor.
    if (opd && sym->st_value >= opd->addr &&
        sym->st_value < opd->addr + opd->size) {
      elf_symbols[j].address =
          *(const b_elf_addr *)(opd->data + (sym->st_value - opd->addr));
    } else {
      elf_symbols[j].address = sym->st_value;
    }
    elf_symbols[j].address += base_address;
    elf_symbols[j].size = sym->st_size;
    ++j;
  }

  backtrace_sort(elf_symbols, elf_symbol_count, sizeof(struct elf_symbol),
                 elf_symbol_compare);

  sdata->next = NULL;
  sdata->symbols = elf_symbols;
  sdata->count = elf_symbol_count;

  return 1;
}

// Add EDATA to the list in STATE.
void elf_add_syminfo_data(ten_backtrace_t *self_,
                          struct elf_syminfo_data *edata) {
  ten_backtrace_posix_t *self = (ten_backtrace_posix_t *)self_;
  assert(self && "Invalid argument.");

  while (1) {
    struct elf_syminfo_data **pp =
        (struct elf_syminfo_data **)(void *)&self->get_syminfo_user_data;

    while (1) {
      struct elf_syminfo_data *p = ten_atomic_ptr_load((void *)pp);
      if (p == NULL) {
        break;
      }

      pp = &p->next;
    }

    if (__sync_bool_compare_and_swap(pp, NULL, edata)) {
      break;
    }
  }
}

// Compare an ADDR against an elf_symbol for bsearch.  We allocate one
// extra entry in the array so that this can look safely at the next
// entry.
static int elf_symbol_search(const void *vkey, const void *ventry) {
  const uintptr_t *key = (const uintptr_t *)vkey;
  const struct elf_symbol *entry = (const struct elf_symbol *)ventry;
  uintptr_t addr = *key;
  if (addr < entry->address) {
    return -1;
  } else if (addr >= entry->address + entry->size) {
    return 1;
  } else {
    return 0;
  }
}

// Return the symbol name and value for an ADDR.
void elf_syminfo(ten_backtrace_t *self_, uintptr_t addr,
                 ten_backtrace_dump_syminfo_func_t callback,
                 ten_backtrace_error_func_t error_cb, void *data) {
  ten_backtrace_posix_t *self = (ten_backtrace_posix_t *)self_;
  assert(self && "Invalid argument.");

  struct elf_symbol *sym = NULL;

  struct elf_syminfo_data **pp =
      (struct elf_syminfo_data **)(void *)&self->get_syminfo_user_data;
  while (1) {
    struct elf_syminfo_data *edata = ten_atomic_ptr_load((void *)pp);
    if (edata == NULL) {
      break;
    }

    sym = ((struct elf_symbol *)bsearch(&addr, edata->symbols, edata->count,
                                        sizeof(struct elf_symbol),
                                        elf_symbol_search));
    if (sym != NULL) {
      break;
    }

    pp = &edata->next;
  }

  if (sym == NULL) {
    callback(self_, addr, NULL, 0, 0, data);
  } else {
    callback(self_, addr, sym->name, sym->address, sym->size, data);
  }
}
