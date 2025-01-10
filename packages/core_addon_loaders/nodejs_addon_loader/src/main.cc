//
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0.
// See the LICENSE file for more information.
//
#include <cstring>
#include <iostream>
#include <string>
#include <thread>

#include "include_internal/ten_runtime/binding/cpp/detail/addon_loader.h"
#include "include_internal/ten_runtime/binding/cpp/detail/addon_manager.h"
#include "node.h"
#include "ten_utils/lib/module.h"
#include "v8-script.h"

using node::CommonEnvironmentSetup;
using node::Environment;
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

class nodejs_addon_loader_t : public ten::addon_loader_t {
 public:
  explicit nodejs_addon_loader_t(const char *name) { (void)name; };

  ~nodejs_addon_loader_t() override { on_deinit(); }

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
        std::cerr << "Error: " << err << std::endl;
      }
      return 1;
    }

    this->setup_ = std::move(setup);

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
          "const js_require = require('module').createRequire(process.cwd() +"
          "'/');"
          "console.log(process.cwd());"
          "const ten_runtime_nodejs = js_require('./ten_packages/system/ten_"
          "runtime_nodejs');"
          "console.log(ten_runtime_nodejs);"
          "ten_runtime_nodejs.AddonManager._load_all_addons();"
          "ten_runtime_nodejs.AddonManager._register_all_addons();"
          "console.log('hello');");

      if (loadenv_ret.IsEmpty()) {  // There has been a JS exception.
        return 1;
      }

      exit_code = node::SpinEventLoop(env).FromMaybe(1);

      // Call JS code.
      // const char *js_code =
      //     // "ten_runtime_nodejs.AddonManager._load_all_addons();"
      //     // "ten_runtime_nodejs.AddonManager._register_all_addons();"
      //     "console.log('hello');";
      // v8::Local<v8::String> source =
      //     v8::String::NewFromUtf8(this->setup_->isolate(), js_code,
      //                             v8::NewStringType::kNormal)
      //         .ToLocalChecked();
      // v8::Local<v8::Script> script =
      //     v8::Script::Compile(this->setup_->context(),
      //     source).ToLocalChecked();
      // script->Run(this->setup_->context()).ToLocalChecked();
    }

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
        std::cerr << "Nodejs addon loader error: " << error << std::endl;
      }

      if (result->early_return()) {
        std::cerr << "Nodejs addon loader early return: " << result->exit_code()
                  << std::endl;
        exit(result->exit_code());
      }

      // RunNodeInstance in another thread
      this->node_thread_ = std::thread([this, args]() {
        std::unique_ptr<MultiIsolatePlatform> platform =
            MultiIsolatePlatform::Create(4);
        V8::InitializePlatform(platform.get());
        V8::Initialize();

        int ret = RunNodeInstance(platform.get(), args, {});
        if (ret != 0) {
          // exit(ret);
        }
      });
    } catch (const std::exception &e) {
      std::cerr << "Nodejs addon loader init exception: " << e.what()
                << std::endl;
      exit(1);
    }
  }

  void on_deinit() override {
    if (this->setup_) {
      node::Stop(this->setup_->env());
    }

    this->node_thread_.join();

    this->setup_.reset();  // reset the setup

    V8::Dispose();
    V8::DisposePlatform();

    node::TearDownOncePerProcess();
  }

  // **Note:** This function, used to dynamically load other addons, may be
  // called from multiple threads. Therefore, it must be thread-safe. Since it
  // calls `ten_py_gil_state_ensure` and `ten_py_gil_state_release`, thread
  // safety is ensured.
  void on_load_addon(TEN_ADDON_TYPE addon_type,
                     const char *addon_name) override {
    if (this->setup_) {
      // Locker locker(this->setup_->isolate());
      // Isolate::Scope isolate_scope(this->setup_->isolate());
      // HandleScope handle_scope(this->setup_->isolate());
      // Context::Scope context_scope(this->setup_->context());

      // const char *js_code = "console.log('js on_load_addon');";

      // v8::Local<v8::String> source =
      //     v8::String::NewFromUtf8(this->setup_->isolate(), js_code,
      //                             v8::NewStringType::kNormal)
      //         .ToLocalChecked();
      // v8::Local<v8::Script> script =
      //     v8::Script::Compile(this->setup_->context(),
      //     source).ToLocalChecked();
      // script->Run(this->setup_->context()).ToLocalChecked();
    }
  }

 private:
  std::unique_ptr<CommonEnvironmentSetup> setup_{nullptr};
  std::thread node_thread_;
};

TEN_CPP_REGISTER_ADDON_AS_ADDON_LOADER(nodejs_addon_loader,
                                       nodejs_addon_loader_t);

}  // namespace
