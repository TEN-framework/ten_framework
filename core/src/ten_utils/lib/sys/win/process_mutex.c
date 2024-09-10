//
// This file is part of the TEN Framework project.
// See https://github.com/TEN-framework/ten_framework/LICENSE for license
// information.
//
#include "ten_utils/lib/process_mutex.h"

#include <Windows.h>
#include <assert.h>
#include <stdlib.h>

#include "ten_utils/lib/string.h"

#define TEN_PROCESS_MUTEX_CREATE_MODE 0644

typedef struct ten_process_mutex_t {
  void *handle;
  ten_string_t *name;
} ten_process_mutex_t;

ten_process_mutex_t *ten_process_mutex_create(const char *name) {
  assert(name);

  ten_process_mutex_t *mutex = malloc(sizeof(ten_process_mutex_t));
  assert(mutex);

  mutex->handle = NULL;
  mutex->handle = CreateMutex(NULL, false, name);
  assert(mutex->handle);
  mutex->name = ten_string_create_formatted("%s", name);

  return mutex;
}

int ten_process_mutex_lock(ten_process_mutex_t *mutex) {
  assert(mutex);

  DWORD ret = WaitForSingleObject(mutex->handle, INFINITE);
  if (ret != WAIT_OBJECT_0) {
    return -1;
  }

  return 0;
}

int ten_process_mutex_unlock(ten_process_mutex_t *mutex) {
  assert(mutex);

  if (ReleaseMutex(mutex->handle)) {
    return 0;
  }

  return -1;
}

void ten_process_mutex_destroy(ten_process_mutex_t *mutex) {
  assert(mutex);

  CloseHandle(mutex->handle);
  ten_string_destroy(mutex->name);
  free(mutex);
}
