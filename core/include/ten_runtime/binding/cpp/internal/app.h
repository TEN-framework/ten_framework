//
// This file is part of the TEN Framework project.
// See https://github.com/TEN-framework/ten_framework/LICENSE for license
// information.
//
#pragma once

#include <cstddef>
#include <string>

#include "ten_utils/macro/check.h"
#include "ten_runtime/app/app.h"
#include "ten_runtime/binding/common.h"
#include "ten_runtime/binding/cpp/internal/common.h"
#include "ten_runtime/binding/cpp/internal/extension_group.h"
#include "ten_runtime/ten.h"

using ten_app_t = struct ten_app_t;

namespace ten {

class app_t {
 public:
  app_t()
      : app_(ten_app_create(cpp_app_on_init_cb_wrapper, nullptr, nullptr)),
        ten_(new ten_env_t(ten_app_get_ten_env(app_))) {
    TEN_ASSERT(ten_, "Should not happen.");
    ten_binding_handle_set_me_in_target_lang(
        reinterpret_cast<ten_binding_handle_t *>(app_),
        static_cast<void *>(this));
  }

  virtual ~app_t() {
    ten_app_destroy(app_);
    app_ = nullptr;

    TEN_ASSERT(ten_, "Should not happen.");
    delete ten_;
  }

  // @{
  app_t(const app_t &) = delete;
  app_t(app_t &&) = delete;
  app_t &operator=(const app_t &) = delete;
  app_t &operator=(app_t &&) = delete;
  // @}

  // NOLINTNEXTLINE(readability-convert-member-functions-to-static)
  bool run(bool run_in_background = false, error_t *err = nullptr) {
    if (app_ == nullptr) {
      return false;
    }

    return ten_app_run(
        app_, run_in_background,
        err != nullptr ? err->get_internal_representation() : nullptr);
  }

  bool close(error_t *err = nullptr) {
    return ten_app_close(
        app_, err != nullptr ? err->get_internal_representation() : nullptr);
  }

  bool wait(error_t *err = nullptr) {
    return ten_app_wait(
        app_, err != nullptr ? err->get_internal_representation() : nullptr);
  }

 protected:
  virtual void on_init(ten_env_t &ten_env) { ten_env.on_init_done(); }

  virtual void on_deinit(ten_env_t &ten_env) { ten_env.on_deinit_done(); }

 private:
  static void cpp_app_on_init_cb_wrapper(ten_app_t *app, ::ten_env_t *ten_env) {
    TEN_ASSERT(app && ten_app_check_integrity(app, true), "Should not happen.");
    TEN_ASSERT(ten_env && ten_env_check_integrity(ten_env, true),
               "Should not happen.");
    TEN_ASSERT(ten_app_get_ten_env(app) == ten_env, "Should not happen.");

    auto *cpp_app =
        static_cast<app_t *>(ten_binding_handle_get_me_in_target_lang(
            reinterpret_cast<ten_binding_handle_t *>(app)));
    auto *cpp_ten_env =
        static_cast<ten_env_t *>(ten_binding_handle_get_me_in_target_lang(
            reinterpret_cast<ten_binding_handle_t *>(ten_env)));

    cpp_app->on_init_helper_for_cpp(*cpp_ten_env);
  }

  void on_init_helper_for_cpp(ten_env_t &ten_env) {
    // The TEN runtime does not use C++ exceptions. The use of try/catch here is
    // merely to intercept any exceptions that might be thrown by the user's app
    // code. If exceptions are disabled during the compilation of the TEN
    // runtime (i.e., with -fno-exceptions), it implies that the extensions used
    // will also not employ exceptions (otherwise it would be unreasonable). In
    // this case, the try/catch blocks become no-ops. Conversely, if exceptions
    // are enabled during compilation, then the try/catch here can intercept all
    // exceptions thrown by user code that are not already caught, serving as a
    // kind of fallback.
    try {
      on_init(ten_env);
    } catch (...) {
      TEN_LOGW("Caught a exception of type '%s' in App on_init().",
               curr_exception_type_name().c_str());
    }
  }

  static void cpp_app_on_close_cb_wrapper(ten_app_t *app,
                                          ::ten_env_t *ten_env) {
    TEN_ASSERT(app && ten_app_check_integrity(app, true), "Should not happen.");
    TEN_ASSERT(ten_env && ten_env_check_integrity(ten_env, true),
               "Should not happen.");
    TEN_ASSERT(ten_app_get_ten_env(app) == ten_env, "Should not happen.");

    auto *cpp_app =
        static_cast<app_t *>(ten_binding_handle_get_me_in_target_lang(
            reinterpret_cast<ten_binding_handle_t *>(app)));
    auto *cpp_ten_env =
        static_cast<ten_env_t *>(ten_binding_handle_get_me_in_target_lang(
            reinterpret_cast<ten_binding_handle_t *>(ten_env)));

    cpp_app->on_deinit_helper_for_cpp(*cpp_ten_env);
  }

  void on_deinit_helper_for_cpp(ten_env_t &ten_env) {
    // The TEN runtime does not use C++ exceptions. The use of try/catch here is
    // merely to intercept any exceptions that might be thrown by the user's app
    // code. If exceptions are disabled during the compilation of the TEN
    // runtime (i.e., with -fno-exceptions), it implies that the extensions used
    // will also not employ exceptions (otherwise it would be unreasonable). In
    // this case, the try/catch blocks become no-ops. Conversely, if exceptions
    // are enabled during compilation, then the try/catch here can intercept all
    // exceptions thrown by user code that are not already caught, serving as a
    // kind of fallback.
    try {
      on_deinit(ten_env);
    } catch (...) {
      TEN_LOGW("Caught a exception of type '%s' in App on_close().",
               curr_exception_type_name().c_str());
    }
  }

  ::ten_app_t *app_ = nullptr;
  ten_env_t *ten_;
};

}  // namespace ten
