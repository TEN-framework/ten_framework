//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#include <nlohmann/json.hpp>
#include <string>
#include <thread>

#include "gtest/gtest.h"
#include "include_internal/ten_runtime/binding/cpp/ten.h"
#include "ten_runtime/binding/cpp/detail/ten_env_proxy.h"
#include "ten_utils/lang/cpp/lib/error.h"
#include "ten_utils/lib/thread.h"
#include "ten_utils/lib/time.h"
#include "tests/common/client/cpp/msgpack_tcp.h"
#include "tests/ten_runtime/smoke/util/binding/cpp/check.h"

namespace {

bool callback_is_called = false;

class test_extension_1 : public ten::extension_t {
 public:
  explicit test_extension_1(const char *name) : ten::extension_t(name) {}

  void on_start(ten::ten_env_t &ten_env) override {
    auto cmd = ten::cmd_t::create("return_immediately");
    ten_env.send_cmd(std::move(cmd));

    ten_env.on_start_done();
  }

  void on_cmd(ten::ten_env_t &ten_env,
              std::unique_ptr<ten::cmd_t> cmd) override {
    if (cmd->get_name() == "close_app") {
      auto close_app_cmd = ten::cmd_close_app_t::create();
      close_app_cmd->set_dest("localhost", nullptr, nullptr, nullptr);
      ten_env.send_cmd(std::move(close_app_cmd));

      auto cmd_result = ten::cmd_result_t::create(TEN_STATUS_CODE_OK);
      cmd_result->set_property("detail", "app closed");

      ten_env.return_result(std::move(cmd_result), std::move(cmd));
      return;
    }
  }
};

class test_extension_2 : public ten::extension_t {
 public:
  explicit test_extension_2(const char *name) : ten::extension_t(name) {}

  void on_cmd(ten::ten_env_t &ten_env,
              std::unique_ptr<ten::cmd_t> cmd) override {
    if (cmd->get_name() == "return_after_3_second") {
      auto *ten_env_proxy = ten::ten_env_proxy_t::create(ten_env);

      auto cmd_shared =
          std::make_shared<std::unique_ptr<ten::cmd_t>>(std::move(cmd));

      thread_ = std::thread([ten_env_proxy, cmd_shared]() {
        ten_sleep(3000);

        bool rc = ten_env_proxy->notify(
            [ten_env_proxy, cmd_shared](ten::ten_env_t &ten_env) {
              callback_is_called = true;

              // At this moment, ten_env has been closed because
              // ten_env.on_deinit_done() has been called. So all API calls to
              // ten_env will result in an error.

              ten::error_t err;

              auto rc = ten_env.return_result(
                  ten::cmd_result_t::create(TEN_STATUS_CODE_OK),
                  std::move(*cmd_shared), nullptr, &err);
              ASSERT_FALSE(rc);
              ASSERT_EQ(err.get_c_error()->err_no, TEN_ERRNO_TEN_IS_CLOSED);

              ten::error_t err2;
              auto str = ten_env.get_property_to_json("test_property", &err2);
              ASSERT_EQ(str, "");
              ASSERT_EQ(err2.get_c_error()->err_no, TEN_ERRNO_TEN_IS_CLOSED);

              ten::error_t err3;
              auto new_cmd = ten::cmd_t::create("new_cmd");
              auto rc2 = ten_env.send_cmd(std::move(new_cmd), nullptr, &err3);
              ASSERT_FALSE(rc2);
              ASSERT_EQ(err3.get_c_error()->err_no, TEN_ERRNO_TEN_IS_CLOSED);

              delete ten_env_proxy;
            });

        // The presence of ten_env_proxy prevents the extension runloop from
        // stopping, so the notify() will succeed.
        ASSERT_TRUE(rc);
      });

      return;
    } else if (cmd->get_name() == "return_immediately") {
      auto cmd_result = ten::cmd_result_t::create(TEN_STATUS_CODE_OK);
      cmd_result->set_property("detail", "done");
      ten_env.return_result(std::move(cmd_result), std::move(cmd));
      return;
    }
  }

  void on_stop(ten::ten_env_t &ten_env) override { ten_env.on_stop_done(); }

  void on_deinit(ten::ten_env_t &ten_env) override {
    if (thread_.joinable()) {
      thread_.join();
    }

    ten_env.on_deinit_done();
  }

 private:
  std::thread thread_;
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
    call_ten_api_after_on_deinit_done__test_extension_1, test_extension_1);
TEN_CPP_REGISTER_ADDON_AS_EXTENSION(
    call_ten_api_after_on_deinit_done__test_extension_2, test_extension_2);

}  // namespace

TEST(CloseAppTest, CallTenApiAfterOnDeinitDone) {  // NOLINT
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
                "addon": "call_ten_api_after_on_deinit_done__test_extension_1",
                "extension_group": "basic_extension_group_1",
                "app": "msgpack://127.0.0.1:8001/"
             },{
                "type": "extension",
                "name": "test_extension_2",
                "addon": "call_ten_api_after_on_deinit_done__test_extension_2",
                "extension_group": "basic_extension_group_2",
                "app": "msgpack://127.0.0.1:8001/",
                "property": {
                  "test_property": "test_value"
                }
             }],
             "connections": [{
               "app": "msgpack://127.0.0.1:8001/",
               "extension": "test_extension_1",
               "cmd": [{
                 "name": "return_immediately",
                 "dest": [{
                   "app": "msgpack://127.0.0.1:8001/",
                   "extension": "test_extension_2"
                 }]
               }]
             }]
           })");
  auto cmd_result =
      client->send_cmd_and_recv_result(std::move(start_graph_cmd));
  ten_test::check_status_code(cmd_result, TEN_STATUS_CODE_OK);

  // Send a return_after_3_second command.
  auto return_after_3_second_cmd = ten::cmd_t::create("return_after_3_second");
  return_after_3_second_cmd->set_dest("msgpack://127.0.0.1:8001/", nullptr,
                                      "basic_extension_group_2",
                                      "test_extension_2");
  client->send_cmd(std::move(return_after_3_second_cmd));

  // Wait 1 second to make sure the return_after_3_second command is
  // processed.
  ten_sleep(1000);

  // Send a close_app command.
  auto close_app_cmd = ten::cmd_t::create("close_app");
  close_app_cmd->set_dest("msgpack://127.0.0.1:8001/", nullptr,
                          "basic_extension_group_1", "test_extension_1");
  client->send_cmd(std::move(close_app_cmd));

  ten_thread_join(app_thread, -1);

  delete client;

  EXPECT_EQ(callback_is_called, true);
}