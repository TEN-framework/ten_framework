/**
 *
 * Agora Real Time Engagement
 * Created by Wei Hu in 2022-04.
 * Copyright (c) 2023 Agora IO. All rights reserved.
 *
 */
#pragma once

#include <node_api.h>

#include "ten_utils/lib/mutex.h"
#include "ten_utils/lib/ref.h"
#include "ten_utils/lib/signature.h"
#include "ten_utils/lib/string.h"
#include "ten_utils/sanitizer/thread_check.h"

#define TEN_NODEJS_THREADSAFE_FUNCTION_SIGNATURE 0x1D11D6EF2722D8FBU

typedef struct ten_nodejs_tsfn_t {
  ten_signature_t signature;
  ten_sanitizer_thread_check_t thread_check;

  ten_ref_t ref;  // Used to determine the timing of destroying this TSFN.

  ten_mutex_t *lock;
  ten_string_t name;

  napi_threadsafe_function tsfn;  // The TSFN itself.

  // The JS function which this tsfn calling.
  //
  // Because the JS functions pointed to by thread-safe functions may not
  // necessarily be original functions existing in the RTE JS world, they could
  // be dynamically created JS functions, such as the unlink handler function of
  // js_ref. The life cycle of dynamically generated JS functions is bound to
  // the thread-safe functions. Therefore, for unified handling, RTE first
  // acquires a reference to a JS function to prevent it from being garbage
  // collected. Then, when the thread-safe function is finalized, RTE cancels
  // that reference, allowing the JS function to be garbage collected.
  napi_ref js_func_ref;
} ten_nodejs_tsfn_t;
