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
#include "include_internal/ten_utils/backtrace/vector.h"
#include "ten_utils/lib/atomic.h"

#define IS_DIR_SEPARATOR(c) ((c) == '/')
#define IS_ABSOLUTE_PATH(f) (IS_DIR_SEPARATOR((f)[0]))

// DWARF constants.

/**
 * @brief An enum for the DWARF sections we care about.
 */
typedef enum dwarf_section {
  DEBUG_INFO,
  DEBUG_LINE,
  DEBUG_ABBREV,
  DEBUG_RANGES,
  DEBUG_STR,
  DEBUG_ADDR,
  DEBUG_STR_OFFSETS,
  DEBUG_LINE_STR,
  DEBUG_RNGLISTS,

  DEBUG_MAX
} dwarf_section;

typedef enum dwarf_tag {
  DW_TAG_entry_point = 0x3,
  DW_TAG_compile_unit = 0x11,
  DW_TAG_inlined_subroutine = 0x1d,
  DW_TAG_subprogram = 0x2e,
  DW_TAG_skeleton_unit = 0x4a,
} dwarf_tag;

typedef enum dwarf_form {
  DW_FORM_addr = 0x01,
  DW_FORM_block2 = 0x03,
  DW_FORM_block4 = 0x04,
  DW_FORM_data2 = 0x05,
  DW_FORM_data4 = 0x06,
  DW_FORM_data8 = 0x07,
  DW_FORM_string = 0x08,
  DW_FORM_block = 0x09,
  DW_FORM_block1 = 0x0a,
  DW_FORM_data1 = 0x0b,
  DW_FORM_flag = 0x0c,
  DW_FORM_sdata = 0x0d,
  DW_FORM_strp = 0x0e,
  DW_FORM_udata = 0x0f,
  DW_FORM_ref_addr = 0x10,
  DW_FORM_ref1 = 0x11,
  DW_FORM_ref2 = 0x12,
  DW_FORM_ref4 = 0x13,
  DW_FORM_ref8 = 0x14,
  DW_FORM_ref_udata = 0x15,
  DW_FORM_indirect = 0x16,
  DW_FORM_sec_offset = 0x17,
  DW_FORM_exprloc = 0x18,
  DW_FORM_flag_present = 0x19,
  DW_FORM_ref_sig8 = 0x20,
  DW_FORM_strx = 0x1a,
  DW_FORM_addrx = 0x1b,
  DW_FORM_ref_sup4 = 0x1c,
  DW_FORM_strp_sup = 0x1d,
  DW_FORM_data16 = 0x1e,
  DW_FORM_line_strp = 0x1f,
  DW_FORM_implicit_const = 0x21,
  DW_FORM_loclistx = 0x22,
  DW_FORM_rnglistx = 0x23,
  DW_FORM_ref_sup8 = 0x24,
  DW_FORM_strx1 = 0x25,
  DW_FORM_strx2 = 0x26,
  DW_FORM_strx3 = 0x27,
  DW_FORM_strx4 = 0x28,
  DW_FORM_addrx1 = 0x29,
  DW_FORM_addrx2 = 0x2a,
  DW_FORM_addrx3 = 0x2b,
  DW_FORM_addrx4 = 0x2c,
  DW_FORM_GNU_addr_index = 0x1f01,
  DW_FORM_GNU_str_index = 0x1f02,
  DW_FORM_GNU_ref_alt = 0x1f20,
  DW_FORM_GNU_strp_alt = 0x1f21
} dwarf_form;

