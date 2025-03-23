//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#include "ten_utils/ten_config.h"

#include "include_internal/ten_utils/backtrace/platform/win/internal.h"

#include <assert.h>
#include <inttypes.h>
#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "include_internal/ten_utils/backtrace/backtrace.h"
#include "include_internal/ten_utils/backtrace/common.h"

/**
 * @brief Dynamically loads and retrieves function pointers for Windows
 * backtrace functionality.
 *
 * This function loads the necessary DLLs (DbgHelp.dll and NtDll.dll) and
 * retrieves function pointers for symbol handling and stack trace capture.
 * It asserts if any required function cannot be found.
 *
 * @param self Pointer to the Windows backtrace structure. Must not be NULL.
 */
static void retrieve_windows_backtrace_funcs(ten_backtrace_win_t *self) {
  assert(self && "Invalid argument.");

  HMODULE dll_handle = LoadLibraryA("DbgHelp.dll");
  if (dll_handle != NULL) {
    self->SymFromAddr =
        (win_SymFromAddr_func_t)GetProcAddress(dll_handle, "SymFromAddr");
    assert(self->SymFromAddr && "Failed to find DbgHelp.SymFromAddr");

    self->SymGetLineFromAddr = (win_SymGetLineFromAddr_func_t)GetProcAddress(
        dll_handle, "SymGetLineFromAddr64");
    assert(self->SymGetLineFromAddr &&
           "Failed to find DbgHelp.SymGetLineFromAddr");

    self->SymInitialize =
        (win_SymInitialize_func_t)GetProcAddress(dll_handle, "SymInitialize");
    assert(self->SymInitialize && "Failed to find DbgHelp.SymInitialize");

    self->SymCleanup =
        (win_SymCleanup_func_t)GetProcAddress(dll_handle, "SymCleanup");
    assert(self->SymCleanup && "Failed to find DbgHelp.SymCleanup");

    self->SymGetOptions =
        (win_SymGetOptions_func_t)GetProcAddress(dll_handle, "SymGetOptions");
    assert(self->SymGetOptions && "Failed to find DbgHelp.SymGetOptions");

    self->SymSetOptions =
        (win_SymSetOptions_func_t)GetProcAddress(dll_handle, "SymSetOptions");
    assert(self->SymSetOptions && "Failed to find DbgHelp.SymSetOptions");
  } else {
    assert(0 && "Failed to find DbgHelp.dll");
  }

  dll_handle = LoadLibraryA("NtDll.dll");
  if (dll_handle != NULL) {
    self->RtlCaptureStackBackTrace =
        (win_RtlCaptureStackBackTrace_func_t)GetProcAddress(
            dll_handle, "RtlCaptureStackBackTrace");
    assert(self->RtlCaptureStackBackTrace &&
           "Failed to find NtDll.RtlCaptureStackBackTrace");
  } else {
    assert(0 && "Failed to find NtDll.dll");
  }
}

/**
 * @brief Creates a new backtrace object for Windows platform.
 *
 * This function allocates memory for a `ten_backtrace_win_t` structure and
 * initializes its fields with default values. It sets up the common fields
 * with default callback functions for dumping stack traces and handling errors,
 * and loads the necessary Windows functions for backtrace functionality.
 *
 * @return A pointer to the newly created backtrace object cast to
 * `ten_backtrace_t`, or `NULL` if memory allocation fails.
 *
 * @note The returned object must be freed with `ten_backtrace_destroy()` when
 * no longer needed.
 */
ten_backtrace_t *ten_backtrace_create(void) {
  ten_backtrace_win_t *self = malloc(sizeof(ten_backtrace_win_t));
  if (!self) {
    assert(0 && "Failed to allocate memory.");

    // Return NULL if malloc fails.
    return NULL;
  }

  ten_backtrace_common_init(&self->common, ten_backtrace_default_dump,
                            ten_backtrace_default_error);
  retrieve_windows_backtrace_funcs(self);

  return (ten_backtrace_t *)self;
}

/**
 * @brief Destroys a backtrace object and frees associated resources.
 *
 * This function properly cleans up resources associated with the backtrace
 * object by first calling the common deinitialization function and then
 * freeing the memory allocated for the object itself.
 *
 * @param self Pointer to the backtrace object to destroy. Must not be NULL.
 */
