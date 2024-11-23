//
// Copyright © 2024 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#include <nlohmann/json.hpp>

#include "tests/common/client/cpp/msgpack_tcp.h"

int main(int argc, char **argv) {
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
               "name": "test_extension",
               "addon": "default_extension_cpp",
               "app": "msgpack://127.0.0.1:8001/",
               "extension_group": "test_extension_group"
             }]
           }
         })"_json);
  TEN_ASSERT(TEN_STATUS_CODE_OK == cmd_result->get_status_code(),
             "Should not happen.");

  // Send a user-defined 'hello world' command.
  cmd_result = client->send_json_and_recv_result(
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
  TEN_ASSERT(TEN_STATUS_CODE_OK == cmd_result->get_status_code(),
             "Should not happen.");
  TEN_ASSERT(static_cast<std::string>("hello world, too") ==
                 cmd_result->get_property_string("detail"),
             "Should not happen.");

  delete client;
}
