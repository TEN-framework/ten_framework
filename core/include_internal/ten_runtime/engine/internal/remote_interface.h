//
// This file is part of the TEN Framework project.
// See https://github.com/TEN-framework/ten_framework/LICENSE for license
// information.
//
#pragma once

#include "ten_runtime/ten_config.h"

#include "ten_utils/lib/smart_ptr.h"

typedef struct ten_engine_t ten_engine_t;
typedef struct ten_remote_t ten_remote_t;
typedef struct ten_connection_t ten_connection_t;

TEN_RUNTIME_PRIVATE_API void ten_engine_add_remote(ten_engine_t *self,
                                                   ten_remote_t *remote);

TEN_RUNTIME_PRIVATE_API void ten_engine_add_weak_remote(ten_engine_t *self,
                                                        ten_remote_t *remote);

TEN_RUNTIME_PRIVATE_API void ten_engine_upgrade_weak_remote_to_normal_remote(
    ten_engine_t *self, ten_remote_t *remote);

TEN_RUNTIME_PRIVATE_API void ten_engine_connect_to_graph_remote(
    ten_engine_t *self, const char *uri, ten_shared_ptr_t *cmd);

TEN_RUNTIME_PRIVATE_API void ten_engine_route_msg_to_remote(
    ten_engine_t *self, ten_shared_ptr_t *msg);

TEN_RUNTIME_PRIVATE_API ten_remote_t *ten_engine_check_remote_is_existed(
    ten_engine_t *self, const char *uri);

TEN_RUNTIME_PRIVATE_API bool ten_engine_check_remote_is_duplicated(
    ten_engine_t *self, const char *uri);

TEN_RUNTIME_PRIVATE_API bool ten_engine_check_remote_is_weak(
    ten_engine_t *self, ten_remote_t *remote);

TEN_RUNTIME_PRIVATE_API void ten_engine_on_remote_closed(ten_remote_t *remote,
                                                         void *on_closed_data);

TEN_RUNTIME_PRIVATE_API bool ten_engine_receive_msg_from_remote(
    ten_remote_t *remote, ten_shared_ptr_t *msg, void *user_data);

TEN_RUNTIME_PRIVATE_API ten_remote_t *ten_engine_find_remote(ten_engine_t *self,
                                                             const char *uri);

TEN_RUNTIME_PRIVATE_API void ten_engine_link_connection_to_remote(
    ten_engine_t *self, ten_connection_t *connection, const char *uri);
