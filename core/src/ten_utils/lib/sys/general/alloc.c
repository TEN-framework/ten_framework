//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#include "ten_utils/lib/alloc.h"

#include <assert.h>
#include <stdlib.h>
#include <string.h>

#include "include_internal/ten_utils/lib/alloc.h"

void *ten_malloc(size_t size) {
  if (!size) {
    assert(0 && "malloc of size 0 is implementation defined behavior.");
    return NULL;
  }

  void *result = malloc(size);
  if (!result) {
    TEN_ASSERT(0, "Failed to allocate memory.");
  }

  return result;
}

void *ten_calloc(size_t cnt, size_t size) {
  if (!cnt || !size) {
    assert(0 && "calloc of size 0 is implementation defined behavior.");
    return NULL;
  }

  void *result = calloc(cnt, size);
  if (!result) {
    TEN_ASSERT(0, "Failed to allocate memory.");
  }

  return result;
}

void *ten_realloc(void *p, size_t size) {
  if (!size) {
    assert(0 && "realloc of size 0 is implementation defined behavior.");
    return NULL;
  }

  void *result = realloc(p, size);
  if (!result) {
    TEN_ASSERT(0, "Failed to allocate memory.");
  }

  return result;
}

void ten_free(void *p) {
  assert(p);
  free(p);
}

char *ten_strdup(const char *str) {
  if (!str) {
    assert(0 && "Invalid argument.");
    return NULL;
  }

  char *result = strdup(str);
  if (!result) {
    TEN_ASSERT(0, "Failed to allocate memory.");
  }

  return result;
}

char *ten_strndup(const char *str, size_t size) {
  if (!str) {
    assert(0 && "Invalid argument.");
    return NULL;
  }

  char *result = NULL;

#if defined(OS_WINDOWS)
  result = (char *)malloc(size + 1);
#else
  result = strndup(str, size);
#endif

  if (!result) {
    TEN_ASSERT(0, "Failed to allocate memory.");
    return NULL;
  }

#if defined(OS_WINDOWS)
  strncpy(result, str, size);
  result[size] = '\0';
#endif

  return result;
}