typedef enum dwarf_attribute {
  DW_AT_sibling = 0x01,
  DW_AT_location = 0x02,
  DW_AT_name = 0x03,
  DW_AT_ordering = 0x09,
  DW_AT_subscr_data = 0x0a,
  DW_AT_byte_size = 0x0b,
  DW_AT_bit_offset = 0x0c,
  DW_AT_bit_size = 0x0d,
  DW_AT_element_list = 0x0f,
  DW_AT_stmt_list = 0x10,
  DW_AT_low_pc = 0x11,
  DW_AT_high_pc = 0x12,
  DW_AT_language = 0x13,
  DW_AT_member = 0x14,
  DW_AT_discr = 0x15,
  DW_AT_discr_value = 0x16,
  DW_AT_visibility = 0x17,
  DW_AT_import = 0x18,
  DW_AT_string_length = 0x19,
  DW_AT_common_reference = 0x1a,
  DW_AT_comp_dir = 0x1b,
  DW_AT_const_value = 0x1c,
  DW_AT_containing_type = 0x1d,
  DW_AT_default_value = 0x1e,
  DW_AT_inline = 0x20,
  DW_AT_is_optional = 0x21,
  DW_AT_lower_bound = 0x22,
  DW_AT_producer = 0x25,
  DW_AT_prototyped = 0x27,
  DW_AT_return_addr = 0x2a,
  DW_AT_start_scope = 0x2c,
  DW_AT_bit_stride = 0x2e,
  DW_AT_upper_bound = 0x2f,
  DW_AT_abstract_origin = 0x31,
  DW_AT_accessibility = 0x32,
  DW_AT_address_class = 0x33,
  DW_AT_artificial = 0x34,
  DW_AT_base_types = 0x35,
  DW_AT_calling_convention = 0x36,
  DW_AT_count = 0x37,
  DW_AT_data_member_location = 0x38,
  DW_AT_decl_column = 0x39,
  DW_AT_decl_file = 0x3a,
  DW_AT_decl_line = 0x3b,
  DW_AT_declaration = 0x3c,
  DW_AT_discr_list = 0x3d,
  DW_AT_encoding = 0x3e,
  DW_AT_external = 0x3f,
  DW_AT_frame_base = 0x40,
  DW_AT_friend = 0x41,
  DW_AT_identifier_case = 0x42,
  DW_AT_macro_info = 0x43,
  DW_AT_namelist_items = 0x44,
  DW_AT_priority = 0x45,
  DW_AT_segment = 0x46,
  DW_AT_specification = 0x47,
  DW_AT_static_link = 0x48,
  DW_AT_type = 0x49,
  DW_AT_use_location = 0x4a,
  DW_AT_variable_parameter = 0x4b,
  DW_AT_virtuality = 0x4c,
  DW_AT_vtable_elem_location = 0x4d,
  DW_AT_allocated = 0x4e,
  DW_AT_associated = 0x4f,
  DW_AT_data_location = 0x50,
  DW_AT_byte_stride = 0x51,
  DW_AT_entry_pc = 0x52,
  DW_AT_use_UTF8 = 0x53,
  DW_AT_extension = 0x54,
  DW_AT_ranges = 0x55,
  DW_AT_trampoline = 0x56,
  DW_AT_call_column = 0x57,
  DW_AT_call_file = 0x58,
  DW_AT_call_line = 0x59,
  DW_AT_description = 0x5a,
  DW_AT_binary_scale = 0x5b,
  DW_AT_decimal_scale = 0x5c,
  DW_AT_small = 0x5d,
  DW_AT_decimal_sign = 0x5e,
  DW_AT_digit_count = 0x5f,
  DW_AT_picture_string = 0x60,
  DW_AT_mutable = 0x61,
  DW_AT_threads_scaled = 0x62,
  DW_AT_explicit = 0x63,
  DW_AT_object_pointer = 0x64,
  DW_AT_endianity = 0x65,
  DW_AT_elemental = 0x66,
  DW_AT_pure = 0x67,
  DW_AT_recursive = 0x68,
  DW_AT_signature = 0x69,
  DW_AT_main_subprogram = 0x6a,
  DW_AT_data_bit_offset = 0x6b,
  DW_AT_const_expr = 0x6c,
  DW_AT_enum_class = 0x6d,
  DW_AT_linkage_name = 0x6e,
  DW_AT_string_length_bit_size = 0x6f,
  DW_AT_string_length_byte_size = 0x70,
  DW_AT_rank = 0x71,
  DW_AT_str_offsets_base = 0x72,
  DW_AT_addr_base = 0x73,
  DW_AT_rnglists_base = 0x74,
  DW_AT_dwo_name = 0x76,
  DW_AT_reference = 0x77,
  DW_AT_rvalue_reference = 0x78,
  DW_AT_macros = 0x79,
  DW_AT_call_all_calls = 0x7a,
  DW_AT_call_all_source_calls = 0x7b,
  DW_AT_call_all_tail_calls = 0x7c,
  DW_AT_call_return_pc = 0x7d,
  DW_AT_call_value = 0x7e,
  DW_AT_call_origin = 0x7f,
  DW_AT_call_parameter = 0x80,
  DW_AT_call_pc = 0x81,
  DW_AT_call_tail_call = 0x82,
  DW_AT_call_target = 0x83,
  DW_AT_call_target_clobbered = 0x84,
  DW_AT_call_data_location = 0x85,
  DW_AT_call_data_value = 0x86,
  DW_AT_noreturn = 0x87,
  DW_AT_alignment = 0x88,
  DW_AT_export_symbols = 0x89,
  DW_AT_deleted = 0x8a,
  DW_AT_defaulted = 0x8b,
  DW_AT_loclists_base = 0x8c,
  DW_AT_lo_user = 0x2000,
  DW_AT_hi_user = 0x3fff,
  DW_AT_MIPS_fde = 0x2001,
  DW_AT_MIPS_loop_begin = 0x2002,
  DW_AT_MIPS_tail_loop_begin = 0x2003,
  DW_AT_MIPS_epilog_begin = 0x2004,
  DW_AT_MIPS_loop_unroll_factor = 0x2005,
  DW_AT_MIPS_software_pipeline_depth = 0x2006,
  DW_AT_MIPS_linkage_name = 0x2007,
  DW_AT_MIPS_stride = 0x2008,
  DW_AT_MIPS_abstract_name = 0x2009,
  DW_AT_MIPS_clone_origin = 0x200a,
  DW_AT_MIPS_has_inlines = 0x200b,
  DW_AT_HP_block_index = 0x2000,
  DW_AT_HP_unmodifiable = 0x2001,
  DW_AT_HP_prologue = 0x2005,
  DW_AT_HP_epilogue = 0x2008,
  DW_AT_HP_actuals_stmt_list = 0x2010,
  DW_AT_HP_proc_per_section = 0x2011,
  DW_AT_HP_raw_data_ptr = 0x2012,
  DW_AT_HP_pass_by_reference = 0x2013,
  DW_AT_HP_opt_level = 0x2014,
  DW_AT_HP_prof_version_id = 0x2015,
  DW_AT_HP_opt_flags = 0x2016,
  DW_AT_HP_cold_region_low_pc = 0x2017,
  DW_AT_HP_cold_region_high_pc = 0x2018,
  DW_AT_HP_all_variables_modifiable = 0x2019,
  DW_AT_HP_linkage_name = 0x201a,
  DW_AT_HP_prof_flags = 0x201b,
  DW_AT_HP_unit_name = 0x201f,
  DW_AT_HP_unit_size = 0x2020,
  DW_AT_HP_widened_byte_size = 0x2021,
  DW_AT_HP_definition_points = 0x2022,
  DW_AT_HP_default_location = 0x2023,
  DW_AT_HP_is_result_param = 0x2029,
  DW_AT_sf_names = 0x2101,
  DW_AT_src_info = 0x2102,
  DW_AT_mac_info = 0x2103,
  DW_AT_src_coords = 0x2104,
  DW_AT_body_begin = 0x2105,
  DW_AT_body_end = 0x2106,
  DW_AT_GNU_vector = 0x2107,
  DW_AT_GNU_guarded_by = 0x2108,
  DW_AT_GNU_pt_guarded_by = 0x2109,
  DW_AT_GNU_guarded = 0x210a,
  DW_AT_GNU_pt_guarded = 0x210b,
  DW_AT_GNU_locks_excluded = 0x210c,
  DW_AT_GNU_exclusive_locks_required = 0x210d,
  DW_AT_GNU_shared_locks_required = 0x210e,
  DW_AT_GNU_odr_signature = 0x210f,
  DW_AT_GNU_template_name = 0x2110,
  DW_AT_GNU_call_site_value = 0x2111,
  DW_AT_GNU_call_site_data_value = 0x2112,
  DW_AT_GNU_call_site_target = 0x2113,
  DW_AT_GNU_call_site_target_clobbered = 0x2114,
  DW_AT_GNU_tail_call = 0x2115,
  DW_AT_GNU_all_tail_call_sites = 0x2116,
  DW_AT_GNU_all_call_sites = 0x2117,
  DW_AT_GNU_all_source_call_sites = 0x2118,
  DW_AT_GNU_macros = 0x2119,
  DW_AT_GNU_deleted = 0x211a,
  DW_AT_GNU_dwo_name = 0x2130,
  DW_AT_GNU_dwo_id = 0x2131,
  DW_AT_GNU_ranges_base = 0x2132,
  DW_AT_GNU_addr_base = 0x2133,
  DW_AT_GNU_pubnames = 0x2134,
  DW_AT_GNU_pubtypes = 0x2135,
  DW_AT_GNU_discriminator = 0x2136,
  DW_AT_GNU_locviews = 0x2137,
  DW_AT_GNU_entry_view = 0x2138,
  DW_AT_VMS_rtnbeg_pd_address = 0x2201,
  DW_AT_use_GNAT_descriptive_type = 0x2301,
  DW_AT_GNAT_descriptive_type = 0x2302,
  DW_AT_GNU_numerator = 0x2303,
  DW_AT_GNU_denominator = 0x2304,
  DW_AT_GNU_bias = 0x2305,
  DW_AT_upc_threads_scaled = 0x3210,
  DW_AT_PGI_lbase = 0x3a00,
  DW_AT_PGI_soffset = 0x3a01,
  DW_AT_PGI_lstride = 0x3a02,
  DW_AT_APPLE_optimized = 0x3fe1,
  DW_AT_APPLE_flags = 0x3fe2,
  DW_AT_APPLE_isa = 0x3fe3,
  DW_AT_APPLE_block = 0x3fe4,
  DW_AT_APPLE_major_runtime_vers = 0x3fe5,
  DW_AT_APPLE_runtime_class = 0x3fe6,
  DW_AT_APPLE_omit_frame_ptr = 0x3fe7,
  DW_AT_APPLE_property_name = 0x3fe8,
  DW_AT_APPLE_property_getter = 0x3fe9,
  DW_AT_APPLE_property_setter = 0x3fea,
  DW_AT_APPLE_property_attribute = 0x3feb,
  DW_AT_APPLE_objc_complete_type = 0x3fec,
  DW_AT_APPLE_property = 0x3fed
} dwarf_attribute;