void ten_backtrace_destroy(ten_backtrace_t *self) {
  assert(self && "Invalid argument.");

  ten_backtrace_common_deinit(self);
  free(self);
}

/**
 * @brief Dumps the current call stack.
 *
 * This function captures the current call stack using Windows-specific APIs
 * and processes it to retrieve symbol and line information. It then calls
 * the registered dump callback function for each frame in the stack trace.
 *
 * @param self Pointer to the backtrace object. Must not be NULL.
 * @param skip Number of stack frames to skip from the top of the call stack.
 *             This is useful to exclude the backtrace function itself and
 *             its immediate callers from the output.
 */
void ten_backtrace_dump(ten_backtrace_t *self, size_t skip) {
  if (!self) {
    assert(self && "Invalid argument.");
    return;
  }

  ten_backtrace_win_t *win_self = (ten_backtrace_win_t *)self;

  if (win_self->SymInitialize == NULL || win_self->SymCleanup == NULL ||
      win_self->SymGetOptions == NULL || win_self->SymSetOptions == NULL ||
      win_self->SymFromAddr == NULL || win_self->SymGetLineFromAddr == NULL ||
      win_self->RtlCaptureStackBackTrace == NULL) {
    fprintf(stderr, "Failed to retrieve Windows backtrace functions.\n");
    return;
  }

  HANDLE process = GetCurrentProcess();

  // Configure symbol handler options.
  win_self->SymSetOptions(win_self->SymGetOptions() | SYMOPT_LOAD_LINES |
                          SYMOPT_DEFERRED_LOADS);

  // Initialize symbol handler
  if (!win_self->SymInitialize(process, NULL, TRUE)) {
    fprintf(stderr, "Failed to initialize symbol handler: %lu\n",
            GetLastError());
    return;
  }

  // Capture the stack trace.
  void *stack[MAX_CAPTURED_CALL_STACK_DEPTH] = {0};

  USHORT frames = win_self->RtlCaptureStackBackTrace(
      0, MAX_CAPTURED_CALL_STACK_DEPTH, stack, NULL);

  if (skip >= frames) {
    fprintf(stderr, "Skip count (%zu) exceeds available frames (%hu).\n", skip,
            frames);
    win_self->SymCleanup(process);
    return;
  }

  // Allocate memory for symbol information.
  SYMBOL_INFO *symbol = (SYMBOL_INFO *)calloc(
      offsetof(SYMBOL_INFO, Name) + 256 * sizeof(symbol->Name[0]), 1);
  if (!symbol) {
    fprintf(stderr, "Failed to allocate memory for symbol information.\n");
    win_self->SymCleanup(process);
    return;
  }

  symbol->MaxNameLen = 255;
  symbol->SizeOfStruct = sizeof(SYMBOL_INFO);

  // Process each frame in the stack trace.
  for (size_t i = skip; i < frames; i++) {
    DWORD64 address = (DWORD64)(stack[i]);

    // Get symbol information.
    if (!win_self->SymFromAddr(process, address, 0, symbol)) {
      fprintf(stderr, "Failed to get symbol for address 0x%llx: %lu\n",
              (unsigned long long)address, GetLastError());
      continue;
    }

    // Get line information.
    IMAGEHLP_LINE lineInfo = {0};
    lineInfo.SizeOfStruct = sizeof(IMAGEHLP_LINE);
    DWORD dwLineDisplacement = 0;

    if (win_self->SymGetLineFromAddr(process, address, &dwLineDisplacement,
                                     &lineInfo)) {
      // Call the dump callback with full information.
      win_self->common.on_dump_file_line(self, symbol->Address,
                                         lineInfo.FileName, lineInfo.LineNumber,
                                         symbol->Name, NULL);
    } else {
      // Call the dump callback with only symbol information.
      win_self->common.on_dump_file_line(self, symbol->Address, NULL, 0,
                                         symbol->Name, NULL);
    }
  }

  // Clean up.
  free(symbol);
  win_self->SymCleanup(process);
}
