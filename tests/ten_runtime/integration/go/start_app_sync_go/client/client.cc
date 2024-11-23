//
// Copyright © 2024 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#include <nlohmann/json.hpp>

#include "ten_utils/macro/mark.h"
#include "tests/common/client/cpp/msgpack_tcp.h"

int main(TEN_UNUSED int argc, TEN_UNUSED char **argv) {
  // Create a client and connect to the app.
  auto *client = new ten::msgpack_tcp_client_t("msgpack://127.0.0.1:8007/");

  auto cmd_result = client->send_json_and_recv_result(
      R"({
    "_ten": {
      "name": "hello",
      "seq_id": "238",
      "dest": [{
        "app": "msgpack://127.0.0.1:8007/",
        "graph": "default",
        "extension_group": "default_extension_group",
        "extension": "default_extension_go"
      }]
    }
  })"_json);
  TEN_ASSERT(TEN_STATUS_CODE_OK == cmd_result->get_status_code(),
             "Should not happen.");

  // NOTE the order: client destroy, then connection lost, then nodejs exits
  delete client;
}