typedef enum dwarf_line_number_op {
  DW_LNS_extended_op = 0x0,
  DW_LNS_copy = 0x1,
  DW_LNS_advance_pc = 0x2,
  DW_LNS_advance_line = 0x3,
  DW_LNS_set_file = 0x4,
  DW_LNS_set_column = 0x5,
  DW_LNS_negate_stmt = 0x6,
  DW_LNS_set_basic_block = 0x7,
  DW_LNS_const_add_pc = 0x8,
  DW_LNS_fixed_advance_pc = 0x9,
  DW_LNS_set_prologue_end = 0xa,
  DW_LNS_set_epilogue_begin = 0xb,
  DW_LNS_set_isa = 0xc,
} dwarf_line_number_op;

typedef enum dwarf_extended_line_number_op {
  DW_LNE_end_sequence = 0x1,
  DW_LNE_set_address = 0x2,
  DW_LNE_define_file = 0x3,
  DW_LNE_set_discriminator = 0x4,
} dwarf_extended_line_number_op;

typedef enum dwarf_line_number_content_type {
  DW_LNCT_path = 0x1,
  DW_LNCT_directory_index = 0x2,
  DW_LNCT_timestamp = 0x3,
  DW_LNCT_size = 0x4,
  DW_LNCT_MD5 = 0x5,
  DW_LNCT_lo_user = 0x2000,
  DW_LNCT_hi_user = 0x3fff
} dwarf_line_number_content_type;

