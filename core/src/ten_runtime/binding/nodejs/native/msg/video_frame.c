//
// Copyright © 2024 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#include "include_internal/ten_runtime/binding/nodejs/msg/video_frame.h"

#include "include_internal/ten_runtime/msg/msg.h"
#include "ten_runtime/msg/video_frame/video_frame.h"
#include "ten_utils/macro/memory.h"

static napi_ref js_video_frame_constructor_ref = NULL;  // NOLINT

static napi_value ten_nodejs_video_frame_register_class(
    napi_env env, napi_callback_info info) {
  const size_t argc = 1;
  napi_value args[argc];  // video_frame constructor
  if (!ten_nodejs_get_js_func_args(env, info, args, argc)) {
    napi_fatal_error(NULL, NAPI_AUTO_LENGTH,
                     "Incorrect number of parameters passed.",
                     NAPI_AUTO_LENGTH);
    TEN_ASSERT(0, "Should not happen.");
    return NULL;
  }

  napi_status status =
      napi_create_reference(env, args[0], 1, &js_video_frame_constructor_ref);
  if (status != napi_ok) {
    napi_fatal_error(
        NULL, NAPI_AUTO_LENGTH,
        "Failed to create JS reference to JS video_frame constructor.",
        NAPI_AUTO_LENGTH);
    TEN_ASSERT(
        0, "Failed to create JS reference to JS video_frame constructor: %d",
        status);
  }

  return js_undefined(env);
}

static void ten_nodejs_video_frame_destroy(ten_nodejs_video_frame_t *self) {
  TEN_ASSERT(self, "Should not happen.");

  ten_nodejs_msg_deinit(&self->msg);

  TEN_FREE(self);
}

static void ten_nodejs_video_frame_finalize(napi_env env, void *video_frame,
                                            void *hint) {
  ten_nodejs_video_frame_t *video_frame_bridge = video_frame;
  TEN_ASSERT(video_frame_bridge, "Should not happen.");

  napi_delete_reference(env, video_frame_bridge->msg.bridge.js_instance_ref);

  ten_nodejs_video_frame_destroy(video_frame_bridge);
}

static napi_value ten_nodejs_video_frame_create(napi_env env,
                                                napi_callback_info info) {
  const size_t argc = 2;
  napi_value args[argc];  // this, video_frame_name
  if (!ten_nodejs_get_js_func_args(env, info, args, argc)) {
    napi_fatal_error(NULL, NAPI_AUTO_LENGTH,
                     "Incorrect number of parameters passed.",
                     NAPI_AUTO_LENGTH);
    TEN_ASSERT(0, "Should not happen.");
  }

  ten_string_t video_frame_name;
  ten_string_init(&video_frame_name);

  bool rc = ten_nodejs_get_str_from_js(env, args[1], &video_frame_name);
  if (!rc) {
    napi_fatal_error(NULL, NAPI_AUTO_LENGTH, "Failed to get video_frame_name.",
                     NAPI_AUTO_LENGTH);
    TEN_ASSERT(0, "Should not happen.");
  }

  ten_error_t error;
  ten_error_init(&error);

  ten_shared_ptr_t *c_video_frame =
      ten_video_frame_create(ten_string_get_raw_str(&video_frame_name), &error);
  TEN_ASSERT(c_video_frame, "Failed to create video_frame.");

  ten_string_deinit(&video_frame_name);

  ten_nodejs_video_frame_t *video_frame_bridge =
      TEN_MALLOC(sizeof(ten_nodejs_video_frame_t));
  TEN_ASSERT(video_frame_bridge, "Failed to allocate memory.");

  ten_nodejs_msg_init_from_c_msg(&video_frame_bridge->msg, c_video_frame);
  // Decrement the reference count of c_video_frame to indicate that the JS
  // video_frame takes the full ownership of this c_video_frame, in other words,
  // when the JS video_frame is finalized, its C video_frame would be destroyed,
  // too.
  ten_shared_ptr_destroy(c_video_frame);

  napi_status status = napi_wrap(
      env, args[0], video_frame_bridge, ten_nodejs_video_frame_finalize, NULL,
      &video_frame_bridge->msg.bridge.js_instance_ref);
  if (status != napi_ok) {
    napi_fatal_error(NULL, NAPI_AUTO_LENGTH,
                     "Failed to wrap JS video_frame object.", NAPI_AUTO_LENGTH);
    TEN_ASSERT(0, "Failed to wrap JS video_frame object: %d", status);
  }

  return js_undefined(env);
}

napi_value ten_nodejs_video_frame_wrap(napi_env env,
                                       ten_shared_ptr_t *video_frame) {
  TEN_ASSERT(video_frame && ten_msg_check_integrity(video_frame),
             "Should not happen.");

  ten_nodejs_video_frame_t *video_frame_bridge =
      TEN_MALLOC(sizeof(ten_nodejs_video_frame_t));
  TEN_ASSERT(video_frame_bridge, "Failed to allocate memory.");

  ten_nodejs_msg_init_from_c_msg(&video_frame_bridge->msg, video_frame);

  napi_value js_msg_name = NULL;
  napi_value js_create_shell_only_flag = NULL;

  const char *msg_name = ten_msg_get_name(video_frame);
  TEN_ASSERT(msg_name, "Should not happen.");

  napi_status status =
      napi_create_string_utf8(env, msg_name, NAPI_AUTO_LENGTH, &js_msg_name);
  ASSERT_IF_NAPI_FAIL(status == napi_ok && js_msg_name != NULL,
                      "Failed to create JS string: %d", status);

  status = napi_get_boolean(env, true, &js_create_shell_only_flag);
  ASSERT_IF_NAPI_FAIL(status == napi_ok && js_create_shell_only_flag != NULL,
                      "Failed to create JS boolean: %d", status);

  napi_value argv[] = {js_msg_name, js_create_shell_only_flag};

  napi_value js_video_frame = ten_nodejs_create_new_js_object_and_wrap(
      env, js_video_frame_constructor_ref, video_frame_bridge,
      ten_nodejs_video_frame_finalize,
      &video_frame_bridge->msg.bridge.js_instance_ref, 2, argv);
  ASSERT_IF_NAPI_FAIL(js_video_frame != NULL,
                      "Failed to create JS video_frame object.");

  return js_video_frame;
}

napi_value ten_nodejs_video_frame_module_init(napi_env env,
                                              napi_value exports) {
  EXPORT_FUNC(env, exports, ten_nodejs_video_frame_register_class);
  EXPORT_FUNC(env, exports, ten_nodejs_video_frame_create);

  return exports;
}