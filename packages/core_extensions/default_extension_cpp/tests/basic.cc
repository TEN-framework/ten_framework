//
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0.
// See the LICENSE file for more information.
//
#include "gtest/gtest.h"
#include "ten_runtime/addon/addon.h"
#include "ten_runtime/binding/cpp/ten.h"
#include "ten_utils/lib/path.h"
#include "ten_utils/lib/string.h"

namespace {

class default_extension_tester : public ten::extension_tester_t {
 public:
  void on_start(ten::ten_env_tester_t &ten_env) override {
    // Send the first command to the extension.
    auto new_cmd = ten::cmd_t::create("foo");

    ten_env.send_cmd(std::move(new_cmd),
                     [](ten::ten_env_tester_t &ten_env,
                        std::unique_ptr<ten::cmd_result_t> result,
                        ten::error_t * /*error*/) {
                       if (result->get_status_code() == TEN_STATUS_CODE_OK) {
                         ten_env.stop_test();
                       }
                     });
  }
};

}  // namespace

TEST(Test, Basic) {  // NOLINT
  auto *tester = new default_extension_tester();

  ten_string_t *path = ten_path_get_executable_path();
  ten_path_join_c_str(path, "../ten_packages/extension/default_extension_cpp/");

  tester->add_addon_base_dir(ten_string_get_raw_str(path));

  ten_string_destroy(path);

  tester->set_test_mode_single("default_extension_cpp");

  tester->run();

  delete tester;
}
