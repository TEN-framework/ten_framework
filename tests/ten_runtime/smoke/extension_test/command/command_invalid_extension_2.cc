//
// Copyright © 2024 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#include <memory>
#include <nlohmann/json.hpp>
#include <string>
#include <vector>

#include "gtest/gtest.h"
#include "include_internal/ten_runtime/binding/cpp/ten.h"
#include "ten_utils/lib/thread.h"
#include "tests/common/client/cpp/msgpack_tcp.h"
#include "tests/ten_runtime/smoke/extension_test/util/binding/cpp/check.h"

namespace {

class test_extension : public ten::extension_t {
 public:
  explicit test_extension(const std::string &name) : ten::extension_t(name) {}

  void on_cmd(ten::ten_env_t &ten_env,
              std::unique_ptr<ten::cmd_t> cmd) override {
    nlohmann::json json = nlohmann::json::parse(cmd->to_json());

    if (json["_ten"]["name"] == "hello_world") {
      requested_cmd = std::move(cmd);

      // We send out a request with invalid extension, the extension thread will
      // return the error result.
      nlohmann::json request = R"({
        "_ten": {
        "name": "test",
        "dest": [
          {
            "app": "localhost",
            "extension": "a",
            "extension_group": "test_extension_group"
          }
        ]
              }
        })"_json;
      ten_env.send_json(
          request.dump().c_str(),
          [this](ten::ten_env_t &ten_env,
                 std::unique_ptr<ten::cmd_result_t> result) {
            nlohmann::json json = nlohmann::json::parse(result->to_json());
            auto cmd_result =
                ten::cmd_result_t::create(result->get_status_code());
            cmd_result->set_property("detail", json.value("detail", ""));
            ten_env.return_result(std::move(cmd_result),
                                  std::move(requested_cmd));
          });
      return;
    }
  }

 private:
  std::unique_ptr<ten::cmd_t> requested_cmd;
};

class test_extension_group : public ten::extension_group_t {
 public:
  explicit test_extension_group(const std::string &name)
      : ten::extension_group_t(name) {}

  void on_create_extensions(ten::ten_env_t &ten_env) override {
    std::vector<ten::extension_t *> extensions;
    extensions.push_back(new test_extension("test_extension"));
    ten_env.on_create_extensions_done(extensions);
  }

  void on_destroy_extensions(
      ten::ten_env_t &ten_env,
      const std::vector<ten::extension_t *> &extensions) override {
    for (auto *extension : extensions) {
      delete extension;
    }
    ten_env.on_destroy_extensions_done();
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

TEN_CPP_REGISTER_ADDON_AS_EXTENSION_GROUP(
    command_invalid_extension_2__extension_group, test_extension_group);

}  // namespace

TEST(ExtensionTest, CommandInvalidExtension2) {  // NOLINT
  auto *app_thread =
      ten_thread_create("app thread", test_app_thread_main, nullptr);

  // Create a client and connect to the app.
  auto *client = new ten::msgpack_tcp_client_t("msgpack://127.0.0.1:8001/");

  // Send graph.
  nlohmann::json resp = client->send_json_and_recv_resp_in_json(
      R"({
           "_ten": {
             "type": "start_graph",
             "seq_id": "55",
             "nodes": [{
               "type": "extension_group",
               "name": "test_extension_group",
               "addon": "command_invalid_extension_2__extension_group",
               "app": "msgpack://127.0.0.1:8001/"
             }]
           }
         })"_json);
  ten_test::check_status_code_is(resp, TEN_STATUS_CODE_OK);

  // Send a user-defined 'hello world' command.
  resp = client->send_json_and_recv_resp_in_json(
      R"({
           "_ten": {
             "name": "hello_world",
             "seq_id": "137",
             "dest": [{
               "app": "msgpack://127.0.0.1:8001/",
               "extension_group": "test_extension_group",
               "extension": "test_extension"
             }]
           }
         })"_json);
  ten_test::check_result_is(resp, "137", TEN_STATUS_CODE_ERROR,
                            "The extension[a] is invalid.");

  delete client;

  ten_thread_join(app_thread, -1);
}
