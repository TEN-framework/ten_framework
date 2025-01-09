//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#include "gtest/gtest.h"
#include "include_internal/ten_runtime/binding/cpp/ten.h"
#include "ten_runtime/binding/cpp/detail/msg/cmd/stop_graph.h"
#include "tests/common/client/cpp/msgpack_tcp.h"
#include "tests/ten_runtime/smoke/util/binding/cpp/check.h"

namespace {

class test_normal_extension : public ten::extension_t {
 public:
  explicit test_normal_extension(const char *name) : ten::extension_t(name) {}

  void on_cmd(ten::ten_env_t &ten_env,
              std::unique_ptr<ten::cmd_t> cmd) override {
    if (cmd->get_name() == "hello_world") {
      auto cmd_result = ten::cmd_result_t::create(TEN_STATUS_CODE_OK);
      cmd_result->set_property("detail", "hello world, too");
      ten_env.return_result(std::move(cmd_result), std::move(cmd));
    }
  }
};

class test_predefined_graph : public ten::extension_t {
 public:
  explicit test_predefined_graph(const char *name) : ten::extension_t(name) {}

  void on_start(ten::ten_env_t &ten_env) override {
    auto start_graph_cmd = ten::cmd_start_graph_t::create();
    start_graph_cmd->set_dest("localhost", nullptr, nullptr, nullptr);
    start_graph_cmd->set_graph_from_json(R"({
      "nodes": [{
        "type": "extension",
        "name": "normal_extension",
        "addon": "start_graph_and_communication__normal_extension",
        "app": "msgpack://127.0.0.1:8001/",
        "extension_group": "start_graph_and_communication__normal_extension_group"
      }]
    })"_json.dump()
                                             .c_str());

    ten_env.send_cmd(
        std::move(start_graph_cmd),
        [this](ten::ten_env_t &ten_env, std::unique_ptr<ten::cmd_result_t> cmd,
               ten::error_t *err) {
          auto graph_id = cmd->get_property_string("detail");

          auto hello_world_cmd = ten::cmd_t::create("hello_world");
          hello_world_cmd->set_dest(
              "msgpack://127.0.0.1:8001/", graph_id.c_str(),
              "start_graph_and_communication__normal_extension_group",
              "normal_extension");
          ten_env.send_cmd(
              std::move(hello_world_cmd),
              [this, graph_id](ten::ten_env_t &ten_env,
                               std::unique_ptr<ten::cmd_result_t> cmd,
                               ten::error_t *err) {
                // Shut down the graph; otherwise, the app won't be able to
                // close because there is still a running engine/graph.
                auto stop_graph_cmd = ten::cmd_stop_graph_t::create();
                stop_graph_cmd->set_dest("localhost", nullptr, nullptr,
                                         nullptr);
                stop_graph_cmd->set_graph_id(graph_id.c_str());

                ten_env.send_cmd(
                    std::move(stop_graph_cmd),
                    [this](ten::ten_env_t &ten_env,
                           std::unique_ptr<ten::cmd_result_t> cmd,
                           ten::error_t *err) {
                      received_hello_world_resp = true;

                      if (test_cmd != nullptr) {
                        nlohmann::json detail = {{"id", 1}, {"name", "a"}};

                        auto cmd_result =
                            ten::cmd_result_t::create(TEN_STATUS_CODE_OK);
                        cmd_result->set_property_from_json(
                            "detail", detail.dump().c_str());
                        ten_env.return_result(std::move(cmd_result),
                                              std::move(test_cmd));
                      }
                    });
              });
        });

    ten_env.on_start_done();
  }

  void on_cmd(ten::ten_env_t &ten_env,
              std::unique_ptr<ten::cmd_t> cmd) override {
    if (cmd->get_name() == "test") {
      if (received_hello_world_resp) {
        nlohmann::json detail = {{"id", 1}, {"name", "a"}};

        auto cmd_result = ten::cmd_result_t::create(TEN_STATUS_CODE_OK);
        cmd_result->set_property_from_json("detail", detail.dump().c_str());
        ten_env.return_result(std::move(cmd_result), std::move(cmd));
      } else {
        test_cmd = std::move(cmd);
        return;
      }
    } else {
      TEN_ASSERT(0, "Should not happen.");
    }
  }

 private:
  bool received_hello_world_resp{};
  std::unique_ptr<ten::cmd_t> test_cmd;
};

class test_app : public ten::app_t {
 public:
  void on_configure(ten::ten_env_t &ten_env) override {
    bool rc = ten::ten_env_internal_accessor_t::init_manifest_from_json(
        ten_env,
        // clang-format off
                 R"({
                      "type": "app",
                      "name": "test_app",
                      "version": "0.1.0"
                    })"
        // clang-format on
    );
    ASSERT_EQ(rc, true);

    rc = ten_env.init_property_from_json(
        // clang-format off
                 R"({
                      "_ten": {
                        "uri": "msgpack://127.0.0.1:8001/",
                        "log_level": 2,
                        "predefined_graphs": [{
                          "name": "default",
                          "auto_start": false,
                          "singleton": true,
                          "nodes": [{
                            "type": "extension",
                            "name": "predefined_graph",
                            "addon": "start_graph_and_communication__predefined_graph_extension",
                            "extension_group": "start_graph_and_communication__predefined_graph_group"
                          }]
                        }]
                      }
                    })"
        // clang-format on
    );
    ASSERT_EQ(rc, true);

    ten_env.on_configure_done();
  }
};

void *app_thread_main(TEN_UNUSED void *args) {
  auto *app = new test_app();
  app->run();
  delete app;

  return nullptr;
}

TEN_CPP_REGISTER_ADDON_AS_EXTENSION(
    start_graph_and_communication__predefined_graph_extension,
    test_predefined_graph);
TEN_CPP_REGISTER_ADDON_AS_EXTENSION(
    start_graph_and_communication__normal_extension, test_normal_extension);

}  // namespace

TEST(StartGraphTest, StartGraphAndCommunication) {  // NOLINT
  auto *app_thread = ten_thread_create("app thread", app_thread_main, nullptr);

  // Create a client and connect to the app.
  auto *client = new ten::msgpack_tcp_client_t("msgpack://127.0.0.1:8001/");

  // Do not need to send 'start_graph' command first.
  // The 'graph_id' MUST be "default" (a special string) if we want to send the
  // request to predefined graph.
  auto test_cmd = ten::cmd_t::create("test");
  test_cmd->set_dest("msgpack://127.0.0.1:8001/", "default",
                     "start_graph_and_communication__predefined_graph_group",
                     "predefined_graph");
  auto cmd_result = client->send_cmd_and_recv_result(std::move(test_cmd));
  ten_test::check_status_code(cmd_result, TEN_STATUS_CODE_OK);
  ten_test::check_detail_with_json(cmd_result, R"({"id": 1, "name": "a"})");

  delete client;

  ten_thread_join(app_thread, -1);
}