typedef enum dwarf_range_list_entry {
  DW_RLE_end_of_list = 0x00,
  DW_RLE_base_addressx = 0x01,
  DW_RLE_startx_endx = 0x02,
  DW_RLE_startx_length = 0x03,
  DW_RLE_offset_pair = 0x04,
  DW_RLE_base_address = 0x05,
  DW_RLE_start_end = 0x06,
  DW_RLE_start_length = 0x07
} dwarf_range_list_entry;

typedef enum dwarf_unit_type {
  DW_UT_compile = 0x01,
  DW_UT_type = 0x02,
  DW_UT_partial = 0x03,
  DW_UT_skeleton = 0x04,
  DW_UT_split_compile = 0x05,
  DW_UT_split_type = 0x06,
  DW_UT_lo_user = 0x80,
  DW_UT_hi_user = 0xff
} dwarf_unit_type;

// A buffer to read DWARF info.
typedef struct dwarf_buf {
  // Buffer name for error messages.
  const char *name;

  // Start of the buffer.
  const unsigned char *start;

  // Next byte to read.
  const unsigned char *buf;

  // The number of bytes remaining.
  size_t left;

  // Whether the data is big-endian.
  int is_bigendian;

  // Error callback routine.
  ten_backtrace_error_func_t error_cb;

  // Data for error_cb.
  void *data;

  // Non-zero if we've reported an underflow error.
  int reported_underflow;
} dwarf_buf;

