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
#include "ten_utils/lang/cpp/lib/value.h"
#include "ten_utils/lib/thread.h"
#include "tests/common/client/cpp/msgpack_tcp.h"
#include "tests/ten_runtime/smoke/extension_test/util/binding/cpp/check.h"

namespace {

class test_extension : public ten::extension_t {
 public:
  explicit test_extension(const std::string &name) : ten::extension_t(name) {}

  void on_cmd(ten::ten_env_t &ten_env,
              std::unique_ptr<ten::cmd_t> cmd) override {
    auto mode = ten_env.get_property_string("widget_props");
    if (mode.empty()) {
      mode = "default";
    }

    nlohmann::json json = nlohmann::json::parse(cmd->to_json());
    if (json["_ten"]["name"] == "hello_world") {
      auto cmd_result = ten::cmd_result_t::create(TEN_STATUS_CODE_OK);
      cmd_result->set_property("detail", mode);
      ten_env.return_result(std::move(cmd_result), std::move(cmd));
    }
  }
};

class test_app : public ten::app_t {
 public:
  void on_configure(ten::ten_env_t &ten_env) override {
    ten::ten_env_internal_accessor_t ten_env_internal_accessor(&ten_env);
    bool rc = ten_env_internal_accessor.init_manifest_from_json(
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
                 R"###({
                      "_ten": {
                        "uri": "msgpack://127.0.0.1:8001/",
                        "log_level": 2,
                        "predefined_graphs": [{
                           "name": "default",
                           "auto_start": true,
                           "nodes": [{
                             "type": "extension_group",
                             "name": "property_predefined_graph",
                             "addon": "default_extension_group"
                           },{
                             "type": "extension",
                             "name": "property_predefined_graph",
                             "addon": "property_predefined_graph__extension",
                             "extension_group": "property_predefined_graph",
                             "property": {
                               "widget_props": "1"
                             }
                           },{
                             "type": "extension",
                             "name": "property_predefined_graph_no_prop",
                             "addon": "property_predefined_graph__extension",
                             "extension_group": "property_predefined_graph"
                           }]
                         }]
                        }
                    })###"
        // clang-format on
    );
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

TEN_CPP_REGISTER_ADDON_AS_EXTENSION(property_predefined_graph__extension,
                                    test_extension);

}  // namespace

TEST(ExtensionTest, PropertyPredefinedGraph) {  // NOLINT
  // Start app.
  auto *app_thread =
      ten_thread_create("app thread", test_app_thread_main, nullptr);

  // Create a client and connect to the app.
  auto *client = new ten::msgpack_tcp_client_t("msgpack://127.0.0.1:8001/");

  // Send a user-defined 'hello world' command.
  nlohmann::json resp = client->send_json_and_recv_resp_in_json(
      R"({
           "_ten": {
             "name": "hello_world",
             "seq_id": "137",
             "dest":[{
               "app": "msgpack://127.0.0.1:8001/",
               "graph": "default",
               "extension_group": "property_predefined_graph",
               "extension": "property_predefined_graph"
             }]
           }
         })"_json);
  ten_test::check_result_is(resp, "137", TEN_STATUS_CODE_OK, "1");

  resp = client->send_json_and_recv_resp_in_json(
      R"({
           "_ten": {
             "name": "hello_world",
             "seq_id": "137",
             "dest":[{
               "app": "msgpack://127.0.0.1:8001/",
               "graph": "default",
               "extension_group": "property_predefined_graph",
               "extension": "property_predefined_graph_no_prop"
             }]
           }
         })"_json);
  ten_test::check_result_is(resp, "137", TEN_STATUS_CODE_OK, "default");

  // Destroy the client.
  delete client;

  ten_thread_join(app_thread, -1);
}
