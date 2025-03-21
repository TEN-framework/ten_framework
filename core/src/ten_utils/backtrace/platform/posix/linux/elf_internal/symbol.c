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
                           ten_backtrace_on_error_func_t on_error, void *data,
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
      on_error(self, "symbol string index out of range", 0, data);
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
    elf_syminfo_data **pp = (elf_syminfo_data **)&self->on_get_syminfo_data;

    while (1) {
      elf_syminfo_data *p = ten_atomic_ptr_load((ten_atomic_ptr_t *)pp);
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

/**
 * @brief Compare an address against an ELF symbol for binary search.
 *
 * This function is used as a comparison callback for `bsearch()` when looking
 * up symbols by address. It determines if the given address falls within the
 * memory range occupied by a symbol (address >= symbol.address &&
 * address < symbol.address + symbol.size).
 *
 * @param vkey Pointer to the address to look up (cast to void*).
 * @param ventry Pointer to the `elf_symbol` structure to compare against (cast
 * to void*).
 * @return -1 if address is less than symbol's address range,
 *          0 if address falls within the symbol's address range,
 *          1 if address is greater than symbol's address range.
 *
 * @note We allocate one extra entry in the symbols array with a very high
 * address to ensure this function can safely check address ranges even for the
 * last real symbol in the array.
 */
static int elf_symbol_search(const void *vkey, const void *ventry) {
  const uintptr_t *key = (const uintptr_t *)vkey;
  const elf_symbol *entry = (const elf_symbol *)ventry;
  uintptr_t addr = *key;

  // If address is before the symbol's start address.
  if (addr < entry->address) {
    return -1;
  }
  // If address is at or after the symbol's end address (address + size).
  // Note: For symbols with size 0, we'll only match exact address matches.
  else if (addr >= entry->address + entry->size) {
    return 1;
  }
  // Address is within the symbol's range.
  else {
    return 0;
  }
}

/**
 * @brief Look up symbol information for a given address.
 *
 * This function searches through all available symbol tables to find
 * information about the symbol at the specified address. It traverses a linked
 * list of symbol data structures, each containing an array of symbols sorted by
 * address. When a matching symbol is found, it calls the provided callback with
 * the symbol's name, address, and size.
 *
 * @param self_ The backtrace context.
 * @param addr The address to look up.
 * @param on_dump_syminfo Function to call with the symbol information if found.
 * @param on_error Function to call to report errors.
 * @param data User data to pass to the callback.
 */
void elf_syminfo(ten_backtrace_t *self, uintptr_t addr,
                 ten_backtrace_on_dump_syminfo_func_t on_dump_syminfo,
                 TEN_UNUSED ten_backtrace_on_error_func_t on_error,
                 void *data) {
  ten_backtrace_posix_t *self_posix = (ten_backtrace_posix_t *)self;
  assert(self_posix && "Invalid argument.");

  elf_symbol *sym = NULL;

  // Get the head of the linked list of symbol data structures.
  elf_syminfo_data **pp = (elf_syminfo_data **)&self_posix->on_get_syminfo_data;

  // Traverse the linked list of symbol data structures.
  while (1) {
    elf_syminfo_data *edata = ten_atomic_ptr_load((ten_atomic_ptr_t *)pp);
    if (edata == NULL) {
      break; // End of the list.
    }

    // Binary search for the symbol containing the address.
    sym = ((elf_symbol *)bsearch(&addr, edata->symbols, edata->count,
                                 sizeof(elf_symbol), elf_symbol_search));
    if (sym != NULL) {
      break; // Found a matching symbol.
    }

    // Move to the next symbol data structure.
    pp = &edata->next;
  }

  // Call the callback with the symbol information or NULL if not found.
  if (sym == NULL) {
    on_dump_syminfo(self, addr, NULL, 0, 0, data);
  } else {
    on_dump_syminfo(self, addr, sym->name, sym->address, sym->size, data);
  }
}