// A single attribute in a DWARF abbreviation.
typedef struct attr {
  // The attribute name.
  enum dwarf_attribute name;

  // The attribute form.
  enum dwarf_form form;

  // The attribute value, for DW_FORM_implicit_const.
  int64_t val;
} attr;

// A single DWARF abbreviation.
typedef struct abbrev {
  // The abbrev code--the number used to refer to the abbrev.
  uint64_t code;

  // The entry tag.
  enum dwarf_tag tag;

  // Non-zero if this abbrev has child entries.
  int has_children;

  // The number of attributes.
  size_t num_attrs;

  // The attributes.
  struct attr *attrs;
} abbrev;

// The DWARF abbreviations for a compilation unit. This structure only exists
// while reading the compilation unit. Most DWARF readers seem to a hash table
// to map abbrev ID's to abbrev entries. However, we primarily care about GCC,
// and GCC simply issues ID's in numerical order starting at 1. So we simply
// keep a sorted vector, and try to just look up the code.
typedef struct abbrevs {
  // The number of abbrevs in the vector.
  size_t num_abbrevs;

  // The abbrevs, sorted by the code field.
  struct abbrev *abbrevs;
} abbrevs;

// The different kinds of attribute values.
typedef enum attr_val_encoding {
  /* No attribute value.  */
  ATTR_VAL_NONE,
  /* An address.  */
  ATTR_VAL_ADDRESS,
  /* An index into the .debug_addr section, whose value is relative to
     the DW_AT_addr_base attribute of the compilation unit.  */
  ATTR_VAL_ADDRESS_INDEX,
  /* A unsigned integer.  */
  ATTR_VAL_UINT,
  /* A sigd integer.  */
  ATTR_VAL_SINT,
  /* A string.  */
  ATTR_VAL_STRING,
  /* An index into the .debug_str_offsets section.  */
  ATTR_VAL_STRING_INDEX,
  /* An offset to other data in the containing unit.  */
  ATTR_VAL_REF_UNIT,
  /* An offset to other data within the .debug_info section.  */
  ATTR_VAL_REF_INFO,
  /* An offset to other data within the alt .debug_info section.  */
  ATTR_VAL_REF_ALT_INFO,
  /* An offset to data in some other section.  */
  ATTR_VAL_REF_SECTION,
  /* A type signature.  */
  ATTR_VAL_REF_TYPE,
  /* An index into the .debug_rnglists section.  */
  ATTR_VAL_RNGLISTS_INDEX,
  /* A block of data (not represented).  */
  ATTR_VAL_BLOCK,
  /* An expression (not represented).  */
  ATTR_VAL_EXPR,
} attr_val_encoding;

