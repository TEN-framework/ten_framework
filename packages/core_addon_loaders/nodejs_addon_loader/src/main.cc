//
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0.
// See the LICENSE file for more information.
//
#include <uv.h>

#include <condition_variable>
#include <cstring>
#include <iostream>
#include <string>
#include <thread>

#include "node.h"
#include "v8-script.h"
// Header file region splitter.
#include "include_internal/ten_runtime/binding/cpp/detail/addon_loader.h"
#include "include_internal/ten_runtime/binding/cpp/detail/addon_manager.h"
#include "ten_utils/lib/module.h"

using node::CommonEnvironmentSetup;
using node::MultiIsolatePlatform;
using v8::Context;
using v8::HandleScope;
using v8::Isolate;
using v8::Locker;
using v8::MaybeLocal;
using v8::V8;
using v8::Value;

static void foo() {}

namespace {

class nodejs_addon_loader_t;

class load_addon_data_t {
 public:
  std::string addon_name;
  nodejs_addon_loader_t *loader;
  bool addon_loaded_{false};

  load_addon_data_t(const char *addon_name, nodejs_addon_loader_t *loader)
      : addon_name(addon_name), loader(loader) {}
};

class nodejs_addon_loader_t : public ten::addon_loader_t {
 public:
  explicit nodejs_addon_loader_t(const char *name) { (void)name; };

  int RunNodeInstance(MultiIsolatePlatform *platform,
                      const std::vector<std::string> &args,
                      const std::vector<std::string> &exec_args) {
    int exit_code = 0;

    // Setup up a libuv event loop, v8::Isolate, and Node.js Environment.
    std::vector<std::string> errors;
    std::unique_ptr<CommonEnvironmentSetup> setup =
        CommonEnvironmentSetup::Create(platform, &errors, args, exec_args);
    if (!setup) {
      for (const std::string &err : errors) {
        std::cerr << "Error: " << err << '\n';
      }
      return 1;
    }

    this->setup_ = std::move(setup);
    this->event_loop_ = this->setup_->event_loop();

    auto *isolate = this->setup_->isolate();
    auto *env = this->setup_->env();

    {
      Locker locker(isolate);
      Isolate::Scope isolate_scope(isolate);
      HandleScope handle_scope(isolate);
      // The v8::Context needs to be entered when node::CreateEnvironment() and
      // node::LoadEnvironment() are being called.
      Context::Scope context_scope(this->setup_->context());

      // Set up the Node.js instance for execution, and run code inside of it.
      // There is also a variant that takes a callback and provides it with
      // the `require` and `process` objects, so that it can manually compile
      // and run scripts as needed.
      // The `require` function inside this script does *not* access the file
      // system, and can only load built-in Node.js modules.
      // `module.createRequire()` is being used to create one that is able to
      // load files from the disk, and uses the standard CommonJS file loader
      // instead of the internal-only `require` function.
      MaybeLocal<Value> loadenv_ret = node::LoadEnvironment(
          env,
          "js_require = require('module').createRequire(process.cwd() + "
          "'/');"
          "global.ten_runtime_nodejs = "
          "js_require('./ten_packages/system/ten_runtime_nodejs');"
          "setInterval(() => {}, 1000);");

      if (loadenv_ret.IsEmpty()) {  // There has been a JS exception.
        return 1;
      }

      this->node_thread_started_ = true;
      this->cv_.notify_one();

      exit_code = node::SpinEventLoop(env).FromMaybe(1);
    }

    this->setup_.reset();

    return exit_code;
  }

  static void load_nodejs_lib() {
    ten_string_t *module_path =
        ten_path_get_module_path(reinterpret_cast<const void *>(foo));
    TEN_ASSERT(module_path, "Failed to get module path.");

    ten_string_t *nodejs_lib_path = ten_string_create_formatted(
        "%s/libnode.so", ten_string_get_raw_str(module_path));

    // The libnode.so must be loaded globally using dlopen, and cannot be a
    // regular shared library dependency. Note that the 2nd parameter must be
    // 0 (as_local = false).
    ten_module_load(nodejs_lib_path, 0);

    ten_string_destroy(nodejs_lib_path);
    ten_string_destroy(module_path);
  }

  void on_init() override {
    // Do some initializations.
    load_nodejs_lib();

    try {
      std::vector<std::string> args;
      args.emplace_back("node");
      args.emplace_back("--expose-gc");
      args.emplace_back("--trace-warnings");
      args.emplace_back("-e");
      args.emplace_back("console.log('wft');");

      std::shared_ptr<node::InitializationResult> result =
          node::InitializeOncePerProcess(
              args,
              {
                  node::ProcessInitializationFlags::kNoInitializeV8,
                  node::ProcessInitializationFlags::kNoInitializeNodeV8Platform,
                  // This is used to test NODE_REPL_EXTERNAL_MODULE is disabled
                  // with kDisableNodeOptionsEnv. If other tests need
                  // NODE_OPTIONS support in the future, split this
                  // configuration out as a command line option.
                  node::ProcessInitializationFlags::kDisableNodeOptionsEnv,
              });

      for (const std::string &error : result->errors()) {
        std::cerr << "Nodejs addon loader error: " << error << '\n';
      }

      if (result->early_return()) {
        std::cerr << "Nodejs addon loader early return: " << result->exit_code()
                  << '\n';
        exit(result->exit_code());
      }

      // RunNodeInstance in another thread
      this->node_thread_ = std::thread([this, args]() {
        std::unique_ptr<MultiIsolatePlatform> platform =
            MultiIsolatePlatform::Create(4);
        V8::InitializePlatform(platform.get());
        V8::Initialize();

        int ret = RunNodeInstance(platform.get(), args, {});
        std::cout << "Node Instance run completed with code: " << ret << '\n';

        V8::Dispose();
        V8::DisposePlatform();

        node::TearDownOncePerProcess();

        this->node_thread_started_ = true;
        this->cv_.notify_one();
      });
    } catch (const std::exception &e) {
      std::cerr << "Nodejs addon loader init exception: " << e.what() << '\n';
      exit(1);
    }

    // Wait for the node thread to start.
    std::unique_lock<std::mutex> lock(this->mutex_);
    this->cv_.wait(lock, [this]() { return this->node_thread_started_; });
  }

