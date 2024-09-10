//
// This file is part of the TEN Framework project.
// See https://github.com/TEN-framework/ten_framework/LICENSE for license
// information.
//
#include "include_internal/ten_runtime/binding/python/common/error.h"
#include "include_internal/ten_runtime/binding/python/msg/audio_frame.h"
#include "include_internal/ten_runtime/binding/python/ten_env/ten_env.h"
#include "ten_runtime/extension/extension.h"
#include "ten_runtime/ten_env/internal/send.h"
#include "ten_utils/macro/memory.h"

typedef struct ten_env_notify_send_audio_frame_info_t {
  ten_shared_ptr_t *c_audio_frame;
} ten_env_notify_send_audio_frame_info_t;

static ten_env_notify_send_audio_frame_info_t *
ten_env_notify_send_audio_frame_info_create(ten_shared_ptr_t *c_cmd) {
  TEN_ASSERT(c_cmd, "Invalid argument.");

  ten_env_notify_send_audio_frame_info_t *info =
      TEN_MALLOC(sizeof(ten_env_notify_send_audio_frame_info_t));
  TEN_ASSERT(info, "Failed to allocate memory.");

  info->c_audio_frame = c_cmd;

  return info;
}

static void ten_env_notify_send_audio_frame_info_destroy(
    ten_env_notify_send_audio_frame_info_t *info) {
  TEN_ASSERT(info, "Invalid argument.");

  if (info->c_audio_frame) {
    ten_shared_ptr_destroy(info->c_audio_frame);
    info->c_audio_frame = NULL;
  }

  TEN_FREE(info);
}

static void ten_env_notify_send_audio_frame(ten_env_t *ten_env,
                                            void *user_data) {
  TEN_ASSERT(user_data, "Invalid argument.");
  TEN_ASSERT(ten_env && ten_env_check_integrity(ten_env, true),
             "Should not happen.");

  ten_env_notify_send_audio_frame_info_t *notify_info = user_data;

  ten_env_send_audio_frame(ten_env, notify_info->c_audio_frame, NULL);

  ten_env_notify_send_audio_frame_info_destroy(notify_info);
}

PyObject *ten_py_ten_env_send_audio_frame(PyObject *self, PyObject *args) {
  ten_py_ten_env_t *py_ten = (ten_py_ten_env_t *)self;
  TEN_ASSERT(py_ten && ten_py_ten_env_check_integrity(py_ten),
             "Invalid argument.");

  bool success = true;

  ten_error_t err;
  ten_error_init(&err);

  ten_py_audio_frame_t *py_audio_frame = NULL;
  if (!PyArg_ParseTuple(args, "O!", ten_py_audio_frame_py_type(),
                        &py_audio_frame)) {
    success = false;
    ten_py_raise_py_type_error_exception(
        "Invalid argument type when send audio frame.");
    goto done;
  }

  ten_shared_ptr_t *cloned_frame =
      ten_shared_ptr_clone(py_audio_frame->msg.c_msg);
  ten_env_notify_send_audio_frame_info_t *notify_info =
      ten_env_notify_send_audio_frame_info_create(cloned_frame);

  if (!ten_env_proxy_notify(py_ten->c_ten_env_proxy,
                            ten_env_notify_send_audio_frame, notify_info, false,
                            &err)) {
    ten_env_notify_send_audio_frame_info_destroy(notify_info);
    success = false;
    ten_py_raise_py_runtime_error_exception("Failed to send audio frame.");
    goto done;
  } else {
    // Destroy the C message from the Python message as the ownership has been
    // transferred to the notify_info.
    ten_py_msg_destroy_c_msg(&py_audio_frame->msg);
  }

done:
  ten_error_deinit(&err);

  if (success) {
    Py_RETURN_NONE;
  } else {
    return NULL;
  }
}
