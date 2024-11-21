//
// Copyright © 2024 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#include <nlohmann/json.hpp>
#include <string>
#include <vector>

#include "gtest/gtest.h"
#include "include_internal/ten_runtime/binding/cpp/ten.h"
#include "ten_utils/lib/thread.h"
#include "tests/common/client/cpp/msgpack_tcp.h"
#include "tests/ten_runtime/smoke/extension_test/util/binding/cpp/check.h"

namespace {

/**
 *               |--> B -|
 * client --> A -|       |--> D
 *               |--> C -|
 */
class test_extension : public ten::extension_t {
 public:
  explicit test_extension(const std::string &name)
      : ten::extension_t(name), name_(name) {}

  void on_init(ten::ten_env_t &ten_env) override {
    is_leaf_node_ = ten_env.get_property_bool("is_leaf");
    ten_env.on_init_done();
  }

  void on_cmd(ten::ten_env_t &ten_env,
              std::unique_ptr<ten::cmd_t> cmd) override {
    nlohmann::json json = nlohmann::json::parse(cmd->to_json());

    if (is_leaf_node_) {
      json["return_from"] = name_;
      auto cmd_result = ten::cmd_result_t::create(TEN_STATUS_CODE_OK);
      cmd_result->set_property_from_json("detail", json.dump().c_str());
      ten_env.return_result(std::move(cmd_result), std::move(cmd));
      return;
    }

    std::vector<std::string> edges = {"B", "C"};
    if (json["_ten"]["name"] == "send") {
      json["from"] = name_;
      if (std::find(edges.begin(), edges.end(), name_) != edges.end()) {
        json["source"] = name_;
      }

      TEN_UNUSED bool const rc = cmd->from_json(json.dump().c_str());
      TEN_ASSERT(rc, "Should not happen.");

      ten_env.send_cmd_ex(
          std::move(cmd),
          [this, edges](ten::ten_env_t &ten_env,
                        std::unique_ptr<ten::cmd_result_t> result) {
            nlohmann::json json = nlohmann::json::parse(result->to_json());
            receive_count++;

            nlohmann::json detail = json["detail"];
            if (detail.type() == nlohmann::json::value_t::string) {
              detail = nlohmann::json::parse(json.value("detail", "{}"));
            }

            detail["return_from"] = name_;
            detail["success"] = true;
            detail["receive_count"] = receive_count;

            if (std::find(edges.begin(), edges.end(), name_) != edges.end()) {
              detail["success"] = (name_ == detail["source"]);
            }

            result->set_property_from_json("detail", detail.dump().c_str());

            ten_env.return_result_directly(std::move(result));
          });
    }
  }

 private:
  const std::string name_;
  bool is_leaf_node_{};
  int receive_count = 0;
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
    graph_polygon_in_one_app_return_all_1__extension, test_extension);

}  // namespace

TEST(ExtensionTest, GraphPolygonInOneAppReturnAll1) {  // NOLINT
  // Start app.
  auto *app_thread =
      ten_thread_create("app thread", test_app_thread_main, nullptr);

  // Create a client and connect to the app.
  auto *client = new ten::msgpack_tcp_client_t("msgpack://127.0.0.1:8001/");
  nlohmann::json resp = client->send_json_and_recv_resp_in_json(
      R"({
           "_ten": {
             "type": "start_graph",
             "seq_id": "55",
             "dest": [{
               "app": "msgpack://127.0.0.1:8001/"
             }],
             "nodes": [{
               "type": "extension",
               "name": "A",
               "addon": "graph_polygon_in_one_app_return_all_1__extension",
               "app": "msgpack://127.0.0.1:8001/",
               "extension_group": "graph_polygon_in_one_app_return_all_1__extension_group",
               "property": {
                 "is_leaf": false
                }
             },{
               "type": "extension",
               "name": "B",
               "addon": "graph_polygon_in_one_app_return_all_1__extension",
               "app": "msgpack://127.0.0.1:8001/",
               "extension_group": "graph_polygon_in_one_app_return_all_1__extension_group",
               "property": {
                 "is_leaf": false
                }
             },{
               "type": "extension",
               "name": "C",
               "addon": "graph_polygon_in_one_app_return_all_1__extension",
               "app": "msgpack://127.0.0.1:8001/",
               "extension_group": "graph_polygon_in_one_app_return_all_1__extension_group",
               "property": {
                 "is_leaf": false
                }
             },{
               "type": "extension",
               "name": "D",
               "addon": "graph_polygon_in_one_app_return_all_1__extension",
               "app": "msgpack://127.0.0.1:8001/",
               "extension_group": "graph_polygon_in_one_app_return_all_1__extension_group",
               "property": {
                 "is_leaf": true
                }
             }],
             "connections": [{
               "app": "msgpack://127.0.0.1:8001/",
               "extension_group": "graph_polygon_in_one_app_return_all_1__extension_group",
               "extension": "A",
               "cmd": [{
                 "name": "send",
                 "dest": [{
                   "app": "msgpack://127.0.0.1:8001/",
                   "extension_group": "graph_polygon_in_one_app_return_all_1__extension_group",
                   "extension": "B"
                 },{
                   "app": "msgpack://127.0.0.1:8001/",
                   "extension_group": "graph_polygon_in_one_app_return_all_1__extension_group",
                   "extension": "C"
                 }]
               }]
             },{
               "app": "msgpack://127.0.0.1:8001/",
               "extension_group": "graph_polygon_in_one_app_return_all_1__extension_group",
               "extension": "B",
               "cmd": [{
                 "name": "send",
                 "dest": [{
                   "app": "msgpack://127.0.0.1:8001/",
                   "extension_group": "graph_polygon_in_one_app_return_all_1__extension_group",
                   "extension": "D"
                 }]
               }]
             },{
               "app": "msgpack://127.0.0.1:8001/",
               "extension_group": "graph_polygon_in_one_app_return_all_1__extension_group",
               "extension": "C",
               "cmd": [{
                 "name": "send",
                 "dest": [{
                   "app": "msgpack://127.0.0.1:8001/",
                   "extension_group": "graph_polygon_in_one_app_return_all_1__extension_group",
                   "extension": "D"
                 }]
               }]
             },{
               "app": "msgpack://127.0.0.1:8001/",
               "extension_group": "graph_polygon_in_one_app_return_all_1__extension_group",
               "extension": "D"
             }]
           }
         })"_json);
  ten_test::check_status_code_is(resp, TEN_STATUS_CODE_OK);

  resp = client->send_json_and_recv_resp_in_json(
      R"({
           "_ten": {
             "name": "send",
             "seq_id": "137",
             "dest": [{
               "app": "msgpack://127.0.0.1:8001/",
               "extension_group": "graph_polygon_in_one_app_return_all_1__extension_group",
               "extension": "A"
             }]
           }
         })"_json);
  ten_test::check_status_code_is(resp, TEN_STATUS_CODE_OK);
  nlohmann::json detail = resp["detail"];

  EXPECT_EQ(detail["return_from"], "A");
  EXPECT_EQ(detail["success"], true);
  EXPECT_EQ(detail["receive_count"], 1);

  delete client;

  ten_thread_join(app_thread, -1);
}