  void on_deinit() override {
    if (this->setup_ != nullptr && this->event_loop_ != nullptr) {
      auto *deinit_handle = new uv_async_t();
      deinit_handle->data = this;

      uv_async_init(this->event_loop_, deinit_handle, [](uv_async_t *handle) {
        auto *this_ptr = static_cast<nodejs_addon_loader_t *>(handle->data);

        Locker locker(this_ptr->setup_->isolate());
        Isolate::Scope isolate_scope(this_ptr->setup_->isolate());
        HandleScope handle_scope(this_ptr->setup_->isolate());
        Context::Scope context_scope(this_ptr->setup_->context());

        std::string js_code =
            "global.gc();"
            "console.log('gc done');";
        v8::Local<v8::String> source =
            v8::String::NewFromUtf8(this_ptr->setup_->isolate(),
                                    js_code.c_str(), v8::NewStringType::kNormal)
                .ToLocalChecked();
        v8::Local<v8::Script> script =
            v8::Script::Compile(this_ptr->setup_->context(), source)
                .ToLocalChecked();
        script->Run(this_ptr->setup_->context()).ToLocalChecked();

        this_ptr->gc_done_ = true;
        this_ptr->cv_.notify_one();

        uv_close(reinterpret_cast<uv_handle_t *>(handle),
                 [](uv_handle_t *handle) {
                   auto *async_handle = reinterpret_cast<uv_async_t *>(handle);
                   delete async_handle;
                 });
      });
      uv_async_send(deinit_handle);

      // Wait for the gc to be done.
      std::unique_lock<std::mutex> lock(this->mutex_);
      this->cv_.wait(lock, [this]() { return this->gc_done_; });
    }

    if (this->setup_) {
      node::Stop(this->setup_->env());
    }

    this->node_thread_.join();

    std::cout << "Nodejs addon loader deinit completed" << '\n';
  }

  // **Note:** This function, used to dynamically load other addons, may be
  // called from multiple threads. Therefore, it must be thread-safe.
  void on_load_addon(TEN_ADDON_TYPE addon_type,
                     const char *addon_name) override {
    if (this->setup_ != nullptr && this->event_loop_ != nullptr) {
      auto *load_addon_handle = new uv_async_t();

      auto *load_addon_data = new load_addon_data_t(addon_name, this);
      load_addon_handle->data = load_addon_data;
      uv_async_init(
          this->event_loop_, load_addon_handle, [](uv_async_t *handle) {
            auto *load_addon_data =
                static_cast<load_addon_data_t *>(handle->data);

            auto *this_ptr = load_addon_data->loader;
            std::string addon_name = load_addon_data->addon_name;

            Locker locker(this_ptr->setup_->isolate());
            Isolate::Scope isolate_scope(this_ptr->setup_->isolate());
            HandleScope handle_scope(this_ptr->setup_->isolate());
            Context::Scope context_scope(this_ptr->setup_->context());

            std::string js_code =
                "if "
                "(global.ten_runtime_nodejs.AddonManager._load_single_addon('" +
                addon_name + "')) {" +
                "global.ten_runtime_nodejs.AddonManager._register_single_addon("
                "'" +
                addon_name + "', null);" + "}";

            v8::Local<v8::String> source =
                v8::String::NewFromUtf8(this_ptr->setup_->isolate(),
                                        js_code.c_str(),
                                        v8::NewStringType::kNormal)
                    .ToLocalChecked();
            v8::Local<v8::Script> script =
                v8::Script::Compile(this_ptr->setup_->context(), source)
                    .ToLocalChecked();
            script->Run(this_ptr->setup_->context()).ToLocalChecked();

            uv_close(reinterpret_cast<uv_handle_t *>(handle),
                     [](uv_handle_t *handle) {
                       auto *async_handle =
                           reinterpret_cast<uv_async_t *>(handle);
                       delete async_handle;
                     });

            load_addon_data->addon_loaded_ = true;
            this_ptr->cv_.notify_one();
          });
      uv_async_send(load_addon_handle);

      // Wait for the addon to be loaded.
      std::unique_lock<std::mutex> lock(this->mutex_);
      this->cv_.wait(
          lock, [load_addon_data]() { return load_addon_data->addon_loaded_; });

      delete load_addon_data;
    }
  }

 private:
  std::unique_ptr<CommonEnvironmentSetup> setup_{nullptr};
  uv_loop_s *event_loop_{nullptr};
  std::thread node_thread_;
  bool node_thread_started_{false};

  std::mutex mutex_;
  std::condition_variable cv_;
  bool gc_done_{false};
};

TEN_CPP_REGISTER_ADDON_AS_ADDON_LOADER(nodejs_addon_loader,
                                       nodejs_addon_loader_t);

}  // namespace
