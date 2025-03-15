//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#pragma once

#include "ten_runtime/ten_config.h"

#include "include_internal/ten_runtime/protocol/integrated/retry.h"
#include "include_internal/ten_runtime/protocol/protocol.h"
#include "ten_utils/io/stream.h"
#include "ten_utils/io/transport.h"

typedef struct ten_protocol_integrated_t ten_protocol_integrated_t;
typedef struct ten_timer_t ten_timer_t;

typedef void (*ten_protocol_integrated_on_input_func_t)(
    ten_protocol_integrated_t *protocol, ten_buf_t buf, ten_list_t *input);

typedef ten_buf_t (*ten_protocol_integrated_on_output_func_t)(
    ten_protocol_integrated_t *protocol, ten_list_t *output);

typedef struct ten_protocol_integrated_connect_to_context_t {
  // The protocol which is trying to connect to the server.
  ten_protocol_integrated_t *protocol;

  // The server URI to connect to.
  ten_string_t server_uri;

  // The callback function to be called when the connection is established or
  // failed.
  //
  // @note Set to NULL if the callback has been called.
  ten_protocol_on_server_connected_func_t on_server_connected;

  void *user_data;
} ten_protocol_integrated_connect_to_context_t;

/**
 * @brief Base protocol implementation that integrates with TEN's event loop
 * system.
 *
 * This protocol serves as the foundation for all protocol implementations that
 * operate within TEN's event loop architecture. It handles communication
 * streams, manages the protocol lifecycle (connection, data transfer, and
 * closure), and provides integration with TEN's threading model.
 *
 * Integrated protocols support both listening (server) and communication
 * (client) roles, with appropriate resource management for each role.
 */
struct ten_protocol_integrated_t {
  // Base protocol that all protocol implementations must inherit from.
  ten_protocol_t base;

  // Role-specific resources that vary depending on protocol function.
  union {
    // For server-side protocols (LISTENING role):
    // Manages incoming connection requests.
    ten_transport_t *listening_transport;

    // For client-side protocols (COMMUNICATION role):
    // Handles data transfer for an established connection.
    ten_stream_t *communication_stream;
  } role_facility;

  // Protocol message conversion functions:

  // Deserializes raw network buffers into TEN runtime messages.
  // Called when data is received from the network.
  ten_protocol_integrated_on_input_func_t on_input;

  // Serializes TEN runtime messages into raw network buffers.
  // Called when messages need to be sent over the network.
  ten_protocol_integrated_on_output_func_t on_output;

  // Connection retry mechanism:
  // Configuration parameters for connection retry attempts.
  ten_protocol_integrated_retry_config_t retry_config;

  // Timer that schedules retry attempts when connections fail.
  ten_timer_t *retry_timer;
};

TEN_RUNTIME_API void ten_protocol_integrated_init(
    ten_protocol_integrated_t *self, const char *name,
    ten_protocol_integrated_on_input_func_t on_input,
    ten_protocol_integrated_on_output_func_t on_output);

TEN_RUNTIME_PRIVATE_API ten_protocol_integrated_connect_to_context_t *
ten_protocol_integrated_connect_to_context_create(
    ten_protocol_integrated_t *self, const char *server_uri,
    ten_protocol_on_server_connected_func_t on_server_connected,
    void *user_data);

TEN_RUNTIME_PRIVATE_API void ten_protocol_integrated_connect_to_context_destroy(
    ten_protocol_integrated_connect_to_context_t *context);
