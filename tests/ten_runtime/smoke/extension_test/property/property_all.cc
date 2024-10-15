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
#include "ten_utils/lang/cpp/lib/value.h"
#include "ten_utils/lib/thread.h"
#include "ten_utils/macro/macros.h"
#include "tests/common/client/cpp/msgpack_tcp.h"
#include "tests/ten_runtime/smoke/extension_test/util/binding/cpp/check.h"

#define EXTENSION_EXT_PROP_NAME "ext_test_prop"
#define EXTENSION_EXT_PROP_TYPE float32
#define EXTENSION_EXT_PROP_VAL 36.78

#define APP_PROP_NAME "app_test_prop"
#define APP_PROP_TYPE string
#define APP_PROP_VAL "app_test_property_val"

#define EXTENSION_PROP_NAME_INT64 "extension_test_property_int64"
#define EXTENSION_PROP_TYPE_INT64 int64
#define EXTENSION_PROP_VAL_INT64 9132342

#define EXTENSION_PROP_NAME_BOOL "extension_test_property_bool"
#define EXTENSION_PROP_TYPE_BOOL bool
#define EXTENSION_PROP_VAL_BOOL false

#define CONN_PROP_NAME EXTENSION_EXT_PROP_NAME
#define CONN_PROP_VAL 92.78

namespace {

class test_extension : public ten::extension_t {
 public:
  explicit test_extension(const std::string &name) : ten::extension_t(name) {}

  void on_configure(ten::ten_env_t &ten_env) override {
    ten::ten_env_internal_accessor_t ten_env_internal_accessor(&ten_env);
    bool rc = ten_env_internal_accessor.init_manifest_from_json(
        // clang-format off
                 "{\
                    \"type\": \"extension\",\
                    \"name\": \"property_all__extension\",\
                    \"version\": \"1.0.0\",\
                    \"api\": {\
                      \"property\": {\
                        \"" EXTENSION_PROP_NAME_INT64 "\": {\
                          \"type\": \"int64\"\
                        }\
                      }\
                    }\
                  }"
        // clang-format on
    );
    ASSERT_EQ(rc, true);

    rc = ten_env.init_property_from_json(
                  "{\
          \"" EXTENSION_PROP_NAME_INT64 "\": " TEN_XSTR(EXTENSION_PROP_VAL_INT64) ",\
          \"" EXTENSION_PROP_NAME_BOOL "\": " TEN_XSTR(EXTENSION_PROP_VAL_BOOL) "}");
    ASSERT_EQ(rc, true);

    ten_env.on_configure_done();
  }

  void on_cmd(ten::ten_env_t &ten_env,
              std::unique_ptr<ten::cmd_t> cmd) override {
    nlohmann::json json = nlohmann::json::parse(cmd->to_json());
    if (json["_ten"]["name"] == "hello_world") {
      auto conn_property_value = ten_env.get_property_float64(CONN_PROP_NAME);

      auto extension_property_value_int64 =
          ten_env.get_property_int64(EXTENSION_PROP_NAME_INT64);

      auto extension_property_value_bool =
          ten_env.get_property_bool(EXTENSION_PROP_NAME_BOOL);

      // The value of a double-type value would be changed slightly after
      // transmission, serialize/deserialize from/to JSON.
      if ((fabs(conn_property_value - CONN_PROP_VAL) < 0.01) &&
          (extension_property_value_int64 == EXTENSION_PROP_VAL_INT64) &&
          (extension_property_value_bool == EXTENSION_PROP_VAL_BOOL)) {
        auto cmd_result = ten::cmd_result_t::create(TEN_STATUS_CODE_OK);
        cmd_result->set_property("detail", "hello world, too");
        ten_env.return_result(std::move(cmd_result), std::move(cmd));
      }
    }
  }
};

class test_app : public ten::app_t {
 public:
  void on_configure(ten::ten_env_t &ten_env) override {
    ten::ten_env_internal_accessor_t ten_env_internal_accessor(&ten_env);
    bool rc = ten_env_internal_accessor.init_manifest_from_json(
        // clang-format off
                 "{\
                    \"type\": \"app\",\
                    \"name\": \"test_app\",\
                    \"version\": \"1.0.0\",\
                    \"api\": {\
                      \"property\": {\
                        \"" APP_PROP_NAME "\": {\
                          \"type\": \"string\"\
                        }\
                      }\
                    }\
                  }"
        // clang-format on
    );
    ASSERT_EQ(rc, true);

    rc = ten_env.init_property_from_json(
        "{\
                    \"_ten\": {\
                    \"uri\": \"msgpack://127.0.0.1:8001/\"},\
                    \"" APP_PROP_NAME "\": " TEN_XSTR(APP_PROP_VAL) "}");
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

TEN_CPP_REGISTER_ADDON_AS_EXTENSION(property_all__extension, test_extension);

}  // namespace

TEST(ExtensionTest, PropertyAll) {  // NOLINT
  // Start app.
  auto *app_thread =
      ten_thread_create("app thread", test_app_thread_main, nullptr);

  // Create a client and connect to the app.
  auto *client = new ten::msgpack_tcp_client_t("msgpack://127.0.0.1:8001/");

  // Send graph with a property.
  nlohmann::json command =
      R"({
           "_ten": {
             "type": "start_graph",
             "seq_id": "55",
             "nodes": [{
               "type": "extension",
               "name": "test_extension",
               "app": "msgpack://127.0.0.1:8001/",
               "addon": "property_all__extension",
               "extension_group": "property_all__extension_group",
               "property": {}
             }]
           }
         })"_json;
  command["_ten"]["nodes"][0]["property"][CONN_PROP_NAME] = CONN_PROP_VAL;

  nlohmann::json resp = client->send_json_and_recv_resp_in_json(command);
  ten_test::check_status_code_is(resp, TEN_STATUS_CODE_OK);

  // Send a user-defined 'hello world' command.
  resp = client->send_json_and_recv_resp_in_json(
      R"({
           "_ten": {
             "name": "hello_world",
             "seq_id": "137",
             "dest":[{
               "app": "msgpack://127.0.0.1:8001/",
               "extension_group": "property_all__extension_group",
               "extension": "test_extension"
             }]
           }
         })"_json);
  ten_test::check_result_is(resp, "137", TEN_STATUS_CODE_OK,
                            "hello world, too");

  delete client;

  ten_thread_join(app_thread, -1);
}
