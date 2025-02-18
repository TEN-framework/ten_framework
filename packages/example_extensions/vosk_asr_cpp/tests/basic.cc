//
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0.
// See the LICENSE file for more information.
//
#include "gtest/gtest.h"
#include "ten_runtime/binding/cpp/ten.h"

namespace {

class vosk_asr_cpp_tester : public ten::extension_tester_t {
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
  auto *tester = new vosk_asr_cpp_tester();
  tester->set_test_mode_single("vosk_asr_cpp");
  tester->run();
  delete tester;
}
