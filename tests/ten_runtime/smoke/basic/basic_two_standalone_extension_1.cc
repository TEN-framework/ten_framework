//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#include <nlohmann/json.hpp>
#include <string>

#include "gtest/gtest.h"
#include "include_internal/ten_runtime/binding/cpp/ten.h"
#include "ten_utils/lib/thread.h"
#include "tests/common/client/cpp/msgpack_tcp.h"
#include "tests/ten_runtime/smoke/util/binding/cpp/check.h"

namespace {

class test_extension_1 : public ten::extension_t {
 public:
  explicit test_extension_1(const char *name) : ten::extension_t(name) {}

  void on_cmd(ten::ten_env_t &ten_env,
              std::unique_ptr<ten::cmd_t> cmd) override {
    auto cmd_result = ten::cmd_result_t::create(TEN_STATUS_CODE_OK);
    cmd_result->set_property("detail", "hello from test extension 1");
    ten_env.return_result(std::move(cmd_result), std::move(cmd));
  }
};

class test_extension_2 : public ten::extension_t {
 public:
  explicit test_extension_2(const char *name) : ten::extension_t(name) {}

  void on_cmd(ten::ten_env_t &ten_env,
              std::unique_ptr<ten::cmd_t> cmd) override {
    auto cmd_result = ten::cmd_result_t::create(TEN_STATUS_CODE_OK);
    cmd_result->set_property("detail", "hello from test extension 2");
    ten_env.return_result(std::move(cmd_result), std::move(cmd));
  }
};

class test_app : public ten::app_t {
 public:
  void on_configure(ten::ten_env_t &ten_env) override {
    bool rc = ten_env.init_property_from_json(
        // clang-format off
                 R"({
                      "_ten": {
                        "uri": "msgpack://127.0.0.1:8001/",
                        "log_level": 2
                      }
                    })"
        // clang-format on
        ,
        nullptr);
    ASSERT_EQ(rc, true);

    ten_env.on_configure_done();
  }
};

void *test_app_thread_main(TEN_UNUSED void *args) {
  auto *app = new test_app();
  app->run();
  delete app;

  return nullptr;
}

TEN_CPP_REGISTER_ADDON_AS_EXTENSION(
    basic_two_standalone_extension_1__test_extension_1, test_extension_1);

TEN_CPP_REGISTER_ADDON_AS_EXTENSION(
    basic_two_standalone_extension_1__test_extension_2, test_extension_2);

}  // namespace

TEST(BasicTest, TwoStandaloneExtension1) {  // NOLINT
  // Start app.
  auto *app_thread =
      ten_thread_create("app thread", test_app_thread_main, nullptr);

  // Create a client and connect to the app.
  auto *client = new ten::msgpack_tcp_client_t("msgpack://127.0.0.1:8001/");

  // Send graph.
  auto start_graph_cmd = ten::cmd_start_graph_t::create();
  start_graph_cmd->set_graph_from_json(R"({
           "nodes": [{
                "type": "extension",
                "name": "test_extension_1",
                "addon": "basic_two_standalone_extension_1__test_extension_1",
                "extension_group": "test_extension_group",
                "app": "msgpack://127.0.0.1:8001/"
             },{
                "type": "extension",
                "name": "test_extension_2",
                "addon": "basic_two_standalone_extension_1__test_extension_2",
                "extension_group": "test_extension_group",
                "app": "msgpack://127.0.0.1:8001/"
             }]
           })");
  auto cmd_result =
      client->send_cmd_and_recv_result(std::move(start_graph_cmd));
  ten_test::check_status_code(cmd_result, TEN_STATUS_CODE_OK);

  // Send a user-defined 'hello world' command.
  auto hello_world_cmd = ten::cmd_t::create("hello_world");
  hello_world_cmd->set_dest("msgpack://127.0.0.1:8001/", nullptr,
                            "test_extension_group", "test_extension_1");

  cmd_result = client->send_cmd_and_recv_result(std::move(hello_world_cmd));

  ten_test::check_status_code(cmd_result, TEN_STATUS_CODE_OK);
  ten_test::check_detail_with_string(cmd_result, "hello from test extension 1");

  hello_world_cmd = ten::cmd_t::create("hello_world");
  hello_world_cmd->set_dest("msgpack://127.0.0.1:8001/", nullptr,
                            "test_extension_group", "test_extension_2");

  cmd_result = client->send_cmd_and_recv_result(std::move(hello_world_cmd));

  ten_test::check_status_code(cmd_result, TEN_STATUS_CODE_OK);
  ten_test::check_detail_with_string(cmd_result, "hello from test extension 2");

  delete client;

  ten_thread_join(app_thread, -1);
}