// An attribute value.
typedef struct attr_val {
  /* How the value is stored in the field u.  */
  enum attr_val_encoding encoding;
  union {
    /* ATTR_VAL_ADDRESS*, ATTR_VAL_UINT, ATTR_VAL_REF*.  */
    uint64_t uint;
    /* ATTR_VAL_SINT.  */
    int64_t sint;
    /* ATTR_VAL_STRING.  */
    const char *string;
    /* ATTR_VAL_BLOCK not stored.  */
  } u;
} attr_val;

// The line number program header.
typedef struct line_header {
  /* The version of the line number information.  */
  int version;
  /* Address size.  */
  int addrsize;
  /* The minimum instruction length.  */
  unsigned int min_insn_len;
  /* The maximum number of ops per instruction.  */
  unsigned int max_ops_per_insn;
  /* The line base for special opcodes.  */
  int line_base;
  /* The line range for special opcodes.  */
  unsigned int line_range;
  /* The opcode base--the first special opcode.  */
  unsigned int opcode_base;
  /* Opcode lengths, indexed by opcode - 1.  */
  const unsigned char *opcode_lengths;
  /* The number of directory entries.  */
  size_t dirs_count;
  /* The directory entries.  */
  const char **dirs;
  /* The number of filenames.  */
  size_t filenames_count;
  /* The filenames.  */
  const char **filenames;
} line_header;

// A format description from a line header.
typedef struct line_header_format {
  int lnct;             /* LNCT code.  */
  enum dwarf_form form; /* Form of entry data.  */
} line_header_format;

/**
 * @brief Map a single PC value to a file/line. We will keep a vector of these
 * sorted by PC value. Each file/line will be correct from the PC up to the PC
 * of the next entry if there is one. We allocate one extra entry at the end so
 * that we can use bsearch.
 */
typedef struct line {
  /* PC.  */
  uintptr_t pc;
  /* File name.  Many entries in the array are expected to point to
     the same file name.  */
  const char *filename;
  /* Line number.  */
  int lineno;
  /* Index of the object in the original array read from the DWARF
     section, before it has been sorted.  The index makes it possible
     to use Quicksort and maintain stability.  */
  int idx;
} line;

// A growable vector of line number information. This is used while reading the
// line numbers.
typedef struct line_vector {
  /* Memory.  This is an array of struct line.  */
  ten_vector_t vec;
  /* Number of valid mappings.  */
  size_t count;
} line_vector;

// A function described in the debug info.
typedef struct function {
  /* The name of the function.  */
  const char *name;
  /* If this is an inlined function, the filename of the call
     site.  */
  const char *caller_filename;
  /* If this is an inlined function, the line number of the call
     site.  */
  int caller_lineno;
  /* Map PC ranges to inlined functions.  */
  struct function_addrs *function_addrs;
  size_t function_addrs_count;
} function;

/**
 * @brief An address range for a function. This maps a PC value to a specific
 * function.
 */
typedef struct function_addrs {
  /* Range is LOW <= PC < HIGH.  */
  uintptr_t low;
  uintptr_t high;
  /* Function for this address range.  */
  struct function *function;
} function_addrs;

// A growable vector of function address ranges.
typedef struct function_vector {
  /* Memory.  This is an array of struct function_addrs.  */
  ten_vector_t vec;
  /* Number of address ranges present.  */
  size_t count;
} function_vector;

