//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#include <string.h>

#include "include_internal/ten_runtime/binding/python/common/error.h"
#include "include_internal/ten_runtime/binding/python/ten_env/ten_env.h"
#include "include_internal/ten_runtime/ten_env/log.h"
#include "ten_runtime/ten_env/internal/log.h"
#include "ten_utils/lib/error.h"
#include "ten_utils/macro/check.h"
#include "ten_utils/macro/memory.h"

typedef struct ten_env_notify_log_ctx_t {
  int32_t level;
  ten_string_t func_name;
  ten_string_t file_name;
  size_t line_no;
  ten_string_t msg;
} ten_env_notify_log_ctx_t;

static ten_env_notify_log_ctx_t *ten_env_notify_log_ctx_create(
    int32_t level, const char *func_name, const char *file_name, size_t line_no,
    const char *msg) {
  ten_env_notify_log_ctx_t *ctx = TEN_MALLOC(sizeof(ten_env_notify_log_ctx_t));
  TEN_ASSERT(ctx, "Failed to allocate memory.");

  ctx->level = level;

  if (func_name) {
    ten_string_init_from_c_str(&ctx->func_name, func_name, strlen(func_name));
  } else {
    ten_string_init(&ctx->func_name);
  }

  if (file_name) {
    ten_string_init_from_c_str(&ctx->file_name, file_name, strlen(file_name));
  } else {
    ten_string_init(&ctx->file_name);
  }
  ctx->line_no = line_no;

  if (msg) {
    ten_string_init_from_c_str(&ctx->msg, msg, strlen(msg));
  } else {
    ten_string_init(&ctx->msg);
  }

  return ctx;
}

static void ten_env_notify_log_ctx_destroy(ten_env_notify_log_ctx_t *ctx) {
  TEN_ASSERT(ctx, "Invalid argument.");

  ten_string_deinit(&ctx->func_name);
  ten_string_deinit(&ctx->file_name);
  ten_string_deinit(&ctx->msg);

  TEN_FREE(ctx);
}

static void ten_env_proxy_notify_log(ten_env_t *ten_env, void *user_data) {
  TEN_ASSERT(user_data, "Invalid argument.");
  TEN_ASSERT(ten_env && ten_env_check_integrity(ten_env, true),
             "Should not happen.");

  ten_env_notify_log_ctx_t *ctx = user_data;
  TEN_ASSERT(ctx, "Should not happen.");

  ten_env_log(ten_env, ctx->level, ten_string_get_raw_str(&ctx->func_name),
              ten_string_get_raw_str(&ctx->file_name), ctx->line_no,
              ten_string_get_raw_str(&ctx->msg));

  ten_env_notify_log_ctx_destroy(ctx);
}

PyObject *ten_py_ten_env_log(PyObject *self, PyObject *args) {
  ten_py_ten_env_t *py_ten_env = (ten_py_ten_env_t *)self;
  TEN_ASSERT(py_ten_env && ten_py_ten_env_check_integrity(py_ten_env),
             "Invalid argument.");

  if (PyTuple_GET_SIZE(args) != 5) {
    return ten_py_raise_py_value_error_exception(
        "Invalid argument count when ten_env.log.");
  }

  TEN_LOG_LEVEL level = TEN_LOG_LEVEL_INVALID;
  const char *func_name = NULL;
  const char *file_name = NULL;
  size_t line_no = 0;
  const char *msg = NULL;
  if (!PyArg_ParseTuple(args, "izzis", &level, &func_name, &file_name, &line_no,
                        &msg)) {
    return ten_py_raise_py_value_error_exception(
        "Failed to parse argument when ten_env.log.");
  }

  if (!py_ten_env->c_ten_env_proxy) {
    Py_RETURN_NONE;
  }

  ten_error_t err;
  ten_error_init(&err);

  if (py_ten_env->c_ten_env->attach_to == TEN_ENV_ATTACH_TO_ADDON) {
    // TODO(Wei): This function is currently specifically designed for the addon
    // because the addon currently does not have a main thread, so it's unable
    // to check thread safety. Once the main thread for the addon is determined
    // in the future, these hacks made specifically for the addon can be
    // completely removed, and comprehensive thread safety checking can be
    // implemented.
    ten_env_log_without_check_thread(py_ten_env->c_ten_env, level, func_name,
                                     file_name, line_no, msg);
  } else {
    ten_env_notify_log_ctx_t *ctx = ten_env_notify_log_ctx_create(
        level, func_name, file_name, line_no, msg);

    if (!ten_env_proxy_notify(py_ten_env->c_ten_env_proxy,
                              ten_env_proxy_notify_log, ctx, false, &err)) {
      ten_env_notify_log_ctx_destroy(ctx);
    }
  }

  ten_error_deinit(&err);
  Py_RETURN_NONE;
}
