//
// Copyright © 2024 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#include <memory>
#include <nlohmann/json.hpp>
#include <string>

#include "gtest/gtest.h"
#include "include_internal/ten_runtime/binding/cpp/ten.h"
#include "ten_runtime/binding/cpp/internal/msg/cmd/cmd.h"
#include "ten_utils/lib/thread.h"
#include "tests/common/client/cpp/msgpack_tcp.h"
#include "tests/ten_runtime/smoke/extension_test/util/binding/cpp/check.h"

#define TEST_DATA 12344321

namespace {

class test_extension_1 : public ten::extension_t {
 public:
  explicit test_extension_1(const std::string &name) : ten::extension_t(name) {}

  void on_configure(ten::ten_env_t &ten_env) override {
    // clang-format off
        ten::ten_env_internal_accessor_t ten_env_internal_accessor(&ten_env);
    bool rc = ten_env_internal_accessor.init_manifest_from_json(
                 R"({
                      "type": "extension",
                      "name": "msg_12__extension_1",
                      "version": "0.1.0",
                                          "api": {
                        "cmd_out": [
                          {
                            "name": "test",
                            "property": {
                              "test_data": {
                                "type": "int32"
                              }
                            }
                          }
                        ]
                      }
                    })");
    // clang-format on
    ASSERT_EQ(rc, true);

    ten_env.on_configure_done();
  }

  void on_cmd(ten::ten_env_t &ten_env,
              std::unique_ptr<ten::cmd_t> cmd) override {
    if (std::string(cmd->get_name()) == "hello_world") {
      hello_world_cmd = std::move(cmd);

      auto test_cmd = ten::cmd_t::create("test");
      test_cmd->set_property("test_data", TEST_DATA);

      ten_env.send_cmd(
          std::move(test_cmd), [this](ten::ten_env_t &ten_env,
                                      std::unique_ptr<ten::cmd_result_t> cmd) {
            auto cmd_result = ten::cmd_result_t::create(TEN_STATUS_CODE_OK);
            cmd_result->set_property("detail",
                                     cmd->get_property_string("detail"));
            ten_env.return_result(std::move(cmd_result),
                                  std::move(hello_world_cmd));
          });
    }
  }

 private:
  std::unique_ptr<ten::cmd_t> hello_world_cmd;
};

class test_extension_2 : public ten::extension_t {
 public:
  explicit test_extension_2(const std::string &name) : ten::extension_t(name) {}

  void on_cmd(ten::ten_env_t &ten_env,
              std::unique_ptr<ten::cmd_t> cmd) override {
    if (std::string(cmd->get_name()) == "test") {
      auto const test_data = cmd->get_property_int32("test_data");
      TEN_ASSERT(test_data == TEST_DATA, "Invalid argument.");

      auto cmd_result = ten::cmd_result_t::create(TEN_STATUS_CODE_OK);
      cmd_result->set_property("detail", "hello world, too");
      ten_env.return_result(std::move(cmd_result), std::move(cmd));
    }
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

void *test_app_thread_main(TEN_UNUSED void *arg) {
  auto *app = new test_app();
  app->run();
  delete app;

  return nullptr;
}

TEN_CPP_REGISTER_ADDON_AS_EXTENSION(msg_12__extension_1, test_extension_1);
TEN_CPP_REGISTER_ADDON_AS_EXTENSION(msg_12__extension_2, test_extension_2);

}  // namespace

TEST(MsgTest, Msg12) {  // NOLINT
  // Start app.
  auto *app_thread =
      ten_thread_create("app thread", test_app_thread_main, nullptr);

  // Create a client and connect to the app.
  auto *client = new ten::msgpack_tcp_client_t("msgpack://127.0.0.1:8001/");

  // Send graph.
  auto cmd_result = client->send_json_and_recv_result(
      R"({
           "_ten": {
             "type": "start_graph",
             "seq_id": "55",
             "nodes": [{
               "type": "extension",
               "name": "msg_12__extension_1",
               "addon": "msg_12__extension_1",
               "app": "msgpack://127.0.0.1:8001/",
               "extension_group": "msg_12__extension_group_1"
             },{
               "type": "extension",
               "name": "msg_12__extension_2",
               "addon": "msg_12__extension_2",
               "app": "msgpack://127.0.0.1:8001/",
               "extension_group": "msg_12__extension_group_2"
             }],
             "connections": [{
               "app": "msgpack://127.0.0.1:8001/",
               "extension_group": "msg_12__extension_group_1",
               "extension": "msg_12__extension_1",
               "cmd": [{
                 "name": "test",
                 "dest": [{
                   "app": "msgpack://127.0.0.1:8001/",
                   "extension_group": "msg_12__extension_group_2",
                   "extension": "msg_12__extension_2"
                 }]
               }]
             }]
           }
         })"_json);
  ten_test::check_status_code(cmd_result, TEN_STATUS_CODE_OK);

  // Send a user-defined 'hello world' command.
  cmd_result = client->send_json_and_recv_result(
      R"({
           "_ten": {
             "name": "hello_world",
             "seq_id": "137",
             "dest": [{
               "app": "msgpack://127.0.0.1:8001/",
               "extension_group": "msg_12__extension_group_1",
               "extension": "msg_12__extension_1"
             }]
           }
         })"_json);
  ten_test::check_status_code(cmd_result, TEN_STATUS_CODE_OK);
  ten_test::check_detail_with_string(cmd_result, "hello world, too");

  delete client;

  ten_thread_join(app_thread, -1);
}