/**
 * @brief A DWARF compilation unit. This only holds the information we need to
 * map a PC to a file and line.
 */
typedef struct unit {
  /* The first entry for this compilation unit.  */
  const unsigned char *unit_data;
  /* The length of the data for this compilation unit.  */
  size_t unit_data_len;
  /* The offset of UNIT_DATA from the start of the information for
     this compilation unit.  */
  size_t unit_data_offset;
  /* Offset of the start of the compilation unit from the start of the
     .debug_info section.  */
  size_t low_offset;
  /* Offset of the end of the compilation unit from the start of the
     .debug_info section.  */
  size_t high_offset;
  /* DWARF version.  */
  int version;
  /* Whether unit is DWARF64.  */
  int is_dwarf64;
  /* Address size.  */
  int addrsize;
  /* Offset into line number information.  */
  off_t lineoff;
  /* Offset of compilation unit in .debug_str_offsets.  */
  uint64_t str_offsets_base;
  /* Offset of compilation unit in .debug_addr.  */
  uint64_t addr_base;
  /* Offset of compilation unit in .debug_rnglists.  */
  uint64_t rnglists_base;
  /* Primary source file.  */
  const char *filename;
  /* Compilation command working directory.  */
  const char *comp_dir;
  /* Absolute file name, only set if needed.  */
  const char *abs_filename;
  /* The abbreviations for this unit.  */
  struct abbrevs abbrevs;

  /* The fields above this point are read in during initialization and
     may be accessed freely.  The fields below this point are read in
     as needed, and therefore require care, as different threads may
     try to initialize them simultaneously.  */

  /* PC to line number mapping.  This is NULL if the values have not
     been read.  This is (struct line *) -1 if there was an error
     reading the values.  */
  struct line *lines;
  ten_atomic_t lines_count;  // Number of entries in lines.

  // PC ranges to function.
  struct function_addrs *function_addrs;
  ten_atomic_t function_addrs_count;
} unit;

/**
 * @brief An address range for a compilation unit. This maps a PC value to a
 * specific compilation unit.
 *
 * @note We invert the representation in DWARF: instead of listing the units and
 * attaching a list of ranges, we list the ranges and have each one point to the
 * unit. This lets us do a binary search to find the unit.
 */
typedef struct unit_addrs {
  /* Range is LOW <= PC < HIGH.  */
  uintptr_t low;
  uintptr_t high;
  /* Compilation unit for this address range.  */
  struct unit *u;
} unit_addrs;

// A growable vector of compilation unit address ranges.
typedef struct unit_addrs_vector {
  /* Memory.  This is an array of struct unit_addrs.  */
  ten_vector_t vec;
  /* Number of address ranges present.  */
  size_t count;
} unit_addrs_vector;

// A growable vector of compilation unit pointer.
typedef struct unit_vector {
  ten_vector_t vec;
  size_t count;
} unit_vector;

// Data for the DWARF sections we care about.
typedef struct dwarf_sections {
  const unsigned char *data[DEBUG_MAX];
  size_t size[DEBUG_MAX];
} dwarf_sections;

// The information we need to map a PC to a file and line.
typedef struct dwarf_data {
  /* The data for the next file we know about.  */
  struct dwarf_data *next;
  /* The data for .gnu_debugaltlink.  */
  struct dwarf_data *altlink;
  /* The base address for this file.  */
  uintptr_t base_address;
  /* A sorted list of address ranges.  */
  struct unit_addrs *addrs;
  /* Number of address ranges in list.  */
  size_t addrs_count;
  /* A sorted list of units.  */
  struct unit **units;
  /* Number of units in the list.  */
  size_t units_count;
  /* The unparsed DWARF debug data.  */
  struct dwarf_sections dwarf_sections;
  /* Whether the data is big-endian or not.  */
  int is_bigendian;
  /* A vector used for function addresses.  We keep this here so that
     we can grow the vector as we read more functions.  */
  struct function_vector fvec;
} dwarf_data;
