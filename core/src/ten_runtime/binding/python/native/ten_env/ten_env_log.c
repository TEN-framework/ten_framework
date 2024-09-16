//
// This file is part of the TEN Framework project.
// See https://github.com/TEN-framework/ten_framework/LICENSE for license
// information.
//
#include <string.h>

#include "include_internal/ten_runtime/binding/python/common/error.h"
#include "include_internal/ten_runtime/binding/python/ten_env/ten_env.h"
#include "include_internal/ten_runtime/ten_env/log.h"
#include "include_internal/ten_utils/log/new.h"
#include "ten_utils/macro/check.h"
#include "ten_utils/macro/memory.h"

typedef struct ten_env_notify_log_info_t {
  int32_t level;
  const char *func_name;
  const char *file_name;
  size_t line_no;
  const char *msg;
  ten_event_t *completed;
} ten_env_notify_log_info_t;

static ten_env_notify_log_info_t *ten_env_notify_log_info_create(
    int32_t level, const char *func_name, const char *file_name, size_t line_no,
    const char *msg) {
  ten_env_notify_log_info_t *info =
      TEN_MALLOC(sizeof(ten_env_notify_log_info_t));
  TEN_ASSERT(info, "Failed to allocate memory.");

  info->level = level;
  info->func_name = func_name;
  info->file_name = file_name;
  info->line_no = line_no;
  info->msg = msg;
  info->completed = ten_event_create(0, 1);

  return info;
}

static void ten_env_notify_log_info_destroy(ten_env_notify_log_info_t *info) {
  TEN_ASSERT(info, "Invalid argument.");

  ten_event_destroy(info->completed);

  TEN_FREE(info);
}

static void ten_env_notify_log(ten_env_t *ten_env, void *user_data) {
  TEN_ASSERT(user_data, "Invalid argument.");
  TEN_ASSERT(ten_env && ten_env_check_integrity(ten_env, true),
             "Should not happen.");

  ten_env_notify_log_info_t *info = user_data;
  TEN_ASSERT(info, "Should not happen.");

  ten_env_log(ten_env, info->level, info->func_name, info->file_name,
              info->line_no, info->msg);

  ten_event_set(info->completed);
}

PyObject *ten_py_ten_env_log(PyObject *self, PyObject *args) {
  ten_py_ten_env_t *py_ten = (ten_py_ten_env_t *)self;
  TEN_ASSERT(py_ten && ten_py_ten_env_check_integrity(py_ten),
             "Invalid argument.");

  if (PyTuple_GET_SIZE(args) != 5) {
    return ten_py_raise_py_value_error_exception(
        "Invalid argument count when ten_env.log.");
  }

  TEN_LOG_NEW_LEVEL level = TEN_LOG_NEW_INVALID;
  const char *func_name = NULL;
  const char *file_name = NULL;
  size_t line_no = 0;
  const char *msg = NULL;
  if (!PyArg_ParseTuple(args, "issis", &level, &func_name, &file_name, &line_no,
                        &msg)) {
    return ten_py_raise_py_value_error_exception(
        "Failed to parse argument when ten_env.log.");
  }

  ten_error_t err;
  ten_error_init(&err);

  ten_env_notify_log_info_t *info =
      ten_env_notify_log_info_create(level, func_name, file_name, line_no, msg);

  if (!ten_env_proxy_notify(py_ten->c_ten_env_proxy, ten_env_notify_log, info,
                            false, &err)) {
    goto done;
  }

  PyThreadState *saved_py_thread_state = PyEval_SaveThread();
  ten_event_wait(info->completed, -1);
  PyEval_RestoreThread(saved_py_thread_state);

done:
  ten_error_deinit(&err);
  ten_env_notify_log_info_destroy(info);

  Py_RETURN_NONE;
}
