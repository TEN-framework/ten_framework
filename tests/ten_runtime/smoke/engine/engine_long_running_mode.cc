//
// Copyright © 2024 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#include <nlohmann/json.hpp>
#include <string>

#include "gtest/gtest.h"
#include "include_internal/ten_runtime/binding/cpp/ten.h"
#include "ten_utils/lib/thread.h"
#include "ten_utils/lib/time.h"
#include "tests/common/client/cpp/msgpack_tcp.h"
#include "tests/common/constant.h"
#include "tests/ten_runtime/smoke/util/binding/cpp/check.h"

namespace {

class ExtensionA : public ten::extension_t {
 public:
  ExtensionA(const std::string &name) : ten::extension_t(name) {}

  void on_cmd(ten::ten_env_t &ten_env,
              std::unique_ptr<ten::cmd_t> cmd) override {
    ten_env.send_cmd(std::move(cmd));
  }
};

class ExtensionB : public ten::extension_t {
 public:
  ExtensionB(const std::string &name) : ten::extension_t(name) {}

  void on_cmd(ten::ten_env_t &ten_env,
              std::unique_ptr<ten::cmd_t> cmd) override {
    nlohmann::json detail = {{"a", "b"}};

    auto cmd_result = ten::cmd_result_t::create(TEN_STATUS_CODE_OK);
    cmd_result->set_property_from_json("detail", detail.dump().c_str());

    ten_env.return_result(std::move(cmd_result), std::move(cmd));
  }
};

class test_app_a : public ten::app_t {
 public:
  void on_configure(ten::ten_env_t &ten_env) override {
    bool rc = ten_env.init_property_from_json(
        // clang-format off
                 R"({
                      "_ten": {
                        "uri": "msgpack://127.0.0.1:8001/",
                        "one_event_loop_per_engine": true,
                        "long_running_mode": true,
                        "log_level": 2
                      }
                    })"
        // clang-format on
    );
    ASSERT_EQ(rc, true);

    ten_env.on_configure_done();
  }
};

TEN_CPP_REGISTER_ADDON_AS_EXTENSION(engine_long_running_mode__extension_a,
                                    ExtensionA);
TEN_CPP_REGISTER_ADDON_AS_EXTENSION(engine_long_running_mode__extension_b,
                                    ExtensionB);

test_app_a *app_a = nullptr;

void *app_thread_1_main(TEN_UNUSED void *args) {
  app_a = new test_app_a();
  app_a->run();
  delete app_a;

  return nullptr;
}

class test_app_b : public ten::app_t {
 public:
  void on_configure(ten::ten_env_t &ten_env) override {
    bool rc = ten_env.init_property_from_json(
        // clang-format off
                 R"({
                      "_ten": {
                        "uri": "msgpack://127.0.0.1:8002/",
                        "long_running_mode": true,
                        "log_level": 2
                      }
                    })"
        // clang-format on
    );
    ASSERT_EQ(rc, true);

    ten_env.on_configure_done();
  }
};

test_app_b *app_b = nullptr;

void *app_thread_2_main(TEN_UNUSED void *args) {
  app_b = new test_app_b();
  app_b->run();
  delete app_b;

  return nullptr;
}

}  // namespace

TEST(ExtensionTest, EngineLongRunningMode) {  // NOLINT
  // Start app.
  auto *app_thread_2 =
      ten_thread_create("app thread 2", app_thread_2_main, nullptr);
  auto *app_thread_1 =
      ten_thread_create("app thread 1", app_thread_1_main, nullptr);

  // Create a client and connect to the app.
  ten::msgpack_tcp_client_t *client = nullptr;
  std::string graph_id;

  for (size_t i = 0; i < MULTIPLE_APP_SCENARIO_GRAPH_CONSTRUCTION_RETRY_TIMES;
       ++i) {
    client = new ten::msgpack_tcp_client_t("msgpack://127.0.0.1:8001/");

    // Send graph.
    auto start_graph_cmd = ten::cmd_start_graph_t::create();
    start_graph_cmd->set_long_running_mode(true);
    start_graph_cmd->set_graph_from_json(
        R"({
             "nodes": [{
               "type": "extension",
               "name": "A",
               "addon": "engine_long_running_mode__extension_a",
               "app": "msgpack://127.0.0.1:8001/",
               "extension_group": "engine_long_running_mode__extension_group_A"
             },{
               "type": "extension",
               "name": "B",
               "addon": "engine_long_running_mode__extension_b",
               "app": "msgpack://127.0.0.1:8002/",
               "extension_group": "engine_long_running_mode__extension_group_B"
             }],
             "connections": [{
               "app": "msgpack://127.0.0.1:8001/",
               "extension_group": "engine_long_running_mode__extension_group_A",
               "extension": "A",
               "cmd": [{
                 "name": "test",
                 "dest": [{
                   "app": "msgpack://127.0.0.1:8002/",
                   "extension_group": "engine_long_running_mode__extension_group_B",
                   "extension": "B"
                 }]
               }]
             }]
           })");

    auto cmd_result =
        client->send_cmd_and_recv_result(std::move(start_graph_cmd));

    if (cmd_result) {
      ten_test::check_status_code(cmd_result, TEN_STATUS_CODE_OK);
      graph_id = cmd_result->get_property_string("detail");

      break;
    } else {
      delete client;
      client = nullptr;

      // To prevent from busy re-trying.
      ten_sleep(10);
    }
  }

  TEN_ASSERT(client, "Failed to connect to the TEN app.");

  // now close connection
  delete client;

  // connect again, send request with graph_id directly
  client = new ten::msgpack_tcp_client_t("msgpack://127.0.0.1:8001/");

  // Send a user-defined 'hello world' command.
  auto test_cmd = ten::cmd_t::create("test");
  test_cmd->set_dest("msgpack://127.0.0.1:8001/", graph_id.c_str(),
                     "engine_long_running_mode__extension_group_A", "A");

  auto cmd_result = client->send_cmd_and_recv_result(std::move(test_cmd));

  ten_test::check_status_code(cmd_result, TEN_STATUS_CODE_OK);
  ten_test::check_detail_with_json(cmd_result, R"({"a": "b"})");

  // Destroy the client.
  delete client;

  if (app_a != nullptr) {
    app_a->close();
  }

  if (app_b != nullptr) {
    app_b->close();
  }

  ten_thread_join(app_thread_1, -1);
  ten_thread_join(app_thread_2, -1);
}